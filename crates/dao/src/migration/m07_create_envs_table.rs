use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum ProjectEnvs {
    Table,
    Id,
    ProjectId,
    ProjectUuid,
    EnvKey,
    EnvValue,
    EnvSalt,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // create table
        manager
            .create_table(
                Table::create()
                    .table(ProjectEnvs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectEnvs::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProjectEnvs::ProjectId).integer().not_null())
                    .col(ColumnDef::new(ProjectEnvs::ProjectUuid).string_len(64).not_null())
                    .col(
                        ColumnDef::new(ProjectEnvs::EnvKey)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectEnvs::EnvValue).text().not_null())
                    .col(
                        ColumnDef::new(ProjectEnvs::EnvSalt)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectEnvs::Status)
                            .string_len(12)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectEnvs::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectEnvs::UpdatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-env-project-id")
                    .table(ProjectEnvs::Table)
                    .col(ProjectEnvs::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-env-key")
                    .table(ProjectEnvs::Table)
                    .col(ProjectEnvs::EnvKey)
                    .to_owned(),
            )
            .await?;

        debug!("Migration: m07_create_envs_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
