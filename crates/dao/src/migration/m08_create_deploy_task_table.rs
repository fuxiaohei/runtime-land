use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum DeployTask {
    Table,
    Id,
    IP,
    WorkerId,
    ProjectId,
    DeploymentId,
    TaskId,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Deployment {
    Table,
    TaskId,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DeployTask::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DeployTask::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DeployTask::IP).string_len(64).not_null())
                    .col(ColumnDef::new(DeployTask::WorkerId).integer().not_null())
                    .col(ColumnDef::new(DeployTask::ProjectId).integer().not_null())
                    .col(
                        ColumnDef::new(DeployTask::DeploymentId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeployTask::TaskId)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(ColumnDef::new(DeployTask::Status).string_len(12).not_null())
                    .col(
                        ColumnDef::new(DeployTask::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DeployTask::UpdatedAt)
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
                    .name("idx-deploytask-deployid")
                    .table(DeployTask::Table)
                    .col(DeployTask::DeploymentId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-deploytask-taskid")
                    .table(DeployTask::Table)
                    .col(DeployTask::TaskId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-deploytask-projectid")
                    .table(DeployTask::Table)
                    .col(DeployTask::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-deploytask-status")
                    .table(DeployTask::Table)
                    .col(DeployTask::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Deployment::Table)
                    .add_column(
                        ColumnDef::new(Deployment::TaskId)
                            .string_len(128)
                            .default("".to_string())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        debug!("Migration: m08_create_deploy_task_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
