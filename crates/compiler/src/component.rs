use crate::find_cmd;
use anyhow::Result;
use std::process::{Command, Stdio};
use wit_component::ComponentEncoder;

pub fn convert(path: &str, lang: &str) -> Result<()> {
    if lang == "js" || lang == "javascript" {
        // TODO: convert wasm component for javascript
        return Ok(());
    }
    let mut target = path.to_string();
    if let Some(opt_wasm) = optimize(path)? {
        target = opt_wasm;
    }

    let file_bytes = std::fs::read(&target).expect("read wasm file error");
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
    std::fs::write(&output, component).expect("Write component file error");
    std::fs::rename(&output, path).expect("Rename component file error");
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
    Ok(Some(target))
}
