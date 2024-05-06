use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum ProjectTraffic {
    Table,
    Id,
    ProjectId,
    TimeAt,
    TrafficKey,
    Value,
    CreatedAt,
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
                    .table(ProjectTraffic::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectTraffic::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectTraffic::ProjectId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectTraffic::TimeAt)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectTraffic::TrafficKey)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectTraffic::Value).integer().not_null())
                    .col(
                        ColumnDef::new(ProjectTraffic::CreatedAt)
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
                    .name("idx-project-traffic-project-id")
                    .table(ProjectTraffic::Table)
                    .col(ProjectTraffic::ProjectId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-traffic-key")
                    .table(ProjectTraffic::Table)
                    .col(ProjectTraffic::TrafficKey)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-traffic-timeat")
                    .table(ProjectTraffic::Table)
                    .col(ProjectTraffic::TimeAt)
                    .to_owned(),
            )
            .await?;

        debug!("Migration: m06_create_traffic_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
