use anyhow::Result;
use land_dao::models::deployment::Model as DeploymentModel;
use tracing::{debug, debug_span, info, warn};

/// run_background starts the background worker to handle the deployer's tasks.
pub async fn run_background() -> Result<()> {
    tokio::spawn(async {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(1));
        loop {
            ticker.tick().await;
            match run_inner().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("run_inner failed: {}", e);
                }
            }
        }
    });
    Ok(())
}

async fn run_inner() -> Result<()> {
    let span = debug_span!("[BACKGROUND]");
    let _enter = span.enter();

    // 1.read waiting tasks from db
    let dps =
        land_dao::deployment::list_by_status(land_dao::deployment::DeployStatus::Waiting).await?;
    if dps.is_empty() {
        debug!("No waiting tasks");
        return Ok(());
    }
    // 2. build wasm file for each task
    debug!("Found {} waiting tasks", dps.len());
    for dp in dps {
        build_wasm_file(&dp).await?;
    }
    Ok(())
}

async fn build_wasm_file(dp: &DeploymentModel) -> Result<()> {
    // 1. read project
    let project = land_dao::projects::get_by_id(dp.project_id, None).await?;
    if project.is_none() {
        land_dao::deployment::set_failed(dp.id, "Project not found or deleted".to_string()).await?;
        return Ok(());
    }
    let project = project.unwrap();
    // 2. if project is not a playground, set failed
    if project.created_by != land_dao::projects::ProjectCreatedBy::Playground.to_string() {
        land_dao::deployment::set_failed(dp.id, "Project is not a playground".to_string()).await?;
        return Ok(());
    }
    // 3. read playground
    let pl = land_dao::projects::get_playground_by_project(dp.user_id, dp.project_id).await?;
    if pl.is_none() {
        land_dao::deployment::set_failed(dp.id, "Playground not found".to_string()).await?;
        return Ok(());
    }
    let pl = pl.unwrap();

    // 4. write source code to file
    let dir = tempdir::TempDir::new("runtime-land")?;
    let source_js = dir.path().join(format!("{}_{}.js", pl.project_id, pl.id));
    debug!(
        "Write playground source to: {:?}, size: {}",
        source_js,
        pl.source.len()
    );
    let source_dir = source_js.parent().unwrap().to_path_buf();
    std::fs::create_dir_all(source_dir)?;
    std::fs::write(&source_js, pl.source)?;

    // 5. build js to wasm
    let target_wasm = dir.path().join(format!("{}_{}.wasm", pl.project_id, pl.id));
    land_wasm_gen::componentize_js(
        source_js.to_str().unwrap(),
        target_wasm.to_str().unwrap(),
        None,
    )?;
    debug!("Compile success: {:?}", target_wasm);

    // 6. set deploy task uploading
    land_dao::deployment::set_uploading(dp.id).await?;

    // 7. upload wasm to storage(r2)
    let now_text = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    let storage_file_name = format!("{}/{}_{}.wasm", project.uuid, dp.domain, now_text);
    let upload_data = std::fs::read(&target_wasm)?;
    let upload_data_md5 = format!("{:x}", md5::compute(&upload_data));
    let upload_data_size = upload_data.len() as i32;
    info!(
        file = storage_file_name,
        size = upload_data_size,
        "Uploading",
    );
    let global_storage = land_dao::settings::STORAGE.lock().await;
    global_storage
        .write(&storage_file_name, upload_data)
        .await?;
    info!(
        file = storage_file_name,
        size = upload_data_size,
        "Upload success"
    );

    // 8. set deploy task uploaded
    land_dao::deployment::set_uploaded(dp.id, storage_file_name, upload_data_md5, upload_data_size)
        .await?;

    Ok(())
}
