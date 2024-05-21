use crate::{db::DB, models::project_envs, now_time};
use anyhow::Result;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set,
};

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
pub async fn update_envs(params: EnvsParams, project_id: i32) -> Result<()> {
    let items = params.items();
    for item in items {
        if item.name.is_empty() || item.op.is_empty() {
            continue;
        }
        if item.op == "delete" {
            delete_env(project_id, &item.name).await?;
        } else {
            set_env(project_id, &item.name, &item.value).await?;
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

async fn set_env(project_id: i32, key: &str, value: &str) -> Result<()> {
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
            env_key: Set(key.to_string()),
            env_value: Set(encrypt_value),
            env_salt: Set(salt),
            created_at: Set(now),
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
        item.save(db).await?;
    }
    Ok(())
}

/// list_envs lists the envs of a project
pub async fn list_envs(project_id: i32) -> Result<Vec<project_envs::Model>> {
    let db = DB.get().unwrap();
    let items = project_envs::Entity::find()
        .filter(project_envs::Column::ProjectId.eq(project_id))
        .filter(project_envs::Column::Status.eq(EnvStatus::Active.to_string()))
        .order_by_desc(project_envs::Column::CreatedAt)
        .all(db)
        .await?;
    Ok(items)
}
