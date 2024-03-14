use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum Worker {
    Table,
    Id,
    IP,
    Hostname,
    IPInfo,
    MachineSize,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Worker::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Worker::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Worker::IP)
                            .string_len(64)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Worker::Hostname).string_len(256).not_null())
                    .col(ColumnDef::new(Worker::IPInfo).text().not_null())
                    .col(ColumnDef::new(Worker::MachineSize).text().not_null())
                    .col(ColumnDef::new(Worker::Status).string_len(12).not_null())
                    .col(
                        ColumnDef::new(Worker::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Worker::UpdatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        debug!("Migration: m07_create_worker_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
