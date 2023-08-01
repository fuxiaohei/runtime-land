use sea_orm_migration::prelude::*;

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
                    .col(ColumnDef::new(Deployment::ProjectId).integer().not_null())
                    .col(
                        ColumnDef::new(Deployment::ProjectUuid)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Deployment::Domain)
                            .string_len(128)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Deployment::ProdDomain)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Deployment::Uuid)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Deployment::StoragePath)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Deployment::OwnerId).integer().not_null())
                    .col(ColumnDef::new(Deployment::Status).string_len(24).not_null())
                    .col(
                        ColumnDef::new(Deployment::DeployStatus)
                            .string_len(24)
                            .not_null(),
                    )
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
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-deployment-owner-id")
                    .table(Deployment::Table)
                    .col(Deployment::OwnerId)
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
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(Iden)]
enum Deployment {
    Table,
    Id,
    OwnerId,
    ProjectId,
    ProjectUuid,
    Domain,
    ProdDomain,
    Uuid,
    StoragePath,
    DeployStatus,
    Status,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
