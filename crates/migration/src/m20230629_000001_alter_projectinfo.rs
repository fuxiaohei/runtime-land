use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ProjectInfo::Table)
                    .add_column(
                        ColumnDef::new(ProjectInfo::ProjectStatus)
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
                    .name("idx-project-info-status")
                    .table(ProjectInfo::Table)
                    .col(ProjectInfo::ProjectStatus)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-project-info-status")
                    .table(ProjectInfo::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ProjectInfo::Table)
                    .drop_column(ProjectInfo::ProjectStatus)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum ProjectInfo {
    Table,
    ProjectStatus,
}
