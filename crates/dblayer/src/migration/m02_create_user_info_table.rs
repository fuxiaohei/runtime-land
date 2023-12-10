use sea_orm_migration::prelude::*;

#[derive(Iden)]
enum UserInfo {
    Table,
    Id,
    DisplayName,
    Email,
    AvatarURL,
    Phone,
    Status,
    Role,
    Uuid,
    CreatedBy,
    OauthProvider,
    OauthUserId,
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
                        ColumnDef::new(UserInfo::Uuid)
                            .string_len(128)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(UserInfo::DisplayName)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserInfo::Email)
                            .string_len(256)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(UserInfo::AvatarURL)
                            .string_len(256)
                            .not_null(),
                    )
                    .col(ColumnDef::new(UserInfo::Phone).string_len(64))
                    .col(ColumnDef::new(UserInfo::Status).string_len(24).not_null())
                    .col(ColumnDef::new(UserInfo::Role).string_len(24).not_null())
                    .col(
                        ColumnDef::new(UserInfo::CreatedBy)
                            .string_len(24)
                            .not_null(),
                    )
                    .col(ColumnDef::new(UserInfo::OauthProvider).string_len(24))
                    .col(ColumnDef::new(UserInfo::OauthUserId).string_len(256))
                    .col(
                        ColumnDef::new(UserInfo::CreatedAt)
                            .timestamp()
                            .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserInfo::UpdatedAt)
                            .timestamp()
                            .extra(
                                "DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string(),
                            )
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
                    .name("idx-user-info-created-by")
                    .table(UserInfo::Table)
                    .col(UserInfo::CreatedBy)
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

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
