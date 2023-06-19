use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectDeployment::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectDeployment::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectDeployment::OwnerId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectDeployment::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectDeployment::Domain)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectDeployment::ProdDomain)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectDeployment::Uuid)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectDeployment::StoragePath)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectDeployment::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectDeployment::UpdatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectDeployment::ProdStatus)
                            .integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectDeployment::DeployStatus)
                            .integer()
                            .default(0)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-deployment-owner")
                    .table(ProjectDeployment::Table)
                    .col(ProjectDeployment::OwnerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-deployment-domain")
                    .table(ProjectDeployment::Table)
                    .col(ProjectDeployment::Domain)
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
enum ProjectDeployment {
    Table,
    Id,
    OwnerId,
    ProjectId,
    Domain,
    ProdDomain,
    Uuid,
    StoragePath,
    CreatedAt,
    UpdatedAt,
    ProdStatus,
    DeployStatus,
}
