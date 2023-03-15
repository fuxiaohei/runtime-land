use crate::env::CliEnv;
use anyhow::Result;
use md5::{Digest, Md5};
use moni_core::{rpc, Meta, DEFAULT_METADATA_FILE};
use std::io::Write;
use std::path::Path;
use tracing::{debug, error, info};
use walkdir::WalkDir;

pub async fn deploy(env: &CliEnv, meta: &Meta) {
    // upload function
    match upload(env, meta).await {
        Ok(_) => info!("Upload function success"),
        Err(e) => error!("Upload function failed: {:?}", e),
    }
}

async fn upload(env: &CliEnv, meta: &Meta) -> Result<(), Box<dyn std::error::Error>> {
    // read output file bytes
    let (output_bytes, output_hash) = build_gzip(&meta.get_output(), &meta.get_src_dir())?;

    let request = rpc::CreateFunctionRequest {
        name: meta.name.clone(),
        description: meta.description.clone(),
        code: output_bytes,
        md5: output_hash,
    };

    // create rpc client with token
    let mut client = match rpc::new_client_with_token(
        env.api_host.clone(),
        env.api_key.clone(),
        env.api_jwt_token.clone(),
    )
    .await
    {
        Ok(client) => client,
        Err(e) => {
            error!("Connect cloud failed: {:?}", e);
            std::process::exit(1);
        }
    };
    let response = client.create_function(request).await?;
    debug!("upload response={:?}", response);
    Ok(())
}

pub fn build_gzip(output: &str, src_dir: &str) -> anyhow::Result<(Vec<u8>, String)> {
    let output_path = Path::new(output);
    let output_path = output_path.file_name().unwrap().to_str().unwrap();
    let bundle_file = output_path.replace(".wasm", ".zip");

    let file = std::fs::File::create(&bundle_file).unwrap();
    let mut zip = zip::ZipWriter::new(file);

    // add wasm file
    debug!("[zip] add wasm file: {}", output_path);
    zip.start_file(output_path, Default::default())?;
    zip.write_all(&std::fs::read(output)?)?;

    // add metadata file
    debug!("[bundle] add metadata file: {}", DEFAULT_METADATA_FILE);
    zip.start_file(DEFAULT_METADATA_FILE, Default::default())?;
    zip.write_all(&std::fs::read(DEFAULT_METADATA_FILE)?)?;

    // add src directory
    let src_dir = Path::new(src_dir);
    let walkdir = WalkDir::new(src_dir);
    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let path = path.to_str().unwrap();
        debug!("[bundle] add src file: {}", path);
        zip.start_file(path, Default::default())?;
        zip.write_all(&std::fs::read(path)?)?;
    }

    zip.flush().expect("flush zip file");

    // show bundle size
    let bundle_size = std::fs::metadata(&bundle_file)?.len();
    info!(
        "bundle size: {:.2} MB",
        bundle_size as f64 / 1024.0 / 1024.0
    );

    let mut hasher = Md5::new();
    let bundle_content = std::fs::read(&bundle_file)?;
    hasher.update(&bundle_content);
    let bundle_hash = format!("{:x}", hasher.finalize());

    Ok((bundle_content, bundle_hash))
}
