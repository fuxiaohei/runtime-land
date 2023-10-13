pub use sea_orm_migration::prelude::*;

pub struct Migrator;

mod m01_create_usertoken_table;
mod m02_create_user_table;
mod m03_create_regions_table;
mod m04_create_project_table;
mod m05_create_deployment_table;
mod m06_create_settings_table;
mod m07_alter_deployment_size_column;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m01_create_usertoken_table::Migration),
            Box::new(m02_create_user_table::Migration),
            Box::new(m03_create_regions_table::Migration),
            Box::new(m04_create_project_table::Migration),
            Box::new(m05_create_deployment_table::Migration),
            Box::new(m06_create_settings_table::Migration),
            Box::new(m07_alter_deployment_size_column::Migration),
        ]
    }
}
