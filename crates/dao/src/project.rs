use std::collections::HashMap;

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

/// find_by_uuid finds a project by uuid
pub async fn find_by_uuid(uuid: String, owner_id: i32) -> Result<Option<project::Model>> {
    let db = DB.get().unwrap();
    let project = project::Entity::find()
        .filter(project::Column::Uuid.eq(uuid))
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

/// set_inactive sets a project to inactive
pub async fn set_inactive(project_id: i32) -> Result<project::Model> {
    let db = DB.get().unwrap();
    let project = project::Entity::find_by_id(project_id)
        .one(db)
        .await?
        .unwrap();
    let mut active_model: project::ActiveModel = project.into();
    active_model.updated_at = Set(chrono::Utc::now());
    active_model.status = Set(Status::InActive.to_string());
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

    // set all deployments to deleted
    super::deployment::set_deleted_by_project(project_id).await?;

    // set project to deleted
    let mut active_model: project::ActiveModel = project.into();
    active_model.status = Set(Status::Deleted.to_string());
    active_model.deleted_at = Set(Some(chrono::Utc::now()));
    active_model.update(db).await?;
    Ok(project_id)
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

/// list_all_available lists all available projects
pub async fn list_all_available() -> Result<Vec<project::Model>> {
    let db = DB.get().unwrap();
    let projects = project::Entity::find()
        .filter(project::Column::Status.ne(Status::Deleted.to_string()))
        .order_by_desc(project::Column::UpdatedAt)
        .all(db)
        .await?;
    Ok(projects)
}

/// list_all_available_with_page lists all available projects with pagination
pub async fn list_all_available_with_page(
    search: Option<String>,
    page: u64,
    page_size: u64,
) -> Result<(Vec<project::Model>, u64, u64)> {
    if page < 1 || page_size < 1 {
        return Err(anyhow::anyhow!("page and page_size must be greater than 0"));
    }
    let db = DB.get().unwrap();
    let mut query = project::Entity::find()
        .filter(project::Column::Status.ne(Status::Deleted.to_string()))
        .order_by_desc(project::Column::UpdatedAt);
    if let Some(search) = search {
        query = query.filter(project::Column::Name.contains(search));
    }
    let pager = query.paginate(db, page_size);
    let projects = pager.fetch_page(page - 1).await?;
    let total_pages = pager.num_pages().await?;
    let total_items = pager.num_items().await?;
    Ok((projects, total_pages, total_items))
}

/// is_recent_updated checks if the project is recent updated
pub async fn is_recent_updated() -> Result<bool> {
    let db = DB.get().unwrap();
    let project: Option<crate::Project> = project::Entity::find()
        .filter(project::Column::Status.ne(Status::Deleted.to_string()))
        .order_by_desc(project::Column::UpdatedAt)
        .one(db)
        .await?;
    if project.is_none() {
        return Ok(false);
    }
    let project = project.unwrap();
    let now = chrono::Utc::now();
    let updated_at = project.updated_at;
    let duration = now.signed_duration_since(updated_at);
    let duration = duration.num_seconds();
    if duration > 60 {
        return Ok(false);
    }
    Ok(true)
}

/// list_by_ids lists projects by ids
pub async fn list_by_ids(project_ids: Vec<i32>) -> Result<HashMap<i32, project::Model>> {
    let db = DB.get().unwrap();
    let projects = project::Entity::find()
        .filter(project::Column::Id.is_in(project_ids))
        .all(db)
        .await?;
    let mut map = HashMap::new();
    for project in projects {
        map.insert(project.id, project);
    }
    Ok(map)
}

/// list_counter_by_owners lists the counter of projects by owners
pub async fn list_counter_by_owners(owner_ids: Vec<i32>) -> Result<HashMap<i32, usize>> {
    if owner_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let db = DB.get().unwrap();
    let sql = format!(
        r#"select count(id) as counter, owner_id from project where owner_id in ({}) and status != 'deleted' group by owner_id"#,
        owner_ids
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );
    let values: Vec<JsonValue> =
        JsonValue::find_by_statement(Statement::from_sql_and_values(DbBackend::MySql, sql, []))
            .all(db)
            .await?;
    let mut map = HashMap::new();
    for value in values {
        let counter = value["counter"].as_i64().unwrap() as usize;
        let owner_id = value["owner_id"].as_i64().unwrap() as i32;
        map.insert(owner_id, counter);
    }
    Ok(map)
}
