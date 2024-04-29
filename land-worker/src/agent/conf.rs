use super::{TaskTotal, TaskValue};
use crate::agent::{CLIENT, DATA_DIR};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info};
use walkdir::WalkDir;

async fn download_wasm(data_dir: &str, task: &TaskValue) -> Result<()> {
    let wasm_path = format!("{}/{}", data_dir, task.wasm_path);
    if !PathBuf::from(wasm_path.clone()).exists() {
        // 1.1 download wasm from url
        let resp = CLIENT.get(&task.download_url).send().await?;
        let wasm_bytes = resp.bytes().await?;
        // 1.2 save wasm to file
        let wasm_dir = PathBuf::from(wasm_path.clone());
        let wasm_dir = wasm_dir.parent().unwrap();
        std::fs::create_dir_all(wasm_dir)?;
        info!("Save wasm to file: {}", wasm_path);
        std::fs::write(wasm_path, wasm_bytes)?;
    } else {
        debug!("Wasm file already exists: {}", wasm_path);
    }
    Ok(())
}

fn write_traefik_conf(data_dir: &str, task: &TaskValue) -> Result<String> {
    let conf_file = format!(
        "{}/traefik/fn_{}.yaml",
        data_dir,
        task.domain.replace('.', "_")
    );
    // if old file exist, check file md5
    if PathBuf::from(conf_file.clone()).exists() {
        let old_conf = std::fs::read_to_string(conf_file.clone())?;
        let old_md5 = format!("{:x}", md5::compute(old_conf.as_bytes()));
        if old_md5.eq(&task.traefik_checksum.clone().unwrap_or_default()) {
            debug!("Traefik conf file already exists: {}", conf_file);
            return Ok(conf_file);
        }
    }
    let conf_dir = PathBuf::from(conf_file.clone());
    let conf_dir = conf_dir.parent().unwrap();
    std::fs::create_dir_all(conf_dir)?;

    info!("Write traefik conf to file: {}", conf_file);
    std::fs::write(&conf_file, task.traefik.clone().unwrap_or_default())?;
    Ok(conf_file)
}

/// handle_task handles the task
pub async fn handle_task(task: &TaskValue) -> Result<()> {
    // 1. save wasm to file
    let data_dir = DATA_DIR.lock().await;
    download_wasm(data_dir.as_str(), task).await?;

    // 2. load wasm into mem
    land_wasm::pool::prepare_worker(&task.wasm_path, true).await?;

    // 3. write traefik conf after wasm is ready
    let _ = write_traefik_conf(data_dir.as_str(), task)?;

    Ok(())
}

/// handle_total handles the total tasks
pub async fn handle_total(data_dir: &str, total: &TaskTotal) -> Result<()> {
    let mut writed_conf_files = HashMap::new();
    for task in &total.tasks {
        // 1. it downloads wasm module file
        download_wasm(data_dir, task).await?;

        // 2. not load wasm to memory, generate aot wasm to make sure loading quickly
        let wasm_path = format!("{}/{}", data_dir, task.wasm_path);
        land_wasm::pool::compile_aot(&wasm_path).await?;

        // 3. write traefik conf
        let conf_file = write_traefik_conf(data_dir, task)?;
        writed_conf_files.insert(conf_file, true);
    }

    // clean deleted deploys conf file
    let traefik_conf_dir = format!("{}/traefik", data_dir);
    for entry in WalkDir::new(traefik_conf_dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let filename = path.file_name().unwrap().to_str().unwrap();
        if !filename.starts_with("fn_") {
            continue;
        }
        // if path exist in writed_conf_files, continue,
        // otherwise, delete the file
        if writed_conf_files.contains_key(path.to_str().unwrap()) {
            continue;
        }
        info!("Delete traefik conf file: {}", path.to_str().unwrap());
        std::fs::remove_file(path)?;
    }
    Ok(())
}
