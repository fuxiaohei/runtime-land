use sea_orm_migration::prelude::*;

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
                    .col(
                        ColumnDef::new(ProjectInfo::Name)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(ProjectInfo::Language)
                            .string_len(24)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProjectInfo::Uuid)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(ProjectInfo::OwnerId).integer().not_null())
                    .col(
                        ColumnDef::new(ProjectInfo::ProdDeployId)
                            .integer()
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
                    .name("idx-project-info-name")
                    .table(ProjectInfo::Table)
                    .col(ProjectInfo::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-info-owner-id")
                    .table(ProjectInfo::Table)
                    .col(ProjectInfo::OwnerId)
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
enum ProjectInfo {
    Table,
    Id,
    Name,
    Language,
    Uuid,
    CreatedAt,
    UpdatedAt,
    OwnerId,
    ProdDeployId,
}
