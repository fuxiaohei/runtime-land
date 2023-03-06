use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::{debug, info};
use which::which;
use wit_bindgen_core::{Files, WorldGenerator};
use wit_component::ComponentEncoder;
use wit_parser::{Resolve, UnresolvedPackage};

/// GuestGeneratorType is the type of the guest generator.
pub enum GuestGeneratorType {
    Rust,
    Js,
    Golang,
}

impl GuestGeneratorType {
    /// create generator by type
    /// /// Generate guest code builder
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
    s: &str,
    world: Option<String>,
    t: GuestGeneratorType,
) -> Result<HashMap<String, String>> {
    // parse exported world in wit file
    let path = Path::new(s);
    if !path.is_file() {
        panic!("wit file `{}` does not exist", path.display());
    }
    // prepare resolver
    let mut resolve = Resolve::default();
    let pkg = resolve.push(UnresolvedPackage::parse_file(path)?, &Default::default())?;

    let world = match &world {
        Some(world) => {
            let mut parts = world.splitn(2, '.');
            let doc = parts.next().unwrap();
            let world = parts.next();
            let doc = *resolve.packages[pkg]
                .documents
                .get(doc)
                .ok_or_else(|| anyhow!("no document named `{doc}` in package"))?;
            match world {
                Some(name) => *resolve.documents[doc]
                    .worlds
                    .get(name)
                    .ok_or_else(|| anyhow!("no world named `{name}` in document"))?,
                None => resolve.documents[doc]
                    .default_world
                    .ok_or_else(|| anyhow!("no default world in document"))?,
            }
        }
        None => {
            let mut docs = resolve.packages[pkg].documents.iter();
            let (_, doc) = docs
                .next()
                .ok_or_else(|| anyhow!("no documents found in package"))?;
            if docs.next().is_some() {
                bail!("multiple documents found in package, specify a default world")
            }
            resolve.documents[*doc]
                .default_world
                .ok_or_else(|| anyhow!("no default world in document"))?
        }
    };

    // create generator
    let mut generator = t.create_generator()?;

    // generate file
    let mut files = Files::default();
    generator.generate(&resolve, world, &mut files);

    let mut output_maps = HashMap::new();
    for (name, contents) in files.iter() {
        output_maps.insert(
            name.to_string(),
            String::from_utf8_lossy(contents).to_string(),
        );
    }
    Ok(output_maps)
}

/// convert_component is used to convert wasm module to component
pub fn convert_component(path: &str, output: Option<String>) -> Result<()> {
    debug!("Convert component, {path}");
    let file_bytes = std::fs::read(path).expect("parse wasm file error");
    let wasi_adapter = include_bytes!("../engine/wasi_snapshot_preview1.wasm");

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
    Ok(())
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

pub fn compile_js(target: &str, src_js_path: &str, js_engine_path: Option<String>) -> Result<()> {
    // js need wizer command
    let cmd = match which("wizer") {
            Ok(cmd) => cmd,
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "Wizer not found \n\tplease install wizer first: \n\tcargo install wizer --all-features\n\tmore infomation see: https://github.com/bytecodealliance/wizer"
                ))
            }
        };

    // create dir
    let engine_dir = Path::new(&target).parent().unwrap();
    std::fs::create_dir_all(engine_dir).expect("create dir failed");
    debug!("Create engine dir: {}", &engine_dir.display());

    // prepare js engine
    let engine_file = engine_dir.join("js_engine.wasm");
    let engine_wasm = if let Some(js_engine) = js_engine_path {
        if !PathBuf::from(&js_engine).exists() {
            bail!("File not found: {}", &js_engine);
        }
        std::fs::read(&js_engine).unwrap()
    } else {
        let engine_bytes = include_bytes!("../engine/quickjs.wasm");
        engine_bytes.to_vec()
    };
    debug!("Use engine_wasm len: {}", engine_wasm.len());
    debug!("Initialize target wasm file: {}", &target);
    std::fs::write(&engine_file, engine_wasm)?;

    // call wizer
    let src_content = std::fs::read(src_js_path)?;

    // wizer leaf_wasm_js.wasm -o leaf_wasm_js_wizer.wasm --allow-wasi --inherit-stdio=true --inherit-env=true
    let mut child = Command::new(cmd)
        .arg(&engine_file)
        .arg("-o")
        .arg(target)
        .arg("--allow-wasi")
        .arg("--inherit-stdio=true")
        .arg("--inherit-env=true")
        .arg("--wasm-bulk-memory=true")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to execute wizer child process");
    let mut stdin = child.stdin.take().expect("failed to get stdin");

    std::thread::spawn(move || {
        stdin
            .write_all(src_content.as_slice())
            .expect("failed to write to stdin");
    });

    let output = child
        .wait_with_output()
        .expect("failed to wait on wizer child process");
    if output.status.success() {
        // print output
        debug!(
            "Wizer output: \n{}",
            std::str::from_utf8(&output.stdout).unwrap()
        );
        info!("Wizer success: {}", &target);
    } else {
        panic!("Wizer failed: {output:?}");
    }

    Ok(())
}
