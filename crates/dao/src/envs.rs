use crate::{db::DB, models::project_envs, now_time};
use anyhow::Result;
use once_cell::sync::Lazy;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum EnvStatus {
    Active,
    Deleted,
}

#[derive(serde::Deserialize, Debug)]
pub struct EnvsParams {
    pub name: Vec<String>,
    pub value: Vec<String>,
    pub op: Vec<String>,
}

impl EnvsParams {
    fn items(self) -> Vec<EnvItem> {
        let mut items = vec![];
        for i in 0..self.name.len() {
            items.push(EnvItem {
                name: self.name[i].clone(),
                value: self.value[i].clone(),
                op: self.op[i].clone(),
            });
        }
        items
    }
}

#[derive(Debug)]
struct EnvItem {
    name: String,
    value: String,
    op: String,
}

/// update_envs updates the envs of a project
pub async fn update_envs(params: EnvsParams, project_id: i32, project_uuid: String) -> Result<()> {
    let items = params.items();
    for item in items {
        if item.name.is_empty() || item.op.is_empty() {
            continue;
        }
        if item.op == "delete" {
            delete_env(project_id, &item.name).await?;
        } else {
            set_env(project_id, project_uuid.clone(), &item.name, &item.value).await?;
        }
    }
    Ok(())
}

async fn delete_env(project_id: i32, key: &str) -> Result<()> {
    let db = DB.get().unwrap();
    project_envs::Entity::update_many()
        .filter(project_envs::Column::ProjectId.eq(project_id))
        .filter(project_envs::Column::EnvKey.eq(key))
        .col_expr(
            project_envs::Column::Status,
            Expr::value(EnvStatus::Deleted.to_string()),
        )
        .exec(db)
        .await?;
    Ok(())
}

async fn set_env(project_id: i32, project_uuid: String, key: &str, value: &str) -> Result<()> {
    let db = DB.get().unwrap();
    let now = now_time();
    let item = project_envs::Entity::find()
        .filter(project_envs::Column::ProjectId.eq(project_id))
        .filter(project_envs::Column::EnvKey.eq(key))
        .one(db)
        .await?;
    if item.is_none() {
        let salt = land_common::encoding::rand_string(16);
        let encrypt_value = land_common::encoding::encrypt_text(value, &salt)?;
        let item = project_envs::ActiveModel {
            project_id: Set(project_id),
            project_uuid: Set(project_uuid),
            env_key: Set(key.to_string()),
            env_value: Set(encrypt_value),
            env_salt: Set(salt),
            created_at: Set(now),
            updated_at: Set(now),
            status: Set(EnvStatus::Active.to_string()),
            ..Default::default()
        };
        item.insert(db).await?;
    } else {
        let item = item.unwrap();
        let salt = item.env_salt.clone();
        let encrypt_value = land_common::encoding::encrypt_text(value, &salt)?;
        let mut item = item.into_active_model();
        item.env_value = Set(encrypt_value);
        item.updated_at = Set(now);
        item.save(db).await?;
    }
    Ok(())
}

/// list_envs_by_project lists the envs of a project
pub async fn list_envs_by_project(project_id: i32) -> Result<Vec<project_envs::Model>> {
    let db = DB.get().unwrap();
    let items = project_envs::Entity::find()
        .filter(project_envs::Column::ProjectId.eq(project_id))
        .filter(project_envs::Column::Status.eq(EnvStatus::Active.to_string()))
        .order_by_desc(project_envs::Column::UpdatedAt)
        .all(db)
        .await?;
    Ok(items)
}

/// list_envs lists all envs
pub async fn list_envs() -> Result<Vec<project_envs::Model>> {
    let db = DB.get().unwrap();
    let items = project_envs::Entity::find()
        .filter(project_envs::Column::Status.eq(EnvStatus::Active.to_string()))
        .order_by_desc(project_envs::Column::UpdatedAt)
        .all(db)
        .await?;
    Ok(items)
}

/// EnvWorkerItem is the environment variables for a project item
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EnvWorkerItem {
    pub key: String,
    pub value: String,
    pub salt: String,
}

/// EnvWorkerTotal is the environment variables for all projects
pub type EnvWorkerTotal = HashMap<String, Vec<EnvWorkerItem>>;

/// EnvWorkerLocal is local data to cache it
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EnvWorkerLocal {
    pub md5: String,
    pub envs: EnvWorkerTotal,
}

impl EnvWorkerLocal {
    pub fn to_raw(&self) -> EnvRawData {
        let mut data = HashMap::new();
        for (k, v) in self.envs.iter() {
            let mut map = HashMap::new();
            for item in v {
                let decrypted =
                    land_common::encoding::decrypt_text(&item.value, &item.salt).unwrap();
                map.insert(item.key.clone(), decrypted);
            }
            data.insert(k.clone(), map);
        }
        data
    }
}

/// ENV_WORKER_LOCAL is the local cache for environment variables
pub static ENV_WORKER_LOCAL: Lazy<Mutex<EnvWorkerLocal>> = Lazy::new(|| {
    Mutex::new(EnvWorkerLocal {
        md5: "".to_string(),
        envs: HashMap::new(),
    })
});

/// EnvRawMap is the raw map of environment variables for a project
pub type EnvRawMap = HashMap<String, String>;
/// EnvRawData is the raw data of environment variables for all projects
pub type EnvRawData = HashMap<String, EnvRawMap>;
