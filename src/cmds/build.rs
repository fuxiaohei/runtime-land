use anyhow::Result;
use clap::Args;
use color_print::cprintln;
use land_common::{MetaData, MANIFEST_FILE};

/// Command Build
#[derive(Args, Debug)]
pub struct Build {
    #[clap(short = 'j', long = "js-engine")]
    pub js_engine: Option<String>,
}

impl Build {
    pub async fn run(&self) -> Result<()> {
        let metadata = MetaData::from_file(MANIFEST_FILE)?;
        let build_command = metadata.build.command;
        if !build_command.is_empty() {
            // run build command in manifest file
            cprintln!(
                "<bright-cyan,bold>Building</> project '{}' with command '{}':",
                metadata.project.name,
                build_command
            );
            land_compiler::build_command(&build_command)?;

            // check build output, if not exist, return error
            if !std::path::Path::new(&metadata.build.target).exists() {
                return Err(anyhow::anyhow!(
                    "Build output '{}' does not exist!",
                    &metadata.build.target,
                ));
            }
        }

        // generate component
        land_compiler::generate_component(
            &metadata.project.name,
            &metadata.build.target,
            &metadata.project.language,
            self.js_engine.clone(),
        )?;

        cprintln!(
            "<bright-cyan,bold>Finished</> building project '{}'.",
            metadata.project.name
        );
        Ok(())
    }
}
