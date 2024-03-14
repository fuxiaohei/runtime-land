use sea_orm_migration::prelude::*;
use tracing::debug;

#[derive(Iden)]
enum UserInfo {
    Table,
    Id,
    Name,
    NickName,
    Email,
    Gravatar,
    Status,
    Role,
    Password, // if user is created by origin-provider(clerk,others). the password is bcrypt(uuid+salt+origin-user-id)
    PasswordSalt, // random string
    OriginProvider,
    OriginUserId,
    OriginEmailId,
    LastLoginAt,
    CreatedAt,
    UpdatedAt,
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
                    .col(
                        ColumnDef::new(UserInfo::Gravatar)
                            .string_len(256)
                            .not_null(),
                    )
                    .col(ColumnDef::new(UserInfo::Status).string_len(12).not_null())
                    .col(ColumnDef::new(UserInfo::Role).string_len(12).not_null())
                    .col(
                        ColumnDef::new(UserInfo::OriginProvider)
                            .string_len(24)
                            .not_null(),
                    )
                    .col(ColumnDef::new(UserInfo::OriginUserId).string_len(256))
                    .col(ColumnDef::new(UserInfo::OriginEmailId).string_len(256))
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
                    .name("idx-user-info-origin")
                    .table(UserInfo::Table)
                    .col(UserInfo::OriginProvider)
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
        debug!("Migration: m02_create_user_info_table has been applied");
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
