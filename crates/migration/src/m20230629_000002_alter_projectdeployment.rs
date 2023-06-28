use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ProjectDeployment::Table)
                    .modify_column(
                        ColumnDef::new(ProjectDeployment::DeployStatus)
                            .string_len(12)
                            .not_null()
                            .default("normal"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-deployment-status")
                    .table(ProjectDeployment::Table)
                    .col(ProjectDeployment::DeployStatus)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-project-deployment-status")
                    .table(ProjectDeployment::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum ProjectDeployment {
    Table,
    DeployStatus,
}
