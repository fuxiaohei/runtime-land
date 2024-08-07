//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "storage")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub owner_id: i32,
    pub project_id: i32,
    pub deploy_id: i32,
    pub task_id: String,
    pub path: String,
    pub file_size: i32,
    pub file_hash: String,
    pub file_target: String,
    pub status: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub deleted_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
