use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Project::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Project::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Project::Name)
                            .string_len(128)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Project::Language).string_len(24).not_null())
                    .col(
                        ColumnDef::new(Project::Uuid)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Project::OwnerId).integer().not_null())
                    .col(
                        ColumnDef::new(Project::ProdDeployId)
                            .integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Project::Status).string_len(24).not_null())
                    .col(
                        ColumnDef::new(Project::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Project::UpdatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(ColumnDef::new(Project::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-name")
                    .table(Project::Table)
                    .col(Project::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-owner-id")
                    .table(Project::Table)
                    .col(Project::OwnerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-project-status")
                    .table(Project::Table)
                    .col(Project::Status)
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
enum Project {
    Table,
    Id,
    Name,
    Language,
    Uuid,
    OwnerId,
    ProdDeployId,
    Status,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
