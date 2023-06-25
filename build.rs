use land_runtime::{generate_guest, GuestGeneratorType};
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wit/*.wit");

    build_wit_guest_code();
    copy_guest_code_to_crates();
}

fn build_wit_guest_code() {
    let wit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("wit");

    let worlds = vec!["http-handler", "http-service"];
    for world_name in worlds {
        let outputs = generate_guest(
            wit_dir.as_path(),
            Some(world_name.to_string()),
            GuestGeneratorType::Rust,
        )
        .unwrap();

        // for range outputs, write content with key name
        for (name, content) in outputs.iter() {
            let target_rs = wit_dir.join(Path::new(name));
            std::fs::write(target_rs, content).unwrap();
        }
    }
}

fn copy_guest_code_to_crates() {
    let wit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("wit");
    let expects = vec![
        ("http_handler.rs", "crates/sdk/macro/src/http_handler.rs"),
        ("http_service.rs", "crates/sdk/src/wit/http_service.rs"),
    ];
    // copy expects
    for (source, target) in expects.iter() {
        let source_path = wit_dir.join(Path::new(source));
        let target_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(target);
        std::fs::copy(source_path, target_path).unwrap();
    }
}
