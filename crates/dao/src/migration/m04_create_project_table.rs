use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum Project {
    Table,
    Id,
    UserId,
    Name,
    Uuid,
    Language,
    Domain,
    Description,
    Status,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Metadata,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Project::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Project::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Project::UserId).integer().not_null())
                    .col(ColumnDef::new(Project::Name).string_len(128).not_null())
                    .col(ColumnDef::new(Project::Language).string_len(24).not_null())
                    .col(ColumnDef::new(Project::Status).string_len(24).not_null())
                    .col(ColumnDef::new(Project::Uuid).string_len(64).not_null())
                    .col(
                        ColumnDef::new(Project::Description)
                            .string_len(256)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Project::Domain).string_len(256).not_null())
                    .col(ColumnDef::new(Project::CreatedBy).string_len(24).not_null())
                    .col(
                        ColumnDef::new(Project::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Project::UpdatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(ColumnDef::new(Project::DeletedAt).timestamp().null())
                    .col(ColumnDef::new(Project::Metadata).text().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-info-userid-name")
                    .table(Project::Table)
                    .col(Project::Name)
                    .col(Project::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-info-status")
                    .table(Project::Table)
                    .col(Project::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-info-created-by")
                    .table(Project::Table)
                    .col(Project::CreatedBy)
                    .to_owned(),
            )
            .await?;

        debug!("Migration: m04_create_project_table has been applied");

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
