use sea_orm_migration::prelude::*;

#[derive(Iden)]
enum RunnerInfo {
    Table,
    Id,
    Uuid,
    IP,
    Region,
    Country,
    Status,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(Iden)]
enum RunnerLabel {
    Table,
    Id,
    RunnerId,
    Label,
    Status,
    CreatedAt,
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
                    .table(RunnerInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RunnerInfo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RunnerInfo::Uuid).string_len(64).not_null())
                    .col(ColumnDef::new(RunnerInfo::IP).string_len(64).not_null())
                    .col(ColumnDef::new(RunnerInfo::Region).string_len(64).not_null())
                    .col(
                        ColumnDef::new(RunnerInfo::Country)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(RunnerInfo::Status).string_len(24).not_null())
                    .col(
                        ColumnDef::new(RunnerInfo::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RunnerInfo::UpdatedAt)
                            .timestamp()
                            .extra(
                                "DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string(),
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(RunnerInfo::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-runner-info-uuid")
                    .table(RunnerInfo::Table)
                    .col(RunnerInfo::Uuid)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-runner-info-status")
                    .table(RunnerInfo::Table)
                    .col(RunnerInfo::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RunnerLabel::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RunnerLabel::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RunnerLabel::RunnerId).integer().not_null())
                    .col(
                        ColumnDef::new(RunnerLabel::Status)
                            .string_len(24)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RunnerLabel::Label)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RunnerLabel::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(ColumnDef::new(RunnerLabel::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-runner-label-label")
                    .table(RunnerLabel::Table)
                    .col(RunnerLabel::Label)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-runner-label-status")
                    .table(RunnerLabel::Table)
                    .col(RunnerLabel::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
