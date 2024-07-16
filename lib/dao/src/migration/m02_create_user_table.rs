use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum UserToken {
    Table,
    Id,
    OwnerId,
    Name,
    Value,
    Status,
    Usage,
    CreatedAt,
    LatestUsedAt,
    ExpiredAt,
    DeletedAt,
}

#[derive(Iden)]
enum UserInfo {
    Table,
    Id,
    Uuid,
    Name,
    NickName,
    Email,
    Avatar,
    Status,
    Role,
    Password, // if user is created by Oauth-provider(clerk,others). the password is bcrypt(uuid+salt+origin-user-id)
    PasswordSalt, // random string
    OauthProvider,
    OauthUserId,
    OauthEmailId,
    LastLoginAt,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

async fn create_user_token_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
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
                .col(ColumnDef::new(UserToken::OwnerId).integer().not_null())
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
                    ColumnDef::new(UserToken::LatestUsedAt)
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
                .col(UserToken::OwnerId)
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
    Ok(())
}

async fn create_user_info_table(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(UserInfo::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(UserInfo::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(UserInfo::Uuid)
                        .string_len(128)
                        .unique_key()
                        .not_null(),
                )
                .col(
                    ColumnDef::new(UserInfo::Password)
                        .string_len(128)
                        .not_null(),
                )
                .col(
                    ColumnDef::new(UserInfo::PasswordSalt)
                        .string_len(64)
                        .not_null(),
                )
                .col(ColumnDef::new(UserInfo::Name).string_len(64).not_null())
                .col(ColumnDef::new(UserInfo::NickName).string_len(64).not_null())
                .col(
                    ColumnDef::new(UserInfo::Email)
                        .string_len(256)
                        .not_null()
                        .unique_key(),
                )
                .col(ColumnDef::new(UserInfo::Avatar).string_len(256).not_null())
                .col(ColumnDef::new(UserInfo::Status).string_len(12).not_null())
                .col(ColumnDef::new(UserInfo::Role).string_len(12).not_null())
                .col(
                    ColumnDef::new(UserInfo::OauthProvider)
                        .string_len(24)
                        .not_null(),
                )
                .col(ColumnDef::new(UserInfo::OauthUserId).string_len(256))
                .col(ColumnDef::new(UserInfo::OauthEmailId).string_len(256))
                .col(
                    ColumnDef::new(UserInfo::CreatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(
                    ColumnDef::new(UserInfo::UpdatedAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(
                    ColumnDef::new(UserInfo::LastLoginAt)
                        .timestamp()
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .not_null(),
                )
                .col(ColumnDef::new(UserInfo::DeletedAt).timestamp().null())
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-user-info-oauth")
                .table(UserInfo::Table)
                .col(UserInfo::OauthProvider)
                .to_owned(),
        )
        .await?;

    manager
        .create_index(
            Index::create()
                .if_not_exists()
                .name("idx-user-info-status")
                .table(UserInfo::Table)
                .col(UserInfo::Status)
                .to_owned(),
        )
        .await?;
    Ok(())
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_user_token_table(manager).await?;
        create_user_info_table(manager).await?;
        debug!("Migration: m02_create_user_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
