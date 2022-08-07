mod entities;
#[macro_use]
extern crate lazy_static;
extern crate tera;

use actix_web::{
    get,
    http::header::{self, HeaderMap},
    middleware, post, web, App, Error as WebError, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use entities::*;
use entities::{prelude::*, sea_orm_active_enums::ResponseType};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use serde::Deserialize;
use std::{env, error::Error};
use tera::{Context, Tera};
use uuid::Uuid;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let tera = match Tera::new("src/templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing errors: {}", e);
                ::std::process::exit(1);
            }
        };
        tera
    };
}

fn compare_scope_strings(requested: &str, allowed: &str) -> bool {
    let req_scopes: Vec<&str> = requested.split_whitespace().collect();
    let allowed_scopes: Vec<&str> = allowed.split_whitespace().collect();

    req_scopes
        .iter()
        .all(|scope| allowed_scopes.contains(scope))
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    client_id: String,
    redirect_uri: String,
    response_type: ResponseType,
    state: String,
    scope: Option<String>,
}

#[get("/authorize")]
async fn authorize(
    request: web::Query<AuthRequest>,
    db: web::Data<AppState>,
) -> Result<HttpResponse, WebError> {
    let client = Clients::find()
        .filter(clients::Column::Uuid.eq(request.client_id.clone()))
        .one(&db.conn)
        .await
        .unwrap(); //TODO: Handle bad db response

    match client {
        Some(client) => {
            if client.redirect_uri == request.redirect_uri {
                //split scopes and compare to make sure all requested scopes are available
                let requested_scope: String = request.scope.clone().unwrap_or_default();
                if !compare_scope_strings(&requested_scope, &client.scope) {
                    return Ok(HttpResponse::BadRequest().body("invalid scope"));
                }

                // generate code and save request
                let requests_id = Uuid::new_v4().to_string();

                let record = requests::ActiveModel {
                    uuid: Set(requests_id.clone()),
                    client_id: Set(client.id),
                    scope: Set(requested_scope.clone()),
                    redirect_uri: Set(request.redirect_uri.clone()),
                    request_type: Set(request.response_type.to_owned()),
                    csrf_token: Set(request.state.to_owned()),
                    ..Default::default()
                };

                //TODO: Handle bad db response
                let _db_resp = record.insert(&db.conn).await;

                let mut context = Context::new();
                context.insert("client", &client);
                context.insert("reqid", &requests_id);
                context.insert(
                    "scope",
                    &requested_scope.split_whitespace().collect::<Vec<&str>>(),
                );

                let body = match TEMPLATES.render("approve.html", &context) {
                    Ok(s) => s,
                    Err(e) => {
                        println!("Error: {}", e);
                        let mut cause = e.source();
                        while let Some(e) = cause {
                            println!("Reason: {}", e);
                            cause = e.source();
                        }
                        e.to_string()
                    }
                };

                Ok(HttpResponse::Ok().content_type("text/html").body(body))
            } else {
                Ok(HttpResponse::BadRequest().body(format!(
                    "Mismatched redirect URI, expected {} got {}",
                    client.redirect_uri, request.redirect_uri
                )))
            }
        }
        None => {
            Ok(HttpResponse::BadRequest().body(format!("Unknown client {}", request.client_id)))
        }
    }
}

#[derive(Debug, Deserialize)]
struct ApproveForm {
    approve: String,
    scope: Option<Vec<String>>,
    reqid: String,
}

