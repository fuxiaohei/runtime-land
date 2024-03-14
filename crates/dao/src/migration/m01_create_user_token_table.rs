use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum UserToken {
    Table,
    Id,
    UserId,
    Name,
    Value,
    Status,
    Usage,
    CreatedAt,
    UpdatedAt,
    ExpiredAt,
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
                    .table(UserToken::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserToken::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserToken::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(UserToken::Value)
                            .string_len(128)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(UserToken::Name).string_len(64).not_null())
                    .col(ColumnDef::new(UserToken::Status).string_len(12).not_null())
                    .col(ColumnDef::new(UserToken::Usage).string_len(12).not_null())
                    .col(
                        ColumnDef::new(UserToken::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserToken::UpdatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(ColumnDef::new(UserToken::ExpiredAt).timestamp().null())
                    .col(ColumnDef::new(UserToken::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-user-token-name")
                    .table(UserToken::Table)
                    .col(UserToken::Name)
                    .col(UserToken::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-user-token-value")
                    .table(UserToken::Table)
                    .col(UserToken::Value)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-user-token-usage")
                    .table(UserToken::Table)
                    .col(UserToken::Usage)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-user-token-status")
                    .table(UserToken::Table)
                    .col(UserToken::Status)
                    .to_owned(),
            )
            .await?;
        debug!("Migration: m01_create_user_token_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
