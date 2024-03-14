use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum DeployTask {
    Table,
    Message,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(DeployTask::Table)
                    .add_column(
                        ColumnDef::new(DeployTask::Message)
                            .string_len(256)
                            .default("".to_string())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        debug!("Migration: m09_alter_deploy_task_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
