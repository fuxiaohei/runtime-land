use anyhow::anyhow;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::{debug, info};
use wit_bindgen_core::wit_parser::Resolve;
use wit_bindgen_core::{Files, WorldGenerator};
use wit_component::ComponentEncoder;

/// GuestGeneratorType is the type of the guest generator.
pub enum GuestGeneratorType {
    Rust,
    Js,
    TinyGo,
}

impl GuestGeneratorType {
    /// create generator by type
    fn create_generator(&self) -> Result<Box<dyn WorldGenerator>> {
        match self {
            GuestGeneratorType::Rust => {
                let opts = wit_bindgen_rust::Opts {
                    macro_export: true,
                    rustfmt: true,
                    ..Default::default()
                };
                let builder = opts.build();
                Ok(builder)
            }
            _ => Err(anyhow!("unsupport guest generator")),
        }
    }
}

/// parse wit file and return world id
pub fn generate_guest(
    wit_dir: &Path,
    world: Option<String>,
    t: GuestGeneratorType,
) -> Result<HashMap<String, String>> {
    let mut generator = t.create_generator()?;

    let mut resolve = Resolve::default();
    let pkg = resolve.push_dir(wit_dir)?.0;

    let mut output_maps = HashMap::new();
    let mut files = Files::default();
    let world = resolve.select_world(pkg, world.as_deref())?;
    generator.generate(&resolve, world, &mut files);
    for (name, contents) in files.iter() {
        output_maps.insert(
            name.to_string(),
            String::from_utf8_lossy(contents).to_string(),
        );
    }
    Ok(output_maps)
}

/// compile_rust compiles the Rust code in the current directory.
pub fn compile_rust(arch: &str, target: &str) -> Result<()> {
    // cargo build --target arch --release
    let mut cmd = Command::new("cargo");
    let child = cmd
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg(arch)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to execute cargo child process");
    let output = child
        .wait_with_output()
        .expect("failed to wait on cargo child process");
    if output.status.success() {
        info!("Cargo build wasm success");
    } else {
        return Err(anyhow!("Cargo build wasm failed: {:?}", output));
    }

    // check target file is exist
    if !PathBuf::from(target).exists() {
        return Err(anyhow!("Wasm file not found: {}", target));
    }

    Ok(())
}

pub fn compile_js(
    _target: &str,
    _src_js_path: &str,
    _js_engine_path: Option<String>,
) -> Result<()> {
    unimplemented!()
}

/// convert_component is used to convert wasm module to component
pub fn convert_component(path: &str, output: Option<String>) -> Result<()> {
    debug!("Convert component, {path}");
    let file_bytes = std::fs::read(path).expect("parse wasm file error");
    let wasi_adapter = include_bytes!("../engine/wasi_snapshot_preview1.reactor.wasm");

    let component = ComponentEncoder::default()
        .module(&file_bytes)
        .expect("Pull custom sections from module")
        .validate(true)
        .adapter("wasi_snapshot_preview1", wasi_adapter)
        .expect("Add adapter to component")
        .encode()
        .expect("Encode component");

    let output = output.unwrap_or_else(|| path.to_string());
    std::fs::write(&output, component).expect("Write component file error");
    info!("Convert component success, {}", &output);

    // get output file size
    let size = std::fs::metadata(&output)?.len();
    info!("Component size: {} KB", size / 1024);
    Ok(())
}
