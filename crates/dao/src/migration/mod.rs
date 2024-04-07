use sea_orm_migration::{MigrationTrait, MigratorTrait};

mod m01_create_settings_table;
mod m02_create_user_table;
mod m03_create_project_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m01_create_settings_table::Migration),
            Box::new(m02_create_user_table::Migration),
            Box::new(m03_create_project_table::Migration),
        ]
    }
}
