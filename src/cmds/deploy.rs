use anyhow::Result;
use clap::Args;
use color_print::cprintln;
use flate2::{write::GzEncoder, Compression};
use land_common::{MetaData, MANIFEST_FILE};
use serde::{Deserialize, Serialize};
use tar::Builder;
use tracing::debug;
use walkdir::WalkDir;

#[derive(Args, Debug)]
pub struct Deploy {
    /// Build the project before deploy
    #[clap(long)]
    pub build: Option<bool>,
    /// The url of cloud server
    #[clap(long = "url", value_parser = validate_url,default_value("https://cloud.runtime.land"))]
    pub cloud_server_url: Option<String>,
}

impl Deploy {
    pub async fn run(&self) -> Result<()> {
        // get local auth config
        let config = super::login::get_local_config();
        if config.is_none() {
            return Err(anyhow::anyhow!("Please login first!"));
        }
        let config = config.unwrap();

        let metadata = MetaData::from_file(MANIFEST_FILE)?;

        // generate temp tar.gz file from os temp dir
        let tmp_tar_gz =
            std::env::temp_dir().join(format!("land-deploy-{}.tar.gz", metadata.project.name));
        debug!("tmp_tar_gz: {:?}", tmp_tar_gz);

        // js project need add dist wasm file
        let mut src_files = metadata.build.src_files.clone();
        let output_path = format!("dist/{}.wasm", metadata.project.name);
        src_files.push(output_path);

        // pack files to tar.gz
        pack_file(src_files, tmp_tar_gz.to_str().unwrap())?;

        // read tar.gz file
        let bundle = std::fs::read(tmp_tar_gz)?;
        debug!("bundle size: {} KB", bundle.len() / 1024);

        // send deploy request
        let deploy_url = format!(
            "{}/api/v2/cli/deploy",
            self.cloud_server_url.as_ref().unwrap()
        );
        let deploy_res1 = post_deploy(
            bundle.clone(),
            metadata,
            config.user_token.clone(),
            config.user_uuid.clone(),
            deploy_url.clone(),
        );
        if deploy_res1.is_err() {
            cprintln!(
                "<bright-red,bold>Upload error: {}</>",
                deploy_res1.err().unwrap(),
            );
            return Err(anyhow::anyhow!("Deploy failed!"));
        };

        cprintln!("Upload success!\nWaiting for deploy...");

        // check deploy status
        let check_url = format!(
            "{}/api/v2/cli/deploy-check",
            self.cloud_server_url.as_ref().unwrap(),
        );
        let deploy_res1 = deploy_res1.unwrap();

        // check deploy status for every 1 second
        let mut time_counter = 0;
        loop {
            time_counter += 1;
            if time_counter > 40 {
                cprintln!("<bright-red,bold>Deploy timeout! Please check your network.</>");
                return Err(anyhow::anyhow!("Deploy timeout!"));
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
            let check_res = check_deploy(
                check_url.clone(),
                deploy_res1.deploy_id,
                config.user_token.clone(),
                config.user_uuid.clone(),
            );
            if check_res.is_err() {
                cprintln!(
                    "<bright-red,bold>Deploy failed! error: {}</>",
                    check_res.err().unwrap(),
                );
                return Err(anyhow::anyhow!("Deploy failed!"));
            }
            let check_res = check_res.unwrap();
            if check_res {
                break;
            }
            cprintln!("Waiting for deploy...");
            continue;
        }

        Ok(())
    }
}

fn post_deploy(
    bundle: Vec<u8>,
    metadata: MetaData,
    user_token: String,
    user_uuid: String,
    deploy_url: String,
) -> Result<DeployResponse> {
    let req = DeployRequest {
        metadata,
        bundle_md5: format!("{:x}", md5::compute(&bundle)),
        bundle,
        user_token,
        user_uuid,
    };

    // send request
    let resp = ureq::post(&deploy_url)
        .set("Content-Type", "application/json")
        .send_json(serde_json::to_value(req)?);

    if resp.is_err() {
        return Err(anyhow::anyhow!("bad response: {}", resp.err().unwrap()));
    }
    let resp = resp.unwrap();
    if resp.status() != 200 {
        return Err(anyhow::anyhow!("bad status code: {}", resp.status()));
    }
    let resp: DeployResponse = resp.into_json()?;
    Ok(resp)
}

fn check_deploy(
    check_url: String,
    deploy_id: i32,
    user_token: String,
    user_uuid: String,
) -> Result<bool> {
    let req = DeployCheckRequest {
        deploy_id,
        user_token,
        user_uuid,
    };
    let resp = ureq::post(&check_url)
        .set("Content-Type", "application/json")
        .send_json(serde_json::to_value(req)?);
    if resp.is_err() {
        return Err(anyhow::anyhow!("bad response: {}", resp.err().unwrap()));
    }
    let resp = resp.unwrap();
    if resp.status() != 200 {
        return Err(anyhow::anyhow!("bad status code: {}", resp.status()));
    }
    let check_res: DeployCheckResponse = resp.into_json()?;
    debug!("check_res: {:?}", check_res);
    if check_res.status == "success" {
        cprintln!(
            "<bright-green,bold>Deploy success! Visit url: \n\t{}</>",
            check_res.visit_url
        );
        return Ok(true);
    }
    Ok(false)
}

fn pack_file(mut files: Vec<String>, output_path: &str) -> Result<()> {
    debug!("files: {:?}", files);
    // apppend MANIFEST_FILE
    files.push(MANIFEST_FILE.to_string());
    // append target file
    // files.push(target_path.to_string());
    let tar_gz = std::fs::File::create(output_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);
    for file in &files {
        // check file exists
        let fpath = std::path::Path::new(file);
        if !fpath.exists() {
            continue;
        }
        // assuming 'file' can be either a directory or a file
        tar.append_path(file)?;
        debug!("pack file-1: {}", file);

        // if fpath is directory, append all files in directory
        if fpath.is_dir() {
            for entry in WalkDir::new(fpath).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }
                debug!("pack file-2: {}", path.to_str().unwrap());
                tar.append_path(path)?;
            }
        }
    }
    tar.finish()?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployRequest {
    pub metadata: MetaData,
    pub bundle: Vec<u8>,
    pub bundle_md5: String,
    pub user_token: String,
    pub user_uuid: String,
}

/// DeployResponse is the response for /cli/deploy
#[derive(Debug, Serialize, Deserialize)]
pub struct DeployResponse {
    pub visit_url: String,
    pub deploy_id: i32,
}

fn validate_url(url: &str) -> Result<String, String> {
    let _: url::Url = url.parse().map_err(|_| "invalid url".to_string())?;
    Ok(url.to_string())
}

/// DeployRequest is the request for /cli/deploy
#[derive(Debug, Serialize, Deserialize)]
pub struct DeployCheckRequest {
    pub deploy_id: i32,
    pub user_token: String,
    pub user_uuid: String,
}

/// DeployCheckResponse is the response for /cli/deploy
#[derive(Debug, Serialize, Deserialize)]
pub struct DeployCheckResponse {
    pub visit_url: String,
    pub status: String,
    pub deploy_uuid: String,
}
