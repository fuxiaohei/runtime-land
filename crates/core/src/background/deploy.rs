use anyhow::{anyhow, Result};
use land_dao::{
    deploy_task,
    deployment::{self, DeployStatus, StorageInfo},
    playground, project,
    storage::GLOBAL,
    worker,
};
use tokio::sync::mpsc;
use tracing::{debug, info, instrument, warn};

/// DeployTask is the task for deployment
#[derive(Debug)]
pub struct Task {
    pub deploy_id: i32,     // deployment id is major priority
    pub playground_id: i32, // playground id is not zero and deploy_id must be zero at the same time
    pub project_id: i32,
}

#[derive(Debug)]
pub struct Deployer {
    tx: mpsc::Sender<Task>,
}

impl Deployer {
    pub async fn send(&self, task: Task) -> Result<()> {
        self.tx.send(task).await.map_err(|e| anyhow!(e))
    }
}

pub fn init() {
    let (tx, mut rx) = mpsc::channel(32);
    super::DEPLOY_SENDER.set(Deployer { tx }).unwrap();

    tokio::spawn(async move {
        while let Some(task) = rx.recv().await {
            info!("Deployment received: {:?}", task);
            if task.playground_id > 0 {
                handle_playground(task.playground_id, task.deploy_id, task.project_id).await;
                continue;
            }
            handle_deployment(task.deploy_id).await;
            continue;
        }
    });
}

/// handle_deploy_tasks is the function to handle deploy tasks
async fn handle_deploy_tasks(deploy_id: i32, task_id: String, project_id: i32) -> Result<()> {
    let living_workers = worker::list_online().await?;
    if living_workers.is_empty() {
        return Err(anyhow!("No living workers"));
    }
    for worker in living_workers {
        let _ = deploy_task::create(
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
    Ok(())
}

/// handle_playground_internal is the internal function to compile js source to wasm
async fn handle_playground_internal(playground_id: i32) -> Result<StorageInfo> {
    let pl = playground::get_by_id(playground_id).await?;
    if pl.is_none() {
        return Err(anyhow!("Playground not found"));
    }
    let pl = pl.unwrap();
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
    crate::build::js(source_js.to_str().unwrap(), target_wasm.to_str().unwrap())?;
    debug!("Compile success");
    // 3. get project info
    let project = project::get_by_id(pl.project_id).await?;
    if project.is_none() {
        return Err(anyhow!("Project not found"));
    }
    let project = project.unwrap();

    // 4. upload to storage
    let now_ts = chrono::Utc::now().timestamp();
    let storage_file_name = format!("{}/{}_{}.wasm", project.uuid, project.domain, now_ts);
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
    let global_storage = GLOBAL.lock().await;
    global_storage
        .write(&storage_file_name, upload_data)
        .await?;
    Ok(StorageInfo {
        path: storage_file_name,
        md5: upload_data_md5,
        size: upload_data_size,
    })
}

#[instrument("[DP]")]
async fn handle_playground(playground_id: i32, deploy_id: i32, project_id: i32) {
    let st = tokio::time::Instant::now();

    // 1. compile js to wasm from playground source
    let res = handle_playground_internal(playground_id).await;
    if let Err(err) = res {
        let _ =
            deployment::mark_status(deploy_id, DeployStatus::Failed, err.to_string(), None, None)
                .await;
        warn!(cost = st.elapsed().as_millis(), "Failed: {}", err);
        return;
    }
    let res = res.unwrap();

    // 2. send deploy task to worker
    let task_id = uuid::Uuid::new_v4().to_string();
    debug!(
        cost = st.elapsed().as_millis(),
        task_id = task_id,
        "Creating tasks"
    );
    if let Err(err) = handle_deploy_tasks(deploy_id, task_id.clone(), project_id).await {
        let _ =
            deployment::mark_status(deploy_id, DeployStatus::Failed, err.to_string(), None, None)
                .await;
        warn!(cost = st.elapsed().as_millis(), "Failed: {}", err);
        return;
    }

    // 3. update deployment status to deploying
    if let Err(err) = deployment::mark_status(
        deploy_id,
        DeployStatus::Deploying,
        "deploying".to_string(),
        Some(task_id.clone()),
        Some(res),
    )
    .await
    {
        let _ =
            deployment::mark_status(deploy_id, DeployStatus::Failed, err.to_string(), None, None)
                .await;
        warn!(cost = st.elapsed().as_millis(), "Failed: {}", err);
        return;
    }

    info!(cost = st.elapsed().as_millis(), "Deploying done");
}

async fn handle_deployment(_id: i32) {}
