use crate::deployment::DeploymentStatus;
use crate::models::deployment;
use crate::{
    db::DB,
    deployment::DeployStatus,
    models::{playground, project},
    now_time,
};
use anyhow::{anyhow, Result};
use rand::Rng;
use random_word::Lang;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, ItemsAndPagesNumber,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use std::str::FromStr;
use tracing::info;

#[derive(strum::Display, strum::EnumString, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum Language {
    JavaScript,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Disabled, // set disabled by user
    Deleted,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum ProjectCreatedBy {
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

/// random_unique_name generates a random unique project name
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

/// create_project_with_playground creates a new project with a playground
pub async fn create_project_with_playground(
    user_id: i32,
    language: Language,
    description: String,
    source: String,
) -> Result<String> {
    let p = create_project_by_playground(user_id, language.clone(), description).await?;
    let py = create_playground(user_id, p.id, language, source, false).await?;
    info!("Playground created: {}, py: {}", p.name, py.id);
    Ok(p.name)
}

/// create_project_by_playground creates from a playground
async fn create_project_by_playground(
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
        status: ProjectStatus::Active.to_string(),
        deploy_status: DeployStatus::Compiling.to_string(),
        uuid: uuid::Uuid::new_v4().to_string(),
        description: description.to_string(),
        dev_domain: String::new(),
        prod_domain: name.to_string(),
        created_by: ProjectCreatedBy::Playground.to_string(),
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
        .filter(project::Column::Status.ne(ProjectStatus::Deleted.to_string()))
        .order_by_desc(project::Column::UpdatedAt);
    if let Some(search) = search {
        let search = format!("%{}%", search);
        select = select.filter(project::Column::Name.like(search));
    }
    let projects = select.all(db).await.map_err(|e| anyhow::anyhow!(e))?;
    Ok(projects)
}

/// list_paginate lists all projects with pagination
pub async fn list_paginate(
    current: u64,
    page_size: u64,
) -> Result<(Vec<project::Model>, ItemsAndPagesNumber)> {
    let db = DB.get().unwrap();
    let pager = project::Entity::find()
        .filter(project::Column::Status.ne(ProjectStatus::Deleted.to_string()))
        .order_by_desc(project::Column::Id)
        .paginate(db, page_size);
    // current page data
    let projects = pager.fetch_page(current - 1).await?;
    let pages = pager.num_items_and_pages().await?;
    Ok((projects, pages))
}

/// get_by_name gets a project by name
pub async fn get_by_name(name: String, user_id: Option<i32>) -> Result<Option<project::Model>> {
    let db = DB.get().unwrap();
    let mut select = project::Entity::find()
        .filter(project::Column::Name.eq(name))
        .filter(project::Column::Status.ne(ProjectStatus::Deleted.to_string()));
    if let Some(user_id) = user_id {
        select = select.filter(project::Column::UserId.eq(user_id));
    }
    let project = select.one(db).await?;
    Ok(project)
}

/// get_by_id gets a project by id
pub async fn get_by_id(id: i32, user_id: Option<i32>) -> Result<Option<project::Model>> {
    let db = DB.get().unwrap();
    let mut select = project::Entity::find()
        .filter(project::Column::Id.eq(id))
        .filter(project::Column::Status.ne(ProjectStatus::Deleted.to_string()));
    if let Some(user_id) = user_id {
        select = select.filter(project::Column::UserId.eq(user_id));
    }
    let project = select.one(db).await?;
    Ok(project)
}

/// delete deletes a project
pub async fn delete(id: i32, name: String) -> Result<()> {
    let db = DB.get().unwrap();
    let now_timestamp = now_time().and_utc().timestamp();
    let rename = format!("{}_deleted_{}", name, now_timestamp);
    project::Entity::update_many()
        .col_expr(
            project::Column::Status,
            Expr::value(ProjectStatus::Deleted.to_string()),
        )
        .col_expr(project::Column::Name, Expr::value(rename))
        .filter(project::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(())
}

/// update_name updates a project name
pub async fn update_name(id: i32, name: String, desc: String) -> Result<()> {
    let db = DB.get().unwrap();
    project::Entity::update_many()
        .col_expr(project::Column::Name, Expr::value(name.clone()))
        .col_expr(project::Column::ProdDomain, Expr::value(name))
        .col_expr(project::Column::Description, Expr::value(desc))
        .filter(project::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(())
}

/// get_project_by_name_with_playground gets a project by name with playground
pub async fn get_project_by_name_with_playground(
    name: String,
    user_id: i32,
) -> Result<(project::Model, Option<playground::Model>)> {
    let p = get_by_name(name, Some(user_id)).await?;
    if p.is_none() {
        return Err(anyhow!("Project not found"));
    }
    let p = p.unwrap();
    let mut py: Option<playground::Model> = None;
    if p.created_by == ProjectCreatedBy::Playground.to_string() {
        py = get_playground_by_project(user_id, p.id).await?;
    }
    Ok((p, py))
}

/// set_disabled sets a project as disabled
pub async fn set_disabled(project_id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    deployment::Entity::update_many()
        .col_expr(
            deployment::Column::Status,
            Expr::value(DeploymentStatus::Disabled.to_string()),
        )
        .col_expr(deployment::Column::UpdatedAt, Expr::value(now_time()))
        .filter(deployment::Column::ProjectId.eq(project_id))
        .filter(deployment::Column::Status.eq(DeploymentStatus::Active.to_string()))
        .exec(db)
        .await?;
    project::Entity::update_many()
        .col_expr(
            project::Column::Status,
            Expr::value(ProjectStatus::Disabled.to_string()),
        )
        .col_expr(project::Column::UpdatedAt, Expr::value(now_time()))
        .filter(project::Column::Id.eq(project_id))
        .exec(db)
        .await?;
    Ok(())
}

/// set_enabled sets a project as enabled
pub async fn set_enabled(project_id: i32) -> Result<()> {
    let db = DB.get().unwrap();
    project::Entity::update_many()
        .col_expr(
            project::Column::Status,
            Expr::value(ProjectStatus::Active.to_string()),
        )
        .col_expr(project::Column::UpdatedAt, Expr::value(now_time()))
        .filter(project::Column::Id.eq(project_id))
        .exec(db)
        .await?;

    // create new deployment to deploy again
    let _ = crate::deployment::create_by_project(project_id).await?;
    Ok(())
}

pub type PlaygroundStatus = ProjectStatus;

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum PlaygroundVisibility {
    Public,
    Private,
}

/// create_playground creates a new playground
async fn create_playground(
    user_id: i32,
    project_id: i32,
    language: Language,
    source: String,
    visible: bool,
) -> Result<playground::Model> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let p = playground::Model {
        id: 0,
        user_id,
        project_id,
        uuid,
        language: language.to_string(),
        source,
        version: String::new(),
        status: PlaygroundStatus::Active.to_string(),
        visiblity: if visible {
            PlaygroundVisibility::Public.to_string()
        } else {
            PlaygroundVisibility::Private.to_string()
        },
        created_at: now_time(),
        deleted_at: None,
    };
    let mut active_model = p.into_active_model();
    active_model.id = Default::default();
    let p = active_model.insert(DB.get().unwrap()).await?;
    Ok(p)
}

/// get_playground_by_project gets a playground by project
pub async fn get_playground_by_project(
    user_id: i32,
    project_id: i32,
) -> Result<Option<playground::Model>> {
    let db = DB.get().unwrap();
    let p = playground::Entity::find()
        .filter(playground::Column::UserId.eq(user_id))
        .filter(playground::Column::ProjectId.eq(project_id))
        .filter(playground::Column::Status.eq(PlaygroundStatus::Active.to_string()))
        .order_by_desc(playground::Column::Id)
        .one(db)
        .await?;
    Ok(p)
}

/// update_playground updates a playground
pub async fn update_playground(
    project_id: i32,
    user_id: i32,
    source: String,
    old_playground: &playground::Model,
) -> Result<()> {
    // create new playground record
    let _ = create_playground(
        user_id,
        project_id,
        Language::from_str(&old_playground.language)?,
        source,
        old_playground.visiblity == PlaygroundVisibility::Public.to_string(),
    )
    .await?;
    // set old one as deleted
    let db = DB.get().unwrap();
    playground::Entity::update_many()
        .col_expr(
            playground::Column::Status,
            Expr::value(PlaygroundStatus::Deleted.to_string()),
        )
        .filter(playground::Column::Id.eq(old_playground.id))
        .exec(db)
        .await?;
    Ok(())
}
