use anyhow::{anyhow, Result};
use land_core::metadata::Metadata;
use md5::{Digest, Md5};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tracing::{debug, info};
use walkdir::WalkDir;
use zip::write::FileOptions;

pub fn prepare(meta: &Metadata) -> Result<Vec<u8>> {
    let output = meta.get_output();
    // if output file is not exist, return error
    if !Path::new(&output).exists() {
        return Err(anyhow!("Wasm file not found: {}\n\tTry run 'land-cli build' to generate wasm file.", output));
    }

    let output_content = std::fs::read(&output)?;
    info!(
        "Wasm size: {}",
        bytesize::to_string(output_content.len() as u64, true),
    );

    // prepare temp zip file, name with random uuid
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("{}.zip", uuid::Uuid::new_v4()));
    debug!("Bundle temp file: {:?}", temp_file);
    let file = File::create(&temp_file)?;

    // prepare zip writer
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
    zip.start_file("bundle.wasm", options)?;
    zip.write_all(&output_content)?;

    // generate md5 hash for output content
    let mut hasher = Md5::new();
    hasher.update(&output_content);
    let result = hasher.finalize();
    let md5 = format!("{:x}", result);
    zip.start_file("bundle.wasm.md5sum", options)?;
    zip.write_all(md5.as_bytes())?;

    // add source dirs
    let mut buffer = Vec::new();
    let source_dirs = meta.get_source_dirs();
    for dir in source_dirs {
        let walkdir = WalkDir::new(dir);
        for entry in walkdir.into_iter() {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                debug!("Bundle add file: {:?}", path);
                zip.start_file(path.to_str().unwrap(), options)?;
                let mut f = File::open(path)?;
                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
                buffer.clear();
            } else {
                debug!("Bundle add dir: {:?}", path);
                zip.add_directory(path.to_str().unwrap(), options)?;
            }
        }
    }

    // flush zip file
    zip.finish()?;

    let content2 = std::fs::read(&temp_file)?;
    info!(
        "Bundle size: {}",
        bytesize::to_string(content2.len() as u64, true),
    );

    Ok(content2)
}
