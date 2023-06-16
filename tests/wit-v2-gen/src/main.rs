use anyhow::Result;
use std::path::Path;
use wit_bindgen_core::wit_parser::Resolve;
use wit_bindgen_core::{Files, WorldGenerator};

fn create_generator() -> Result<Box<dyn WorldGenerator>> {
    let opts = wit_bindgen_rust::Opts {
        macro_export: true,
        rustfmt: true,
        ..Default::default()
    };
    let builder = opts.build();
    Ok(builder)
}

fn main() {
    let mut generator = create_generator().unwrap();

    let mut resolve = Resolve::default();
    let path = Path::new("./wit");
    let pkg = resolve.push_dir(path).unwrap().0;
    let worlds = vec!["http-handler", "http-service"];

    for world_name in worlds {
        let world = resolve.select_world(pkg, Some(world_name)).unwrap();

        let mut files = Files::default();
        generator.generate(&resolve, world, &mut files);

        for (name, contents) in files.iter() {
            println!(
                "{}: {}: {}",
                world_name,
                name,
                String::from_utf8_lossy(contents).len()
            );
        }
    }
}
