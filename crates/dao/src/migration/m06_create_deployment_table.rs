use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum Deployment {
    Table,
    Id,
    UserId,
    Domain,
    ProjectId,
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

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
                    .col(ColumnDef::new(Deployment::ProjectId).integer().not_null())
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
                    .name("idx-project-deployment-domain")
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
                    .name("idx-project-deployment-userid")
                    .table(Deployment::Table)
                    .col(Deployment::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-deployment-project-id")
                    .table(Deployment::Table)
                    .col(Deployment::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-deployment-status")
                    .table(Deployment::Table)
                    .col(Deployment::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-deployment-deploy-status")
                    .table(Deployment::Table)
                    .col(Deployment::DeployStatus)
                    .to_owned(),
            )
            .await?;

        debug!("Migration: m06_create_deployment_table has been applied");

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
