use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum Storage {
    Table,
    Id,
    OwnerId,
    ProjectId,
    DeployId,
    TaskId,
    Path,
    FileSize,
    FileHash,
    FileTarget,
    Status,
    CreatedAt,
    UpdatedAt,
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
                    .table(Storage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Storage::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Storage::OwnerId).integer().not_null())
                    .col(ColumnDef::new(Storage::ProjectId).integer().not_null())
                    .col(ColumnDef::new(Storage::DeployId).integer().not_null())
                    .col(ColumnDef::new(Storage::TaskId).string_len(64).not_null())
                    .col(ColumnDef::new(Storage::Path).string_len(256).not_null())
                    .col(ColumnDef::new(Storage::FileSize).integer().not_null())
                    .col(ColumnDef::new(Storage::FileHash).string_len(128).not_null())
                    .col(
                        ColumnDef::new(Storage::FileTarget)
                            .string_len(256)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Storage::Status).string_len(12).not_null())
                    .col(
                        ColumnDef::new(Storage::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Storage::UpdatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(ColumnDef::new(Storage::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-storage-ownerid")
                    .table(Storage::Table)
                    .col(Storage::OwnerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-storage-project-id")
                    .table(Storage::Table)
                    .col(Storage::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-storage-deploy-id")
                    .table(Storage::Table)
                    .col(Storage::DeployId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-storage-task-id")
                    .table(Storage::Table)
                    .col(Storage::TaskId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-storage-status")
                    .table(Storage::Table)
                    .col(Storage::Status)
                    .to_owned(),
            )
            .await?;

        debug!("Migration: m05_create_storage_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
