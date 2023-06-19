pub use sea_orm_migration::prelude::*;

mod m20230619_000001_create_projectdeployment_table;
mod m20230619_000002_create_projectinfo_table;
mod m20230619_000003_create_userinfo_table;
mod m20230619_000004_create_usertoken_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230619_000001_create_projectdeployment_table::Migration),
            Box::new(m20230619_000002_create_projectinfo_table::Migration),
            Box::new(m20230619_000003_create_userinfo_table::Migration),
            Box::new(m20230619_000004_create_usertoken_table::Migration),
        ]
    }
}
