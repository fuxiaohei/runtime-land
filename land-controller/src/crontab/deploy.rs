use anyhow::{anyhow, Result};
use land_dao::confs::TaskValue;
use land_dao::models::deployment::Model as DeploymentModel;
use land_dao::{deployment, projects, worker};
use tracing::{debug, info, instrument, warn};

#[instrument("[DEPLOY]")]
pub async fn run_tasks() -> Result<()> {
    match handle_deploying().await {
        Ok(_) => {}
        Err(e) => {
            warn!("Deploy tasks failed: {}", e);
        }
    };
    match handle_review().await {
        Ok(_) => {}
        Err(e) => {
            warn!("Review tasks failed: {}", e);
        }
    };
    Ok(())
}

async fn handle_deploying() -> Result<()> {
    let dps = deployment::list_by_deploy_status(deployment::DeployStatus::Waiting).await?;
    if dps.is_empty() {
        // debug!("No waiting deploys");
        return Ok(());
    }
    debug!("Found {} waiting deploys", dps.len());
    for mut dp in dps {
        // set dp as compiling
        deployment::set_compiling(dp.id, dp.project_id).await?;
        tokio::spawn(async move {
            let dp_id = dp.id;
            let project_id = dp.project_id;
            if let Err(e) = handle_deploy_one(&mut dp).await {
                deployment::set_failed(dp_id, project_id, e.to_string())
                    .await
                    .unwrap();
                warn!("Handle deploy failed: {}", e);
            }
        });
    }
    Ok(())
}

async fn handle_deploy_one(dp: &mut DeploymentModel) -> Result<()> {
    // 1. read project data
    let project = projects::get_by_id(dp.project_id, None).await?;
    if project.is_none() {
        return Err(anyhow!("Project not found or deleted"));
    }
    let project = project.unwrap();
    // 2. project need created by playground
    if projects::ProjectCreatedBy::Playground.to_string() != project.created_by {
        return Err(anyhow!("Project is created by playground"));
    }
    // 3. read playground data
    let pl = projects::get_playground_by_project(dp.user_id, dp.project_id).await?;
    if pl.is_none() {
        return Err(anyhow!("Playground not found"));
    }
    let pl = pl.unwrap();

    // 4. write playground source code to temp file
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

    // 5. build js source to wasm
    let target_wasm = dir.path().join(format!("{}_{}.wasm", pl.project_id, pl.id));
    land_wasm_gen::componentize_js(
        source_js.to_str().unwrap(),
        target_wasm.to_str().unwrap(),
        None,
    )?;
    debug!("Compile success: {:?}", target_wasm);

    // 6. set deployment as uploading
    deployment::set_uploading(dp.id, dp.project_id).await?;

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

    // 8. set deployment as uploaded
    deployment::set_uploaded_and_deploying(
        dp.id,
        dp.project_id,
        storage_file_name.clone(),
        upload_data_md5.clone(),
        upload_data_size,
    )
    .await?;

    // 9. get living workers
    let workers = worker::list_online().await?;
    if workers.is_empty() {
        return Err(anyhow!("No online workers"));
    }

    // 10. create each task for each worker
    let (domain, _, service_name) = land_dao::settings::get_domain_settings().await?;
    let storage_settings = land_dao::settings::get_storage().await?;

    dp.storage_path.clone_from(&storage_file_name);
    dp.storage_md5.clone_from(&upload_data_md5);
    let task_value = TaskValue::new(dp, &storage_settings, &domain, &service_name)?;

    let task_content = serde_json::to_string(&task_value)?;
    for worker in workers {
        let task = deployment::create_task(
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

async fn handle_review() -> Result<()> {
    // 1. read deploying deployment
    let dps = deployment::list_by_deploy_status(deployment::DeployStatus::Deploying).await?;
    if dps.is_empty() {
        // debug!("No deploying tasks");
        return Ok(());
    }

    // 2. if no workers online, set dp as failed
    let workers = worker::list_online().await?;
    if workers.is_empty() {
        for dp in dps {
            deployment::set_failed(dp.id, dp.project_id, "No online workers".to_string()).await?;
            warn!(
                id = dp.id,
                domain = dp.domain,
                "Deployment failed, no online workers"
            );
        }
        return Ok(());
    }

    // 3. review each deployment
    for dp in dps {
        tokio::spawn(async move {
            if let Err(e) = handle_review_one(&dp).await {
                let _ = deployment::set_failed(dp.id, dp.project_id, e.to_string()).await;
                warn!("Review deploy failed: {}", e);
            }
        });
    }
    Ok(())
}

async fn handle_review_one(dp: &DeploymentModel) -> Result<()> {
    // 1. read tasks of this deployment
    let tasks = deployment::list_tasks_by_taskid(dp.task_id.clone()).await?;
    if tasks.is_empty() {
        return Err(anyhow!("No tasks found"));
    }

    // 2. check each task's status
    // if all tasks are sucess, set dp as deployed
    let total_count = tasks.len();
    let mut success_count = 0;
    let mut final_count = 0;
    for task in tasks {
        // if task is deploying, skip
        if task.deploy_status == deployment::DeployStatus::Deploying.to_string() {
            continue;
        }
        final_count += 1;
        if task.deploy_status == deployment::DeployStatus::Success.to_string() {
            success_count += 1;
        }
    }

    // 3. update deployment status by tasks status
    if total_count == success_count {
        deployment::set_success(dp.id, dp.project_id).await?;
        info!(id = dp.id, domain = dp.domain, "Deployment success");
    } else if final_count == total_count {
        deployment::set_failed(dp.id, dp.project_id, "Some tasks failed".to_string()).await?;
        warn!(
            id = dp.id,
            domain = dp.domain,
            "Deployment failed, some tasks failed"
        );
    } else {
        debug!(id = dp.id, domain = dp.domain, "Deployment still deploying");
    }
    Ok(())
}
