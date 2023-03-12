fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    //println!("cargo:rerun-if-changed=wit/*.wit");

    let mut config = vergen::Config::default();
    *config.git_mut().sha_kind_mut() = vergen::ShaKind::Short;
    *config.git_mut().commit_timestamp_kind_mut() = vergen::TimestampKind::DateOnly;
    vergen::vergen(config).expect("failed to extract build information");

    build_proto().unwrap()
}

fn build_proto() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("./proto/moni.proto")?;
    Ok(())
}
