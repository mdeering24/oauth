use super::m20220720_125360_requests::Requests;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Code::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Code::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Code::Uuid).string().not_null())
                    .col(ColumnDef::new(Code::RequestId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-code-request_id")
                            .from(Code::Table, Code::RequestId)
                            .to(Requests::Table, Requests::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Code::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Code {
    #[iden = "codes"]
    Table,
    Id,
    #[iden = "request_id"]
    RequestId,
    Uuid,
}
