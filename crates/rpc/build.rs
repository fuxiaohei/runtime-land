fn main() {
    build_proto().unwrap()
}

fn build_proto() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["./proto/lol-rpc.proto"], &["./proto"])?;
    Ok(())
}
