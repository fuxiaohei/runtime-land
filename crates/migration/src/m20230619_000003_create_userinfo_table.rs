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
                        ColumnDef::new(UserInfo::DisplayName)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(ColumnDef::new(UserInfo::Role).integer().not_null())
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
    Password,
    PasswordSalt,
    CreatedAt,
    UpdatedAt,
    DisplayName,
    Role,
}
