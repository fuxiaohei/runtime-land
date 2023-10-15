use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RuntimeNode::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RuntimeNode::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RuntimeNode::Name).string_len(256).not_null())
                    .col(
                        ColumnDef::new(RuntimeNode::Key)
                            .string_len(256)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(RuntimeNode::Ip).string_len(128).not_null())
                    .col(ColumnDef::new(RuntimeNode::City).string_len(128).not_null())
                    .col(
                        ColumnDef::new(RuntimeNode::Region)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RuntimeNode::Country)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RuntimeNode::ConfHash)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RuntimeNode::Status)
                            .string_len(24)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RuntimeNode::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RuntimeNode::UpdatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(ColumnDef::new(RuntimeNode::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-runtime-name")
                    .table(RuntimeNode::Table)
                    .col(RuntimeNode::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-runtime-status")
                    .table(RuntimeNode::Table)
                    .col(RuntimeNode::Status)
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
enum RuntimeNode {
    Table,
    Id,
    Name,
    Key,
    Ip,
    City,
    Region,
    Country,
    ConfHash,
    Status,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
