use anyhow::Result;
use rand::Rng;
use random_word::Lang;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};

use crate::now_time;
use crate::{models::project, DB};

#[derive(strum::Display, strum::EnumString, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum Language {
    Blank,
    Js,
    Rust,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    Deleted,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum CreatedBy {
    Playground,
    Blank,
}

/// random_name generates a random project name
pub fn random_name() -> String {
    // generate two word with 4-6 length
    // generate a 2-3 length number
    let rand_length = rand::thread_rng().gen_range(3..6);
    let word1 = random_word::gen_len(rand_length, Lang::En).unwrap();
    let rand_length = rand::thread_rng().gen_range(3..6);
    let word2 = random_word::gen_len(rand_length, Lang::En).unwrap();
    let number = rand::thread_rng().gen_range(10..100);
    format!("{}-{}-{}", word1, word2, number)
}

/// is_unique_name checks if the project name is unique
async fn is_unique_name(name: String) -> Result<bool> {
    let db = DB.get().unwrap();
    let project = project::Entity::find()
        .filter(project::Column::Name.eq(&name))
        .one(db)
        .await?;
    Ok(project.is_none())
}

async fn random_unique_name() -> Result<String> {
    let mut name = random_name();
    loop {
        if is_unique_name(name.clone()).await? {
            break;
        }
        name = random_name();
    }
    Ok(name)
}

/// get_by_id gets a project by id
pub async fn get_by_id(id: i32) -> Result<Option<project::Model>> {
    let db = DB.get().unwrap();
    let project = project::Entity::find()
        .filter(project::Column::Id.eq(id))
        .filter(project::Column::Status.eq(Status::Active.to_string()))
        .one(db)
        .await?;
    Ok(project)
}

/// get_by_name gets a project by name
pub async fn get_by_name(name: String, user_id: Option<i32>) -> Result<Option<project::Model>> {
    let db = DB.get().unwrap();
    let mut select = project::Entity::find()
        .filter(project::Column::Name.eq(name))
        .filter(project::Column::Status.eq(Status::Active.to_string()));
    if let Some(user_id) = user_id {
        select = select.filter(project::Column::UserId.eq(user_id));
    }
    let project = select.one(db).await?;
    Ok(project)
}

/// create_by_playground creates a new project by playground
pub async fn create_by_playground(
    user_id: i32,
    language: Language,
    description: String,
) -> Result<project::Model> {
    let name = random_unique_name().await?;
    let now = now_time();
    let project = project::Model {
        id: 0,
        user_id,
        name: name.clone(),
        language: language.to_string(),
        status: Status::Active.to_string(),
        uuid: uuid::Uuid::new_v4().to_string(),
        description: description.to_string(),
        domain: name.to_string(),
        created_by: CreatedBy::Playground.to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    let mut project_active_model: project::ActiveModel = project.into();
    project_active_model.id = ActiveValue::default();
    let db = DB.get().unwrap();
    let project = project_active_model.insert(db).await?;
    Ok(project)
}

/// create_blank creates a new blank project
pub async fn create_blank(user_id: i32) -> Result<project::Model> {
    let name = random_unique_name().await?;
    let now = now_time();
    let project = project::Model {
        id: 0,
        user_id,
        name: name.clone(),
        language: Language::Blank.to_string(),
        status: Status::Active.to_string(),
        uuid: uuid::Uuid::new_v4().to_string(),
        description: "This is a blank project".to_string(),
        domain: name.to_string(),
        created_by: CreatedBy::Blank.to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    let mut project_active_model: project::ActiveModel = project.into();
    project_active_model.id = ActiveValue::default();
    let db = DB.get().unwrap();
    let project = project_active_model.insert(db).await?;
    Ok(project)
}

/// list_by_user_id lists all projects by user id
pub async fn list_by_user_id(
    user_id: i32,
    search: Option<String>,
    limit: u64,
) -> Result<Vec<project::Model>> {
    let db = DB.get().unwrap();
    let mut select = project::Entity::find()
        .limit(limit)
        .filter(project::Column::UserId.eq(user_id))
        .filter(project::Column::Status.ne(Status::Deleted.to_string()))
        .order_by_desc(project::Column::UpdatedAt);
    if let Some(search) = search {
        let search = format!("%{}%", search);
        select = select.filter(project::Column::Name.like(search));
    }
    let projects = select.all(db).await.map_err(|e| anyhow::anyhow!(e))?;
    Ok(projects)
}

/// update_name updates the project name
pub async fn update_name(project_id: i32, name: String) -> Result<()> {
    if !is_unique_name(name.clone()).await? {
        return Err(anyhow::anyhow!("Project name is already taken"));
    }
    let db = DB.get().unwrap();
    let _ = project::Entity::update_many()
        .filter(project::Column::Id.eq(project_id))
        .col_expr(project::Column::Name, Expr::value(name.clone()))
        .col_expr(project::Column::Domain, Expr::value(name))
        .col_expr(project::Column::UpdatedAt, Expr::value(now_time()))
        .exec(db)
        .await?;
    Ok(())
}

/// delete deletes a project
pub async fn delete(user_id: i32, project_id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    // set project status to deleted
    project::Entity::update_many()
        .filter(project::Column::Id.eq(project_id))
        .filter(project::Column::UserId.eq(user_id))
        .col_expr(
            project::Column::Status,
            Expr::value(Status::Deleted.to_string()),
        )
        .col_expr(project::Column::DeletedAt, Expr::value(now_time()))
        .exec(db)
        .await?;

    // delete playground
    super::playground::delete_by_project(user_id, project_id).await?;

    Ok(())
}
