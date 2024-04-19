use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum Worker {
    Table,
    Id,
    IP,
    IPv6,
    Region,
    Hostname,
    IPInfo,
    MachineInfo,
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
                    .col(
                        ColumnDef::new(Worker::IPv6)
                            .string_len(128)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Worker::Hostname).string_len(256).not_null())
                    .col(ColumnDef::new(Worker::Region).string_len(32).not_null())
                    .col(ColumnDef::new(Worker::IPInfo).text().not_null())
                    .col(ColumnDef::new(Worker::MachineInfo).text().not_null())
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

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-worker-ip")
                    .table(Worker::Table)
                    .col(Worker::IP)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-worker-ipv6")
                    .table(Worker::Table)
                    .col(Worker::IPv6)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-worker-region")
                    .table(Worker::Table)
                    .col(Worker::Region)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-worker-status")
                    .table(Worker::Table)
                    .col(Worker::Status)
                    .to_owned(),
            )
            .await?;
        debug!("Migration: m05_create_worker_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
