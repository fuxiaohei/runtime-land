use sea_orm_migration::prelude::*;

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
                    .col(ColumnDef::new(UserInfo::Email).string_len(128).not_null())
                    .col(ColumnDef::new(UserInfo::Avatar).string_len(128).not_null())
                    .col(ColumnDef::new(UserInfo::Bio).string_len(2048).not_null())
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
                    .col(
                        ColumnDef::new(UserInfo::NickName)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(ColumnDef::new(UserInfo::Role).string_len(24).not_null())
                    .col(ColumnDef::new(UserInfo::Status).string_len(24).not_null())
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
                    .col(ColumnDef::new(UserInfo::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-user-info-email")
                    .table(UserInfo::Table)
                    .col(UserInfo::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-user-info-role")
                    .table(UserInfo::Table)
                    .col(UserInfo::Role)
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

#[derive(Iden)]
enum UserInfo {
    Table,
    Id,
    Email,
    NickName,
    Avatar,
    Bio,
    Password,
    PasswordSalt,
    Role,
    Status,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
