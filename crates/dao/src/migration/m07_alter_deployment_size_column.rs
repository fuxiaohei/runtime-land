use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::alter()
            .table(Deployment::Table)
            .add_column(
                ColumnDef::new(Deployment::StorageSize)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .add_column(
                ColumnDef::new(Deployment::StorageContentType)
                    .string_len(128)
                    .not_null()
                    .default(""),
            )
            .to_owned();
        manager.alter_table(table).await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(Iden)]
enum Deployment {
    Table,
    StorageSize,
    StorageContentType,
}
