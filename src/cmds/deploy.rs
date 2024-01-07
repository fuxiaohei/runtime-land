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
        let target_path = metadata.build.target.clone();

        // generate temp tar.gz file from os temp dir
        let tmp_tar_gz =
            std::env::temp_dir().join(format!("land-deploy-{}.tar.gz", metadata.project.name));
        debug!("tmp_tar_gz: {:?}", tmp_tar_gz);

        // pack files to tar.gz
        pack_file(
            metadata.build.src_files.clone(),
            &target_path,
            tmp_tar_gz.to_str().unwrap(),
        )?;

        // read tar.gz file
        let bundle = std::fs::read(tmp_tar_gz)?;
        debug!("bundle size: {}", bundle.len());

        // send deploy request
        let deploy_url = format!(
            "{}/api/v2/cli/deploy",
            self.cloud_server_url.as_ref().unwrap()
        );
        let req = DeployRequest {
            metadata,
            bundle_md5: format!("{:x}", md5::compute(&bundle)),
            bundle,
            user_token: config.user_token,
            user_uuid: config.user_uuid,
        };

        // send request
        let resp = ureq::post(&deploy_url)
            .set("Content-Type", "application/json")
            .send_json(serde_json::to_value(req)?);

        if resp.is_err() {
            cprintln!("<bright-red,bold>Upload error: {}</>", resp.err().unwrap(),);
            return Err(anyhow::anyhow!("Deploy failed!"));
        }

        let resp = resp.unwrap();
        if resp.status() != 200 {
            cprintln!("<bright-red,bold>Response error: {}</>", resp.into_string().unwrap());
            return Err(anyhow::anyhow!("Deploy failed!"));
        }

        cprintln!(
            "<bright-cyan,bold>Deploy Success</> to <bright-cyan>{}</>.",
            resp.into_string().unwrap()
        );

        Ok(())
    }
}

fn pack_file(mut files: Vec<String>, target_path: &str, output_path: &str) -> Result<()> {
    // apppend MANIFEST_FILE
    files.push(MANIFEST_FILE.to_string());
    // append target file
    files.push(target_path.to_string());
    let tar_gz = std::fs::File::create(output_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);
    for file in &files {
        // check file exists
        let fpath = std::path::Path::new(file);
        if !fpath.exists() {
            cprintln!(
                "<bright-red,bold>Warning</> file '{}' does not exist!",
                file
            );
            continue;
        }
        // assuming 'file' can be either a directory or a file
        tar.append_path(file)?;

        // if fpath is directory, append all files in directory
        if fpath.is_dir() {
            for entry in WalkDir::new(fpath).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }
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

fn validate_url(url: &str) -> Result<String, String> {
    let _: url::Url = url.parse().map_err(|_| "invalid url".to_string())?;
    Ok(url.to_string())
}
