use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Clients::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Clients::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Clients::Uuid).string().not_null())
                    .col(ColumnDef::new(Clients::Name).string().not_null())
                    .col(ColumnDef::new(Clients::Secret).string().not_null())
                    .col(ColumnDef::new(Clients::RedirectUri).string().not_null())
                    .col(ColumnDef::new(Clients::Scope).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Clients::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Clients {
    Table,
    Id,
    Uuid,
    Name,
    Secret,
    RedirectUri,
    Scope,
}
