use sea_orm_migration::prelude::*;

mod m01_create_user_token_table;
mod m02_create_user_info_table;
mod m03_create_settings_table;
mod m04_create_project_table;
mod m05_create_project_deployment_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m01_create_user_token_table::Migration),
            Box::new(m02_create_user_info_table::Migration),
            Box::new(m03_create_settings_table::Migration),
            Box::new(m04_create_project_table::Migration),
            Box::new(m05_create_project_deployment_table::Migration),
        ]
    }
}
