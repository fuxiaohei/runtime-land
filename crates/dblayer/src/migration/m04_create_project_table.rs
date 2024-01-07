use sea_orm_migration::prelude::*;

#[derive(Iden)]
enum ProjectInfo {
    Table,
    Id,
    OwnerId,
    Name,
    Language,
    Uuid,
    Description,
    Status,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Metadata,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProjectInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProjectInfo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ProjectInfo::OwnerId).integer().not_null())
                    .col(ColumnDef::new(ProjectInfo::Name).string_len(128).not_null())
                    .col(
                        ColumnDef::new(ProjectInfo::Language)
                            .string_len(24)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectInfo::Status)
                            .string_len(24)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectInfo::Uuid).string_len(64).not_null())
                    .col(
                        ColumnDef::new(ProjectInfo::Description)
                            .string_len(256)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectInfo::CreatedBy)
                            .string_len(24)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectInfo::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectInfo::UpdatedAt)
                            .timestamp()
                            .extra(
                                "DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string(),
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProjectInfo::DeletedAt).timestamp().null())
                    .col(ColumnDef::new(ProjectInfo::Metadata).text().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-info-ownerid-name")
                    .table(ProjectInfo::Table)
                    .col(ProjectInfo::Name)
                    .col(ProjectInfo::OwnerId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-info-status")
                    .table(ProjectInfo::Table)
                    .col(ProjectInfo::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-info-created-by")
                    .table(ProjectInfo::Table)
                    .col(ProjectInfo::CreatedBy)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
