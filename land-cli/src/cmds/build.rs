use anyhow::Result;
use clap::Args;
use color_print::cprintln;
use land_common::manifest;
use std::process::Command;

/// Command Build
#[derive(Args, Debug)]
pub struct Build {
    #[clap(short = 'j', long = "js-engine")]
    pub js_engine: Option<String>,
}

impl Build {
    pub async fn run(&self) -> Result<()> {
        let metadata = manifest::Data::from_file(manifest::MANIFEST_FILE)?;
        build_internal(&metadata)?;

        cprintln!(
            "<bright-cyan,bold>Finished</> building project '{}'.",
            metadata.project.name
        );

        Ok(())
    }
}

fn run_command(cmd_str: &str) -> Result<()> {
    let args = cmd_str.split_whitespace().collect::<Vec<&str>>();
    let mut cmd = Command::new(args[0]);
    let child = cmd.args(&args[1..]).spawn()?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        return Err(anyhow::anyhow!(err));
    }
    Ok(())
}

pub fn build_internal(metadata: &manifest::Data) -> Result<()> {
    let build_command = metadata.build.command.clone();
    if !build_command.is_empty() {
        cprintln!("Run build command: {}", build_command);
        run_command(&build_command)?;
    }
    let target = metadata.build.target.clone();
    if metadata.project.language == "javascript" {
        let wasm_target = metadata.wasm_target();
        return land_core::build::js(&target, &wasm_target);
    }
    land_core::build::compile(&target)
}
