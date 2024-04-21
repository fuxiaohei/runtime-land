use anyhow::Result;
use land_dao::models::deployment::Model as DeploymentModel;
use serde::{Deserialize, Serialize};
use tracing::{debug, debug_span, info, warn, Instrument};

pub async fn run_tasks() -> Result<()> {
    // 1.read waiting tasks from db
    let dps =
        land_dao::deployment::list_by_status(land_dao::deployment::DeployStatus::Waiting).await?;
    if dps.is_empty() {
        // debug!("No waiting tasks");
        return Ok(());
    }
    // 2. handle each task
    debug!("Found {} waiting tasks", dps.len());
    for dp in dps {
        tokio::spawn(async move {
            if let Err(e) = handle_deploy(&dp)
                .instrument(debug_span!("[DEPLOY-1]", dp = dp.id))
                .await
            {
                warn!("Handle deploy failed: {}", e);
            }
        });
    }
    Ok(())
}

async fn handle_deploy(dp: &DeploymentModel) -> Result<()> {
    // 0. set current task compiling
    land_dao::deployment::set_compiling(dp.id).await?;

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
    land_dao::deployment::set_uploaded(
        dp.id,
        storage_file_name.clone(),
        upload_data_md5,
        upload_data_size,
    )
    .await?;

    // 9. get living workers
    let workers = land_dao::worker::list_online().await?;
    if workers.is_empty() {
        land_dao::deployment::set_failed(dp.id, "No online workers".to_string()).await?;
        warn!("Deployment {} failed, no online workers", dp.id);
        return Ok(());
    }

    // 10. create each task for each worker
    let (domain, _) = land_dao::settings::get_domain_settings().await?;
    let storage_settings = land_dao::settings::get_storage().await?;
    let task_value = TaskValue {
        user_uuid: dp.user_uuid.clone(),
        project_uuid: dp.project_uuid.clone(),
        domain: format!("{}.{}", dp.domain, domain),
        download_url: storage_settings.build_url(&storage_file_name)?,
        wasm_path: storage_file_name,
        task_id: dp.task_id.clone(),
    };
    let task_content = serde_json::to_string(&task_value)?;
    for worker in workers {
        let task = land_dao::deployment::create_task(
            worker.id,
            worker.ip,
            dp.project_id,
            dp.id,
            dp.task_id.clone(),
            task_content.clone(),
        )
        .await?;
        info!("Create task {} for worker {}", task.id, worker.id);
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct TaskValue {
    user_uuid: String,
    project_uuid: String,
    domain: String,
    download_url: String,
    wasm_path: String,
    task_id: String,
}
