use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum Playground {
    Table,
    Id,
    UserId,
    ProjectId,
    Uuid,
    Language,
    Source,
    Visiblity,
    Status,
    CreatedAt,
    DeletedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Playground::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Playground::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Playground::UserId).integer().not_null())
                    .col(ColumnDef::new(Playground::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Playground::Uuid).string_len(64).not_null())
                    .col(
                        ColumnDef::new(Playground::Language)
                            .string_len(24)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Playground::Source).text().not_null())
                    .col(ColumnDef::new(Playground::Status).string_len(12).not_null())
                    .col(
                        ColumnDef::new(Playground::Visiblity)
                            .string_len(12)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Playground::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(ColumnDef::new(Playground::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-playground-userid")
                    .table(Playground::Table)
                    .col(Playground::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-playground-projectid")
                    .table(Playground::Table)
                    .col(Playground::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-playground-status")
                    .table(Playground::Table)
                    .col(Playground::Status)
                    .to_owned(),
            )
            .await?;

        debug!("Migration: m05_create_playground_table has been applied");

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
