use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Region::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Region::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Region::Name).string_len(256).not_null())
                    .col(
                        ColumnDef::new(Region::Key)
                            .string_len(256)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Region::Ip).string_len(128).not_null())
                    .col(ColumnDef::new(Region::City).string_len(128).not_null())
                    .col(ColumnDef::new(Region::Country).string_len(128).not_null())
                    .col(
                        ColumnDef::new(Region::Runtimes)
                            .integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Region::OwnerId)
                            .integer()
                            .default(0)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Region::Status).string_len(24).not_null())
                    .col(
                        ColumnDef::new(Region::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Region::UpdatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(ColumnDef::new(Region::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-region-name")
                    .table(Region::Table)
                    .col(Region::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-region-owner")
                    .table(Region::Table)
                    .col(Region::OwnerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-region-status")
                    .table(Region::Table)
                    .col(Region::Status)
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
enum Region {
    Table,
    Id,
    Name,
    Key,
    Ip,
    City,
    Country,
    Runtimes,
    OwnerId,
    Status,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
