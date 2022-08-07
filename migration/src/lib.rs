pub use sea_orm_migration::prelude::*;

mod m20220702_125358_response_type;
mod m20220720_125359_client;
mod m20220720_125360_requests;
mod m20220801_214142_code;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220702_125358_response_type::Migration),
            Box::new(m20220720_125359_client::Migration),
            Box::new(m20220720_125360_requests::Migration),
            Box::new(m20220801_214142_code::Migration),
        ]
    }
}