#[post("/approve")]
async fn approve(
    form: web::Form<ApproveForm>,
    db: web::Data<AppState>,
) -> Result<HttpResponse, WebError> {
    // check request_id from recorded requests
    println!("{:?} \n\n", form);

    let request = Requests::find()
        .filter(requests::Column::Uuid.eq(form.reqid.to_owned()))
        .one(&db.conn)
        .await
        .unwrap(); //TODO: Handle bad db response

    match request {
        None => Ok(HttpResponse::BadRequest().body("No matching authorization request")),
        Some(request) => {
            if !form.approve.eq("Approve") {
                Ok(HttpResponse::BadRequest().body("Access denied"))
            } else {
                if request.request_type != ResponseType::Code {
                    return Ok(HttpResponse::BadRequest().body("Unsupported Response Type"));
                }

                let client: Option<entities::clients::Model> =
                    Clients::find_by_id(request.client_id)
                        .one(&db.conn)
                        .await
                        .unwrap(); //TODO: Handle bad db response

                if client.is_none() {
                    return Ok(HttpResponse::BadRequest().body("Request has no client"));
                }
                let client = client.unwrap();

                //TODO: get scope from form body
                let mut requested_scope = String::new();
                if let Some(mut rscope) = form.scope.to_owned() {
                    let tmp = rscope
                        .iter_mut()
                        .map(|s| s.replace("scope_", ""))
                        .collect::<String>();
                    requested_scope.push_str(&tmp)
                }
                if !compare_scope_strings(&requested_scope, &client.scope) {
                    return Ok(HttpResponse::BadRequest().body("invalid scope"));
                }

                let code = Uuid::new_v4().to_string();
                let code_record = codes::ActiveModel {
                    uuid: Set(code.clone()),
                    request_id: Set(request.id),
                    ..Default::default()
                }; //TODO: add user

                let redirect_uri = format!(
                    "{}?code={}&state={}",
                    request.redirect_uri, code, request.csrf_token
                );

                let _new = code_record.insert(&db.conn).await;
                //TODO: handle bad insert

                Ok(HttpResponse::Found()
                    .insert_header((header::LOCATION, redirect_uri.to_string()))
                    .finish())
            }
        }
    }
}

// Post /Token (auth_header, client_id, )
// auth header should have clientId and clientSecret
// if not check the post body for clientId and clientSecret
// - check that they didn't attempt to authenticate with multiple methods (both)
// check for record of client_id
// compare req.secret with recorded client secret
// check if req.grant_type is authorization_code
// check/get record of original request from req.code
// --- burn record of req + code as it has been used
// compare original request client_id to req.client_id
// generate access token
// save access token to client_id with scope
#[derive(Debug, Deserialize)]
struct TokenRequest {
    client_id: String,
    client_secret: String,
}

#[post("/token")]
async fn token(
    req: HttpRequest,
    _data: Option<web::Json<TokenRequest>>,
) -> Result<HttpResponse, WebError> {
    let mut client_id = String::new();
    let mut client_secret = String::new();

    if let Some(auth_header) = req.headers().get("authorization") {
        let auth_str: String = auth_header.to_str().unwrap().replace("basic ", "");
        let auth: Vec<&str> = auth_str.split(':').collect();
        client_id.push_str(auth.get(0).unwrap_or(&""));
        client_secret.push_str(auth.get(1).unwrap_or(&""));

        if client_id.is_empty() || client_secret.is_empty() {
            return Ok(HttpResponse::BadRequest().body("Bad Header"));
        }
    }
    print!("{} - {}\n", client_id, client_secret);
    Ok(HttpResponse::BadRequest().body("end of test"))
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Debug, Clone)]
struct AppState {
    conn: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    // get env vars
    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let host = env::var("HOST").expect("HOST is not set");
    let port = env::var("PORT")
        .expect("PORT is not set")
        .parse::<u16>()
        .unwrap();
    let server_url = format!("{}:{}", host, port);

    //tracing is a framework for instrumenting Rust programs to collect scoped, structured, and async-aware diagnostics
    tracing_subscriber::fmt::init();

    // connect to database and create tables if not exist
    let connection = sea_orm::Database::connect(&db_url).await.unwrap();
    Migrator::up(&connection, None).await.unwrap();
    let state = AppState { conn: connection };

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default())
            .service(authorize)
            .service(approve)
            .service(token)
    })
    .bind((host.as_str(), port))?;

    println!("Starting server at {}", server_url);
    server.run().await?;

    Ok(())
}
