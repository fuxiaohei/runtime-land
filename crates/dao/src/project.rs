use crate::{model::project, DB};
use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Pending,  // pending for deployment
    Active,   // some deployments are active
    InActive, // close project manually
    Deleted,  // deleted
}

/// create creates a project
pub async fn create(
    name: Option<String>,
    prefix: Option<String>,
    language: String,
    owner_id: i32,
) -> Result<project::Model> {
    if name.is_none() && prefix.is_none() {
        return Err(anyhow::anyhow!("name or prefix must be given"));
    }
    let project_name = if let Some(name) = name {
        name
    } else {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(3)
            .map(char::from)
            .collect();
        format!(
            "{}-{}-{}",
            prefix.unwrap(),
            random_word::gen(random_word::Lang::En),
            rand_string.to_lowercase(),
        )
    };
    let now = chrono::Utc::now();
    let uuid = uuid::Uuid::new_v4().to_string();
    let project = project::Model {
        id: 0,
        name: project_name,
        uuid,
        owner_id,
        language,
        prod_deploy_id: 0,
        status: Status::Pending.to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
    };
    let active_model: project::ActiveModel = project.into();
    let db = DB.get().unwrap();
    let project = active_model.insert(db).await?;
    Ok(project)
}

/// find_by_name finds a project by name
pub async fn find_by_name(name: String, owner_id: i32) -> Result<Option<project::Model>> {
    let db = DB.get().unwrap();
    let project = project::Entity::find()
        .filter(project::Column::Name.eq(name))
        .filter(project::Column::OwnerId.eq(owner_id))
        .one(db)
        .await?;
    Ok(project)
}

/// set_active sets a project to active
pub async fn set_active(project_id: i32) -> Result<project::Model> {
    let db = DB.get().unwrap();
    let project = project::Entity::find_by_id(project_id)
        .one(db)
        .await?
        .unwrap();
    let mut active_model: project::ActiveModel = project.into();
    active_model.updated_at = Set(chrono::Utc::now());
    active_model.status = Set(Status::Active.to_string());
    let project = active_model.update(db).await?;
    Ok(project)
}
