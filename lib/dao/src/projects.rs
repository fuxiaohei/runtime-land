use crate::{
    deploys,
    models::{playground, project},
    now_time, DB,
};
use anyhow::Result;
use rand::Rng;
use random_word::Lang;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
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
        deploy_status: deploys::Status::Waiting.to_string(),
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
