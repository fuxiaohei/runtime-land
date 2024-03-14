use super::DeployTask;
use anyhow::{anyhow, Result};
use land_dao::deployment::{self, StorageInfo};
use land_dao::models::playground::Model as PlaygroundModel;
use land_dao::models::project::Model as ProjectModel;
use land_dao::{playground, project};
use once_cell::sync::OnceCell;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// global deploy task channel
pub static DEPLOY_TASK_SENDER: OnceCell<mpsc::Sender<DeployTask>> = OnceCell::new();

/// init starts deploy tasks channel receiver
pub async fn init() -> Result<()> {
    let (tx, mut rx) = mpsc::channel(32);
    DEPLOY_TASK_SENDER.set(tx).unwrap();

    tokio::spawn(async move {
        while let Some(t) = rx.recv().await {
            info!("Deploy task received: {:?}", t);
            let deploy_id = t.deploy_id;
            match handle_task(t).await {
                Ok(_) => (),
                Err(e) => {
                    warn!("send_deploy_task error: {:?}", e);
                    let _ = deployment::mark_status(
                        deploy_id,
                        deployment::DeployStatus::Failed,
                        format!("failed: {:?}", e),
                        None,
                        None,
                    )
                    .await;
                }
            }
        }
    });
    Ok(())
}

pub async fn handle_task(t: DeployTask) -> Result<()> {
    if t.playground_id > 0 {
        let (p, py) = prepare_data(&t).await?;
        // compile and upload playground to storage
        let upload_res = compile_and_upload_playground(p, py).await?;
        // upload done, create task-id and subtasks
        let task_id = create_tasks(t.deploy_id, t.project_id).await?;
        // update deployment to deploying
        let info = StorageInfo {
            path: upload_res.path,
            md5: upload_res.md5,
            size: upload_res.size,
        };
        land_dao::deployment::mark_status(
            t.deploy_id,
            land_dao::deployment::DeployStatus::Deploying,
            "deploying".to_string(),
            Some(task_id.clone()),
            Some(info),
        )
        .await?;
        return Ok(());
    }
    todo!("handle task when playground_id = 0")
}

async fn prepare_data(t: &DeployTask) -> Result<(ProjectModel, PlaygroundModel)> {
    let p = project::get_by_id(t.project_id).await?;
    if p.is_none() {
        return Err(anyhow!("Project not found"));
    }
    let p = p.unwrap();
    let py = playground::get_by_id(t.playground_id).await?;
    if py.is_none() {
        return Err(anyhow!("Playground not found"));
    }
    let py = py.unwrap();
    Ok((p, py))
}

async fn compile_and_upload_playground(
    p: ProjectModel,
    pl: PlaygroundModel,
) -> Result<UploadResult> {
    // 1. create temp file to save js source
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
    let target_wasm = dir.path().join(format!("{}_{}.wasm", pl.project_id, pl.id));

    // 2. compile js source to wasm
    build_js(source_js.to_str().unwrap(), target_wasm.to_str().unwrap())?;
    debug!("Compile success");

    // 3. upload to r2
    upload_wasm(target_wasm.to_str().unwrap().to_string(), p.domain, p.uuid).await
}

/// js compile to js to wasm component
fn build_js(src: &str, target: &str) -> Result<()> {
    // compile js to wizer
    land_wit::compile_js(src, target, None)?;
    compile_js(target)
}

/// compile wasm to wasm component
fn compile_js(target: &str) -> Result<()> {
    // use wasm-opt to optimize wasm if wasm-opt exists
    if let Some(op) = land_wit::optimize(target)? {
        std::fs::rename(op, target)?;
    }

    // encode wasm module to component
    land_wit::encode_component(target, target)?;

    // check target exists
    if !std::path::Path::new(target).exists() {
        return Err(anyhow::anyhow!(
            "Build target '{}' does not exist!",
            &target,
        ));
    }
    Ok(())
}

struct UploadResult {
    pub path: String,
    pub md5: String,
    pub size: i32,
}

async fn upload_wasm(
    target_wasm: String,
    project_domain: String,
    project_uuid: String,
) -> Result<UploadResult> {
    let now_ts = chrono::Utc::now().timestamp();
    let storage_file_name = format!("{}/{}_{}.wasm", project_uuid, project_domain, now_ts);
    let upload_data = std::fs::read(&target_wasm)?;
    let upload_data_md5 = format!("{:x}", md5::compute(&upload_data));
    let upload_data_size = upload_data.len() as i32;
    info!(
        file = storage_file_name,
        size = upload_data_size,
        "Uploading",
    );
    debug!(
        "Uploading wasm to storage: {:?}, size: {}",
        storage_file_name, upload_data_size
    );
    let global_storage = land_dao::storage::GLOBAL.lock().await;
    global_storage
        .write(&storage_file_name, upload_data)
        .await?;
    Ok(UploadResult {
        path: storage_file_name,
        md5: upload_data_md5,
        size: upload_data_size,
    })
}

async fn create_tasks(deploy_id: i32, project_id: i32) -> Result<String> {
    let task_id = uuid::Uuid::new_v4().to_string();
    debug!(task_id = task_id, "Creating tasks");

    let living_workers = land_dao::worker::list_online().await?;
    if living_workers.is_empty() {
        return Err(anyhow!("No living workers"));
    }
    for worker in living_workers {
        let _ = land_dao::deploy_task::create(
            deploy_id,
            worker.id,
            worker.ip.clone(),
            project_id,
            task_id.clone(),
        )
        .await?;
        debug!(
            worker_id = worker.id,
            worker_ip = worker.ip,
            "Create deploy task"
        );
    }
    Ok(task_id)
}
