use moni_runtime::{generate_guest, GuestGeneratorType};
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wit/*.wit");

    build_wit_guest_code();
}

fn build_wit_guest_code() {
    let wit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("wit");
    let worlds = vec![("http-interface", "http_interface.rs")];
    for world in worlds {
        let outputs = generate_guest(
            wit_dir.clone(),
            Some(String::from(world.0)),
            GuestGeneratorType::Rust,
        )
        .unwrap();
        let content = outputs.get(world.1).unwrap();
        let target_rs = wit_dir.join(Path::new(world.1));
        std::fs::write(target_rs, content).unwrap();
    }
}
