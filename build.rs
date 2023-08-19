use land_worker::compiler::{generate_guest, GuestGeneratorType};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wit/*.wit");
    println!("cargo:rerun-if-changed=wit/deps/http/*.wit");

    copy_wit_to_worker();
    build_wit_guest_code();
    copy_guest_code_to_sdk();
}

fn build_wit_guest_code() {
    let wit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("wit");

    // set world name to parse. in Wit file, it can provide multiple worlds
    let worlds = vec!["http-handler", "http-service"];

    // set exports to parse.  
    // For example. You need set StructName(eg: HttpImpl) for implmentation of HttpIncoming. 
    // then you write impl HttpIncoming for HttpImpl.
    let mut exports = HashMap::new();
    exports.insert(
        "land:http/http-incoming".to_string(),
        "HttpImpl".to_string(),
    );

    for world_name in worlds {
        let outputs = generate_guest(
            wit_dir.as_path(),
            Some(world_name.to_string()),
            GuestGeneratorType::Rust,
            exports.clone(),
        )
        .expect(format!("generate guest for {} failed", world_name).as_str());

        // for range outputs, write content with key name
        for (name, content) in outputs.iter() {
            let target_rs = wit_dir.join(Path::new(name));
            std::fs::write(target_rs, content).unwrap();
        }
    }
}

fn copy_wit_to_worker() {
    let wit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("wit");
    copy_recursively(wit_dir, PathBuf::from("crates/worker/wit")).unwrap()
}

fn copy_recursively(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn copy_guest_code_to_sdk() {
    let wit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("wit");
    let expects = vec![
        ("http_handler.rs", "crates/sdk-macro/src/http_handler.rs"),
        ("http_service.rs", "crates/sdk/src/http_service.rs"),
    ];
    // copy expects
    for (source, target) in expects.iter() {
        let source_path = wit_dir.join(Path::new(source));
        let target_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(target);
        std::fs::copy(source_path, target_path).unwrap();
    }
}
