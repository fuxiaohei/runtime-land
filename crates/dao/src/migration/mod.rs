pub use sea_orm_migration::prelude::*;

pub struct Migrator;

mod m01_create_usertoken_table;
mod m02_create_user_table;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m01_create_usertoken_table::Migration),
            Box::new(m02_create_user_table::Migration),
        ]
    }
}
