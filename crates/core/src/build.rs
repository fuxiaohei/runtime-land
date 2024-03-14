use anyhow::Result;

/// js compile to js to wasm component
pub fn js(src: &str, target: &str) -> Result<()> {
    // compile js to wizer
    land_wit::compile_js(src, target, None)?;
    compile(target)
}

/// compile wasm to wasm component
pub fn compile(target: &str) -> Result<()> {
    // use wasm-opt to optimize wasm if wasm-opt exists
    if let Some(op) = land_wit::optimize(target)? {
        std::fs::rename(op, target)?;
    }

    // encode wasm module to component
    land_wit::encode_component(target, target)?;

    // check target exists
    if !std::path::Path::new(target).exists() {
        return Err(anyhow::anyhow!(
            "Build target '{}' does not exist!",
            &target,
        ));
    }
    Ok(())
}
