use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use wit_bindgen_core::{Files, WorldGenerator};
use wit_parser::{Resolve, UnresolvedPackage};

/// GuestGeneratorType is the type of the guest generator.
pub enum GuestGeneratorType {
    Rust,
    Js,
    TinyGo,
}

impl GuestGeneratorType {
    /// create generator by type
    fn create_generator(&self) -> Result<Box<dyn WorldGenerator>> {
        match self {
            GuestGeneratorType::Rust => {
                let opts = wit_bindgen_rust::Opts {
                    macro_export: true,
                    rustfmt: true,
                    ..Default::default()
                };
                let builder = opts.build();
                Ok(builder)
            }
            _ => Err(anyhow!("unsupport guest generator")),
        }
    }
}

/// parse wit file and return world id
pub fn generate_guest(
    wit: PathBuf,
    world: Option<String>,
    t: GuestGeneratorType,
) -> Result<HashMap<String, String>> {
    let mut generator = t.create_generator()?;

    let mut resolve = Resolve::default();
    let pkg = if wit.is_dir() {
        resolve.push_dir(&wit)?.0
    } else {
        resolve.push(UnresolvedPackage::parse_file(&wit)?, &Default::default())?
    };

    let mut output_maps = HashMap::new();
    let mut files = Files::default();
    let world = resolve.select_world(pkg, world.as_deref())?;
    generator.generate(&resolve, world, &mut files);
    for (name, contents) in files.iter() {
        output_maps.insert(
            name.to_string(),
            String::from_utf8_lossy(contents).to_string(),
        );
    }
    Ok(output_maps)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use super::generate_guest;

    #[test]
    fn test_compile() {
        let wit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../wit");
        let outputs = generate_guest(
            wit_dir,
            Some(String::from("http-interface")),
            super::GuestGeneratorType::Rust,
        )
        .unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs.contains_key("http_interface.rs"), true);
    }
}
