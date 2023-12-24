use crate::find_cmd;
use anyhow::Result;
use std::{
    io::Write,
    process::{Command, Stdio},
};
use tracing::debug;
use wit_component::ComponentEncoder;

pub fn convert(path: &str, lang: &str, js_engine: Option<String>) -> Result<()> {
    if lang == "js" || lang == "javascript" {
        let js_wasm = compile_js(path, js_engine)?; // this wasm is optimized by wasm-opt
        let output_path = land_common::js_real_target_path(path);
        convert_inner(&output_path, &js_wasm)?;
        return Ok(());
    }
    let mut target = path.to_string();
    if let Some(opt_wasm) = optimize(path)? {
        target = opt_wasm;
    }
    convert_inner(path, &target)
}

fn compile_js(src_path: &str, js_engine: Option<String>) -> Result<String> {
    debug!("compile js file: {}", src_path);
    let cmd = find_cmd("wizer")?;
    let dir = std::path::Path::new(src_path).parent().unwrap();
    let js_engine_file = dir.join("js_engine.wasm");

    // copy js_engine.wasm to path dir
    let js_engine_bytes = if let Some(js_engine) = js_engine {
        std::fs::read(js_engine)?
    } else {
        include_bytes!("../engine/js_engine.wasm").to_vec()
    };
    debug!("js_engine_bytes: {}", js_engine_bytes.len());
    std::fs::write(&js_engine_file, js_engine_bytes)?;

    let src_content = std::fs::read(src_path)?;
    let target_path = src_path.replace(".js", ".wizer.wasm");

    // call wizer to compile js to wasm
    // wizer js_engine.wasm -o {path}.wasm --allow-wasi --inherit-stdio=true --inherit-env=true
    let mut child = Command::new(cmd)
        .arg(&js_engine_file)
        .arg("-o")
        .arg(&target_path)
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
    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        return Err(anyhow::anyhow!(err));
    }
    // print output
    debug!(
        "wizer output: \n{}",
        std::str::from_utf8(&output.stdout).unwrap()
    );
    debug!("wizer success, from {} to {}", src_path, target_path);
    let _ = std::fs::remove_file(&js_engine_file);
    Ok(target_path)
}

fn convert_inner(path: &str, target: &str) -> Result<()> {
    let file_bytes = std::fs::read(target)?;
    let wasi_adapter = include_bytes!("../engine/wasi_snapshot_preview1.reactor.wasm");
    let component = ComponentEncoder::default()
        .module(&file_bytes)
        .expect("Pull custom sections from module")
        .validate(true)
        .adapter("wasi_snapshot_preview1", wasi_adapter)
        .expect("Add adapter to component")
        .encode()
        .expect("Encode component");
    let output = target.replace(".wasm", ".component.wasm");
    std::fs::write(&output, component)?;
    std::fs::rename(&output, path)?;
    debug!("convert success, from {} to {}", target, path);
    Ok(())
}

/// optimize wasm component
pub fn optimize(path: &str) -> Result<Option<String>> {
    let cmd = match find_cmd("wasm-opt") {
        Ok(cmd) => cmd,
        Err(_err) => {
            return Ok(None);
        }
    };
    let target = path.replace(".wasm", ".opt.wasm");
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
        .expect("failed to wait on wasm-opt child process");
    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        return Err(anyhow::anyhow!(err));
    }
    debug!("wasm-opt success, from {} to {}", path, target);
    Ok(Some(target))
}
