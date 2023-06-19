use sea_orm_migration::prelude::*;

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
                    .col(ColumnDef::new(UserToken::OwnerId).integer().not_null())
                    .col(
                        ColumnDef::new(UserToken::Token)
                            .string_len(128)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(UserToken::Uuid)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(UserToken::Name).string_len(64).not_null())
                    .col(ColumnDef::new(UserToken::Origin).string_len(24).not_null())
                    .col(ColumnDef::new(UserToken::ExpiredAt).integer().not_null())
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
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-user-token-uuid")
                    .table(UserToken::Table)
                    .col(UserToken::Uuid)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-user-token-token")
                    .table(UserToken::Table)
                    .col(UserToken::Token)
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
enum UserToken {
    Table,
    Id,
    OwnerId,
    Token,
    Uuid,
    Name,
    CreatedAt,
    UpdatedAt,
    Origin,
    ExpiredAt,
}
