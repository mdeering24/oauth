use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, DbBackend, DeriveActiveEnum, EnumIter, Schema, Statement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let stmt = DbBackend::Postgres.build(
            &Schema::new(DbBackend::Postgres).create_enum_from_active_enum::<ResponseType>(),
        );
        manager.get_connection().execute(stmt).await.map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let stmt = Statement::from_string(
            manager.get_database_backend(),
            "DROP TYPE response_type".to_string(),
        );
        manager.get_connection().execute(stmt).await.map(|_| ())
    }
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "response_type")]
pub enum ResponseType {
    #[sea_orm(string_value = "code")]
    Code,
    #[sea_orm(string_value = "other")]
    Other,
}
