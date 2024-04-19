use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum Deployment {
    Table,
    Id,
    UserId,
    UserUuid,
    ProjectId,
    ProjectUuid,
    TaskId,
    Domain,
    StoragePath,
    StorageSize,
    StorageMd5,
    Spec,
    DeployStatus,
    DeployMessage,
    Status,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn create_deployment_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Deployment::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Deployment::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(Deployment::UserId).integer().not_null())
                .col(
                    ColumnDef::new(Deployment::UserUuid)
                        .string_len(64)
                        .not_null(),
                )
                .col(ColumnDef::new(Deployment::ProjectId).integer().not_null())
                .col(
                    ColumnDef::new(Deployment::ProjectUuid)
                        .string_len(64)
                        .not_null(),
                )
                .col(ColumnDef::new(Deployment::TaskId).string_len(64).not_null())
                .col(
                    ColumnDef::new(Deployment::Domain)
                        .string_len(128)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Deployment::StoragePath)
                        .string_len(256)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Deployment::StorageMd5)
                        .string_len(256)
                        .not_null(),
                )
                .col(ColumnDef::new(Deployment::StorageSize).integer().not_null())
                .col(ColumnDef::new(Deployment::Spec).json().not_null())
                .col(
                    ColumnDef::new(Deployment::DeployStatus)
                        .string_len(12)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Deployment::DeployMessage)
                        .string_len(256)
                        .not_null(),
                )
                .col(ColumnDef::new(Deployment::Status).string_len(12).not_null())
                .col(
                    ColumnDef::new(Deployment::CreatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(
                    ColumnDef::new(Deployment::UpdatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(ColumnDef::new(Deployment::DeletedAt).timestamp().null())
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-domain")
                .table(Deployment::Table)
                .col(Deployment::Domain)
                .unique()
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-userid")
                .table(Deployment::Table)
                .col(Deployment::UserId)
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-user-uuid")
                .table(Deployment::Table)
                .col(Deployment::UserUuid)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-project-id")
                .table(Deployment::Table)
                .col(Deployment::ProjectId)
                .to_owned(),
        )
        .await?;
    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-project-uuid")
                .table(Deployment::Table)
                .col(Deployment::ProjectUuid)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-taskid")
                .table(Deployment::Table)
                .col(Deployment::TaskId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-status")
                .table(Deployment::Table)
                .col(Deployment::Status)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deployment-deploy-status")
                .table(Deployment::Table)
                .col(Deployment::DeployStatus)
                .to_owned(),
        )
        .await?;

    Ok(())
}

#[derive(Iden)]
enum DeploymentTask {
    Table,
    Id,
    IP,
    WorkerId,
    ProjectId,
    DeploymentId,
    TaskId,
    DeployStatus,
    DeployMessage,
    CreatedAt,
    UpdatedAt,
}

async fn create_deployment_task_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(DeploymentTask::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(DeploymentTask::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(DeploymentTask::IP).string_len(64).not_null())
                .col(
                    ColumnDef::new(DeploymentTask::WorkerId)
                        .integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DeploymentTask::ProjectId)
                        .integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DeploymentTask::DeploymentId)
                        .integer()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DeploymentTask::TaskId)
                        .string_len(128)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DeploymentTask::DeployStatus)
                        .string_len(12)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DeploymentTask::DeployMessage)
                        .string_len(256)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DeploymentTask::CreatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(
                    ColumnDef::new(DeploymentTask::UpdatedAt)
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
                .name("idx-deploment-task-deployid")
                .table(DeploymentTask::Table)
                .col(DeploymentTask::DeploymentId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deploment-task-taskid")
                .table(DeploymentTask::Table)
                .col(DeploymentTask::TaskId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deploment-task-projectid")
                .table(DeploymentTask::Table)
                .col(DeploymentTask::ProjectId)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-deploment-task-status")
                .table(DeploymentTask::Table)
                .col(DeploymentTask::DeployStatus)
                .to_owned(),
        )
        .await?;
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_deployment_table(manager).await?;
        create_deployment_task_table(manager).await?;
        debug!("Migration: m04_create_deployment_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
