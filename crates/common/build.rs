use std::error::Error;
use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    // Emit the instructions
    EmitBuilder::builder()
        .all_build()
        .all_cargo()
        .git_sha(true)
        .all_rustc()
        .git_commit_date()
        .emit()?;
    Ok(())
}
