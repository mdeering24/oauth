use super::m20220720_125359_client::Clients;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_query::Table::create()
                    .table(Requests::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Requests::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Requests::Uuid).string().not_null())
                    .col(ColumnDef::new(Requests::ClientId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-request-client_id")
                            .from(Requests::Table, Requests::ClientId)
                            .to(Clients::Table, Clients::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned()
                    .col(ColumnDef::new(Requests::Scope).string().not_null())
                    .col(ColumnDef::new(Requests::RedirectUri).string().not_null())
                    .col(ColumnDef::new(Requests::Token).string().not_null())
                    .col(
                        ColumnDef::new(Requests::RequestType)
                            .enumeration("response_type", ["my_variant"])
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Requests::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Requests {
    Id,
    Uuid,
    ClientId,
    Scope,
    RedirectUri,
    RequestType,
    #[iden = "csrf_token"]
    Token,
    Table,
}
