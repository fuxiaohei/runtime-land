use anyhow::Result;

/// create a function data
pub async fn create(
    name: &str,
    owner_id: i64,
    md5: &str,
    description: &str,
    lang: &str,
    location: &str,
) -> Result<()> {
    let pool = crate::DB.get().unwrap();
    sqlx::query(
        "INSERT INTO function_data (name, owner_id, md5, description, lang, location, status, uuid, version) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    ).bind(name)
    .bind(owner_id)
    .bind(md5)
    .bind(description)
    .bind(lang)
    .bind(location)
    .bind(1) // default status = 1
    .bind(uuid::Uuid::new_v4().to_string())
    .bind("vv")
    .execute(pool).await?;
    Ok(())
}
