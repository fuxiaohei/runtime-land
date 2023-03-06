use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wit/*.wit");

    build_wit_guest_code();
}

fn build_wit_guest_code() {
    // loop wit directory, find .wit files , check same name as .rs file, if not, generate it
    let wit_dir = Path::new("./wit");
    let wit_files = wit_dir.read_dir().unwrap();
    for wit_file in wit_files {
        let wit_file_path = wit_file.unwrap().path();
        if !wit_file_path.is_file() {
            continue;
        }
        if wit_file_path.extension().unwrap() != "wit" {
            continue;
        }
        let outputs = moni_runtime::generate_guest(
            wit_file_path.to_str().unwrap(),
            None,
            moni_runtime::GuestGeneratorType::Rust,
        )
        .unwrap();
        outputs.iter().for_each(|(path, content)| {
            let target_rs = wit_dir.join(path);
            std::fs::write(target_rs, content).unwrap();
        });
    }
}
