use anyhow::Result;
use futures_util::StreamExt;
use land_core::storage::{self, GLOBAL};
use tracing::{debug, info};

/// download_file saves the remote file to local
pub async fn download_file(download_url: &str, path: &str) -> Result<()> {
    // if local file exist, skip download
    let local_op = GLOBAL.lock().await;
    if local_op.is_exist(path).await? {
        debug!("local file exist, path: {}", path);
        return Ok(());
    }
    let resp = reqwest::get(download_url).await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "request file failed, status: {}, url:{}",
            resp.status(),
            download_url,
        ));
    }
    let content_length = resp.content_length().unwrap_or(0);
    let mut reader = resp.bytes_stream();
    let mut writer = local_op.writer(path).await?;
    while let Some(bytes) = reader.next().await {
        let bytes = bytes?;
        writer.write(bytes).await?;
    }
    writer.close().await?;
    info!(
        "save remote to local, path: {}, size:{}",
        path, content_length
    );
    Ok(())
}

/// remove_local removes the local file
pub async fn remove_local(path: &str) -> Result<()> {
    storage::delete(path).await?;
    Ok(())
}
