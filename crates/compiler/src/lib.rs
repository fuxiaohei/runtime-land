use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::Path;
use wit_bindgen_core::wit_parser::Resolve;
use wit_bindgen_core::{Files, WorldGenerator};

/// GuestGeneratorType is the type of the guest generator.
pub enum GuestGeneratorType {
    Rust,
    TinyGo,
}

impl GuestGeneratorType {
    /// create generator by type
    fn create_generator(
        &self,
        gen_exports: HashMap<String, String>,
    ) -> Result<Box<dyn WorldGenerator>> {
        let mut exports = HashMap::new();
        for (name, content) in gen_exports.iter() {
            exports.insert(
                wit_bindgen_rust::ExportKey::Name(name.to_string()),
                content.to_string(),
            );
        }
        match self {
            GuestGeneratorType::Rust => {
                let opts = wit_bindgen_rust::Opts {
                    exports,
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
    wit_dir: &Path,
    world: Option<String>,
    t: GuestGeneratorType,
    gen_exports: HashMap<String, String>,
) -> Result<HashMap<String, String>> {
    let mut generator = t.create_generator(gen_exports)?;

    let mut resolve = Resolve::default();
    let pkg = resolve.push_dir(wit_dir)?.0;

    let mut output_maps = HashMap::new();
    let mut files = Files::default();
    let world = resolve.select_world(pkg, world.as_deref())?;
    generator.generate(&resolve, world, &mut files)?;
    for (name, contents) in files.iter() {
        output_maps.insert(
            name.to_string(),
            String::from_utf8_lossy(contents).to_string(),
        );
    }
    Ok(output_maps)
}
