use crate::{model::project, DB};
use anyhow::Result;
use rand::{thread_rng, Rng};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbBackend, EntityTrait, FromQueryResult, JsonValue,
    PaginatorTrait, QueryFilter, QueryOrder, Set, Statement,
};

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
        let rand_int = thread_rng().gen_range(100..=999);
        format!(
            "{}-{}-{}",
            random_word::gen(random_word::Lang::En),
            random_word::gen(random_word::Lang::En),
            rand_int,
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

/// find_by_id finds a project by id
pub async fn find_by_id(id: i32) -> Result<Option<project::Model>> {
    let db = DB.get().unwrap();
    let project = project::Entity::find_by_id(id).one(db).await?;
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

/// list_available lists all available projects
pub async fn list_available(owner_id: i32) -> Result<Vec<project::Model>> {
    let db = DB.get().unwrap();
    let projects = project::Entity::find()
        .filter(project::Column::OwnerId.eq(owner_id))
        .filter(project::Column::Status.ne(Status::Deleted.to_string()))
        .order_by_desc(project::Column::UpdatedAt)
        .all(db)
        .await?;
    Ok(projects)
}

/// remove_project removes a project
pub async fn remove_project(owner_id: i32, uuid: String) -> Result<i32> {
    let db = DB.get().unwrap();
    let project = project::Entity::find()
        .filter(project::Column::OwnerId.eq(owner_id))
        .filter(project::Column::Uuid.eq(uuid))
        .one(db)
        .await?;
    if project.is_none() {
        return Err(anyhow::anyhow!("project not found"));
    }
    let project = project.unwrap();
    let project_id = project.id;

    // if project status is pending, it can remove directly
    if project.status == Status::Pending.to_string() {
        let mut active_model: project::ActiveModel = project.into();
        active_model.status = Set(Status::Deleted.to_string());
        active_model.deleted_at = Set(Some(chrono::Utc::now()));
        active_model.update(db).await?;
        return Ok(project_id);
    }

    // TODO: remove active project
    Err(anyhow::anyhow!("project is not pending"))
}

/// rename renames a project
pub async fn rename(owner_id: i32, old_name: String, new_name: String) -> Result<project::Model> {
    let db = DB.get().unwrap();
    let project = find_by_name(old_name, owner_id)
        .await?
        .ok_or(anyhow::anyhow!("project not found"))?;
    let mut active_model: project::ActiveModel = project.into();
    active_model.name = Set(new_name);
    active_model.updated_at = Set(chrono::Utc::now());
    let project = active_model.update(db).await?;
    Ok(project)
}

/// get_stats gets the stats of deployments
pub async fn get_stats() -> Result<i32> {
    let db = DB.get().unwrap();
    let values: Vec<JsonValue> = JsonValue::find_by_statement(Statement::from_sql_and_values(
        DbBackend::MySql,
        r#"select count(id) as counter from project where status != 'deleted'"#,
        [],
    ))
    .all(db)
    .await?;
    let counter = values[0]["counter"].as_i64().unwrap() as i32;
    Ok(counter)
}

/// get_pagination gets the pagination of projects
pub async fn get_pagination(page: u64, page_size: u64) -> Result<(Vec<project::Model>, u64, u64)> {
    let db = DB.get().unwrap();
    let pager = project::Entity::find()
        .filter(project::Column::Status.ne(Status::Deleted.to_string()))
        .order_by_desc(project::Column::UpdatedAt)
        .paginate(db, page_size);
    let projects = pager.fetch_page(page).await?;
    let total_pages = pager.num_pages().await?;
    let total_items = pager.num_items().await?;
    Ok((projects, total_pages, total_items))
}
