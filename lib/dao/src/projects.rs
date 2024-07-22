use crate::{
    deploys,
    models::{playground, project},
    now_time, DB,
};
use anyhow::Result;
use rand::Rng;
use random_word::Lang;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ItemsAndPagesNumber,
    PaginatorTrait, QueryFilter, QueryOrder,
};
use tracing::info;

#[derive(strum::Display, strum::EnumString, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum Language {
    JavaScript,
}

#[derive(strum::Display, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum CreatedBy {
    Playground,
    Blank,
}

#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Active,
    Disabled, // set disabled by user
    Deleted,
}

/// random_name generates a random project name
fn random_name() -> String {
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
pub async fn is_unique_name(name: &str) -> Result<bool> {
    let db = DB.get().unwrap();
    let project = project::Entity::find()
        .filter(project::Column::Name.eq(name))
        .one(db)
        .await?;
    Ok(project.is_none())
}

/// random_unique_name generates a random unique project name
pub async fn random_unique_name() -> Result<String> {
    let mut name = random_name();
    loop {
        if is_unique_name(&name).await? {
            break;
        }
        name = random_name();
    }
    Ok(name)
}

/// create_with_playground creates a new project with a playground
pub async fn create_with_playground(
    owner_id: i32,
    language: Language,
    description: String,
    source: String,
) -> Result<(project::Model, playground::Model)> {
    let p = create_internal(
        owner_id,
        language.clone(),
        description,
        CreatedBy::Playground,
    )
    .await?;
    let py = crate::playground::create(owner_id, p.id, language, source, false).await?;
    info!(
        owner_id = owner_id,
        "Create project with playground: {}", p.name
    );
    Ok((p, py))
}

/// create_internal creates a new project
async fn create_internal(
    owner_id: i32,
    language: Language,
    description: String,
    created_by: CreatedBy,
) -> Result<project::Model> {
    let name = random_unique_name().await?;
    let now = now_time();
    let mut project = project::Model {
        id: 0,
        owner_id,
        name: name.clone(),
        language: language.to_string(),
        status: Status::Active.to_string(),
        deploy_status: deploys::Status::WaitingDeploy.to_string(),
        deploy_message: "Waiting to deploy".to_string(),
        uuid: uuid::Uuid::new_v4().to_string(),
        description: description.to_string(),
        dev_domain: String::new(),
        prod_domain: name.to_string(),
        created_by: CreatedBy::Blank.to_string(),
        created_at: now,
        updated_at: now,
        deleted_at: None,
        metadata: None,
    };
    if created_by == CreatedBy::Playground {
        project.created_by = CreatedBy::Playground.to_string();
    }
    let mut project_active_model: project::ActiveModel = project.into();
    project_active_model.id = ActiveValue::default();
    let db = DB.get().unwrap();
    let project = project_active_model.insert(db).await?;
    Ok(project)
}

/// list lists all projects with optional user_id, optional name and pagination
pub async fn list(
    user_id: Option<i32>,
    search: Option<String>,
    page: u64,
    page_size: u64,
) -> Result<(Vec<project::Model>, ItemsAndPagesNumber)> {
    let db = DB.get().unwrap();
    let mut select = project::Entity::find()
        .filter(project::Column::Status.ne(Status::Deleted.to_string()))
        .order_by_desc(project::Column::UpdatedAt);
    if let Some(user_id) = user_id {
        select = select.filter(project::Column::OwnerId.eq(user_id));
    }
    if let Some(search) = search {
        let search = format!("%{}%", search);
        select = select.filter(project::Column::Name.like(search));
    }
    let pager = select.paginate(db, page_size);
    let projects = pager.fetch_page(page - 1).await?;
    let pages = pager.num_items_and_pages().await?;
    Ok((projects, pages))
}

/// get_by_name gets a project by name
pub async fn get_by_name(name: &str, user_id: Option<i32>) -> Result<Option<project::Model>> {
    let db = DB.get().unwrap();
    let mut select = project::Entity::find()
        .filter(project::Column::Name.eq(name))
        .filter(project::Column::Status.ne(Status::Deleted.to_string()));
    if let Some(user_id) = user_id {
        select = select.filter(project::Column::OwnerId.eq(user_id));
    }
    let project = select.one(db).await?;
    Ok(project)
}

/// get_by_id gets a project by id
pub async fn get_by_id(id: i32) -> Result<Option<project::Model>> {
    let db = DB.get().unwrap();
    let project = project::Entity::find()
        .filter(project::Column::Id.eq(id))
        .filter(project::Column::Status.ne(Status::Deleted.to_string()))
        .one(db)
        .await?;
    Ok(project)
}

/// update_names updates a project name
pub async fn update_names(id: i32, name: &str, desc: &str) -> Result<()> {
    let db = DB.get().unwrap();
    project::Entity::update_many()
        .col_expr(project::Column::Name, Expr::value(name))
        .col_expr(project::Column::ProdDomain, Expr::value(name))
        .col_expr(project::Column::Description, Expr::value(desc))
        .col_expr(project::Column::UpdatedAt, Expr::value(now_time()))
        .filter(project::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(())
}

/// set_deploy_status sets a deploy status to a project
pub async fn set_deploy_status(id: i32, status: deploys::Status, msg: &str) -> Result<()> {
    let db = DB.get().unwrap();
    project::Entity::update_many()
        .col_expr(
            project::Column::DeployStatus,
            Expr::value(status.to_string()),
        )
        .col_expr(project::Column::DeployMessage, Expr::value(msg))
        .filter(project::Column::Id.eq(id))
        .exec(db)
        .await?;
    Ok(())
}
