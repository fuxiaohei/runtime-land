use anyhow::Result;
use once_cell::sync::OnceCell;
use std::collections::HashMap;

mod deploy;
mod review;
mod updater;

/// init is the function to initialize background tasks
pub fn init() {
    deploy::init();
    review::init();
    updater::init();
}

/// DEPLOY_SENDER is the sender for deployment
static DEPLOY_SENDER: OnceCell<deploy::Deployer> = OnceCell::new();

/// UPDATER_SENDER is the sender for updating
static UPDATER_SENDER: OnceCell<updater::Updater> = OnceCell::new();

/// send_deploying_task is the function to send deployment task
pub async fn send_deploying_task(
    deploy_id: i32,
    playground_id: i32,
    project_id: i32,
) -> Result<()> {
    DEPLOY_SENDER
        .get()
        .unwrap()
        .send(deploy::Task {
            deploy_id,
            playground_id,
            project_id,
        })
        .await
}

/// send_updater_task is the function to send updater task
pub async fn send_updater_task(ip: String, res: HashMap<String, String>) -> Result<()> {
    UPDATER_SENDER
        .get()
        .unwrap()
        .send(updater::Task { ip, res })
        .await
}
