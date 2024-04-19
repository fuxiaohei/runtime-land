//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "deployment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub user_uuid: String,
    pub project_id: i32,
    pub project_uuid: String,
    pub task_id: String,
    pub domain: String,
    pub storage_path: String,
    pub storage_md5: String,
    pub storage_size: i32,
    pub spec: Json,
    pub deploy_status: String,
    pub deploy_message: String,
    pub status: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub deleted_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
