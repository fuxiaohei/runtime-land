fn main() {
    build_proto().unwrap()
}

fn build_proto() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("./proto/moni-rpc.proto")?;
    Ok(())
}
