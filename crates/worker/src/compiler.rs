use anyhow::anyhow;
use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tracing::{debug, info};
use which::which;
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
    fn create_generator(
        &self,
        gen_exports: HashMap<String, String>,
    ) -> Result<Box<dyn WorldGenerator>> {
        let mut exports = HashMap::new();
        for (name, content) in gen_exports.iter() {
            exports.insert(
                wit_bindgen_rust::ExportKey::Name(name.to_string()),
                content.to_string(),
            );
        }
        match self {
            GuestGeneratorType::Rust => {
                let opts = wit_bindgen_rust::Opts {
                    exports,
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
    gen_exports: HashMap<String, String>,
) -> Result<HashMap<String, String>> {
    let mut generator = t.create_generator(gen_exports)?;

    let mut resolve = Resolve::default();
    let pkg = resolve.push_dir(wit_dir)?.0;

    let mut output_maps = HashMap::new();
    let mut files = Files::default();
    let world = resolve.select_world(pkg, world.as_deref())?;
    generator.generate(&resolve, world, &mut files)?;
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

/// convert_component is used to convert wasm module to component
pub fn convert_component(path: &str, output: Option<String>, language: String) -> Result<()> {
    let path = if language == "js" || language == "javascript" {
        // js sdk wasm is already optimized
        path.to_string()
    } else {
        optimize_module(path)?.unwrap_or(path.to_string())
    };
    debug!("Convert component, {path}");
    let file_bytes = std::fs::read(&path).expect("parse wasm file error");
    let wasi_adapter = include_bytes!("../engine/wasi_snapshot_preview1.reactor.wasm");

    let component = ComponentEncoder::default()
        .module(&file_bytes)
        .expect("Pull custom sections from module")
        .validate(true)
        .adapter("wasi_snapshot_preview1", wasi_adapter)
        .expect("Add adapter to component")
        .encode()
        .expect("Encode component");

    let output = output.unwrap_or(path);
    std::fs::write(&output, component).expect("Write component file error");
    info!("Convert component success, {}", &output);

    // get output file size
    let size = std::fs::metadata(&output)?.len();
    info!("Component size: {} KB", size / 1024);
    Ok(())
}

fn optimize_module(path: &str) -> Result<Option<String>> {
    let mut cmd = which("wasm-opt");
    if cmd.is_err() {
        // find wasm-opt binary in current exe directroy ./wasm-opt-bin/wasm
        let exe_path = std::env::current_exe()?;
        let file = exe_path.parent().unwrap().join("wasm-opt-bin/wasm-opt");

        #[cfg(target_os = "windows")]
        let file = file.with_extension("exe");

        if file.exists() {
            cmd = Ok(file);
        } else {
            debug!("Wasm-opt not found, skip optimize");
            return Ok(None);
        }
    }
    debug!("Optimize module: {}", path);
    let target = path.replace(".wasm", ".opt.wasm");
    let cmd = cmd.unwrap();
    let child = Command::new(cmd)
        .arg("-O")
        .arg("--strip-debug")
        .arg("-o")
        .arg(&target)
        .arg(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to execute wasm-opt child process");

    let output = child
        .wait_with_output()
        .expect("failed to wait on wizer child process");
    if output.status.success() {
        // print output
        debug!(
            "Wasm-opt output: \n{}",
            std::str::from_utf8(&output.stdout).unwrap()
        );
        info!("Wasm-opt success: {}", &target);
    } else {
        panic!("Wasm-opt failed: {output:?}");
    }
    info!("Optimize module success: {}", &target);
    Ok(Some(target))
}

pub fn compile_js(target: &str, src_js_path: &str, js_engine_path: Option<String>) -> Result<()> {
    // js need wizer command
    let mut cmd = which("wizer");
    if cmd.is_err() {
        let exe_path = std::env::current_exe()?;
        let file = exe_path.parent().unwrap().join("wizer-bin/wizer");

        #[cfg(target_os = "windows")]
        let file = file.with_extension("exe");

        if file.exists() {
            cmd = Ok(file);
        } else {
            return Err(anyhow::anyhow!(
            "Wizer not found \n\tplease install wizer first: \n\tcargo install wizer --all-features\n\tmore infomation see: https://github.com/bytecodealliance/wizer"
        ));
        }
    }

    let cmd = cmd.unwrap();
    // create dir
    let engine_dir = Path::new(&target).parent().unwrap();
    std::fs::create_dir_all(engine_dir).expect("create dir failed");
    debug!("Create engine dir: {}", &engine_dir.display());

    // prepare js engine
    let engine_file = engine_dir.join("js_engine.wasm");
    let engine_wasm = if let Some(js_engine) = js_engine_path {
        if !PathBuf::from(&js_engine).exists() {
            anyhow::bail!("File not found: {}", &js_engine);
        }
        std::fs::read(&js_engine).unwrap()
    } else {
        let engine_bytes = include_bytes!("../engine/land-js-sdk.wasm");
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
