use anyhow::Result;
use clap::Args;
use color_print::cprintln;
use land_core_service::metadata;
use std::process::Command;
use tracing::debug;

/// Command Build
#[derive(Args, Debug)]
pub struct Build {
    #[clap(short = 'j', long = "js-engine")]
    pub js_engine: Option<String>,
}

impl Build {
    pub async fn run(&self) -> Result<()> {
        debug!("Build command: {:?}", self);
        let meta = metadata::Data::from_file(metadata::DEFAULT_FILE)?;

        build_internal(&meta, self.js_engine.clone())?;

        cprintln!(
            "<bright-cyan,bold>Finished</> building project '{}'.",
            meta.name,
        );

        Ok(())
    }
}

fn run_command(cmd_str: &str) -> Result<()> {
    let args = cmd_str.split_whitespace().collect::<Vec<&str>>();
    if args.is_empty() {
        return Ok(());
    }
    cprintln!("Run build command: {}", cmd_str);
    let mut cmd = Command::new(args[0]);
    let child = cmd.args(&args[1..]).spawn()?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        return Err(anyhow::anyhow!(err));
    }
    Ok(())
}

pub fn build_internal(meta: &metadata::Data, js_engine: Option<String>) -> Result<()> {
    if let Some(cmd) = &meta.build.cmd {
        run_command(cmd)?;
    }
    if meta.language == "js" {
        let dist_wasm_path = meta.target_wasm_path();
        let dist_wasm_dir = std::path::Path::new(&dist_wasm_path).parent().unwrap();
        std::fs::create_dir_all(dist_wasm_dir)?;
        land_wasm_gen::componentize_js(&meta.build.main, &dist_wasm_path, js_engine)?;
        cprintln!("<green>Build project '{}' success</green>", meta.name);
        return Ok(());
    }
    Err(anyhow::anyhow!("Unsupported language: {}", meta.language))
}
