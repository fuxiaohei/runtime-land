use crate::deploy;
use clap::Args;
use land_core::metadata::{Metadata, DEFAULT_FILE as DEFAULT_METADATA_FILE};
use path_slash::PathBufExt as _;
use std::path::{Path, PathBuf};
use tracing::{debug, debug_span, info, Instrument};

/// Command Init
#[derive(Args, Debug)]
pub struct Init {
    /// The name of the project
    pub name: String,
    /// The template to use
    #[clap(long, default_value("rust-basic"))]
    pub template: Option<String>,
}

impl Init {
    pub async fn run(&self) {
        debug!("Init: {self:?}");

        // create dir by name
        if !Path::new(&self.name).exists() {
            std::fs::create_dir(&self.name).unwrap();
            info!("Created dir: {}", &self.name)
        }

        // create metadata
        let meta = self.create_metadata();
        // create project
        self.create_project(meta);
        info!("Created project {} success", &self.name);
    }

    fn create_metadata(&self) -> Metadata {
        let metadata_file =
            PathBuf::from(&self.template.as_ref().unwrap()).join(DEFAULT_METADATA_FILE);

        let meta = crate::embed::TemplateAssets::get(metadata_file.to_str().unwrap());
        if meta.is_none() {
            panic!("Template {} is not valid", &self.template.as_ref().unwrap());
        }
        let mut meta = Metadata::from_binary(&meta.unwrap().data).unwrap();
        meta.name = self.name.clone();
        meta.build = None;
        let metadata_file = PathBuf::from(&self.name).join(DEFAULT_METADATA_FILE);
        meta.to_file(metadata_file.to_str().unwrap()).unwrap();
        info!(
            "Created metadata: {:?}, name: {:?}",
            metadata_file, self.name,
        );
        meta
    }

    fn create_project(&self, meta: Metadata) {
        // if rust project, copy Cargo.toml
        if meta.language == "rust" {
            self.create_cargo_toml();
        }

        // create src dir
        let src_dir = Path::new(&self.name).join("src");
        std::fs::create_dir_all(src_dir.parent().unwrap()).unwrap();

        // copy src files
        let tpl_dir = Path::new(&self.template.as_ref().unwrap())
            .join("src")
            .to_slash_lossy()
            .to_string();
        crate::embed::TemplateAssets::iter().for_each(|t| {
            if t.starts_with(tpl_dir.clone().as_str()) {
                let src_path = Path::new(t.as_ref()).strip_prefix(tpl_dir.clone()).unwrap();
                let file = crate::embed::TemplateAssets::get(t.as_ref()).unwrap();
                let content = std::str::from_utf8(&file.data).unwrap().to_string();
                let target_path = src_dir.join(src_path);
                debug!("Created src: {:?}, {:?}", src_path, target_path);
                std::fs::create_dir_all(target_path.parent().unwrap()).unwrap();
                std::fs::write(target_path, content).unwrap();
            }
        });
    }

    fn create_cargo_toml(&self) {
        let template = self.template.as_ref().unwrap();
        let name = self.name.as_str();

        let toml_file = PathBuf::from(template).join("Cargo.toml.txt");
        let toml_data = crate::embed::TemplateAssets::get(toml_file.to_str().unwrap());
        if toml_data.is_none() {
            panic!(
                "Template {} is not valid with rust Cargo.toml",
                &self.template.as_ref().unwrap()
            );
        }
        let target_file = PathBuf::from(&self.name).join("Cargo.toml");

        // replace cargo toml to correct deps
        let mut content = std::str::from_utf8(&toml_data.unwrap().data)
            .unwrap()
            .to_string();
        content = content.replace(template, name);
        std::fs::write(target_file.to_str().unwrap(), content).unwrap();

        info!("Created Cargo.toml: {:?}", target_file);
    }
}

/// Command Build
#[derive(Args, Debug)]
pub struct Build {
    /// Set js engine wasm file
    #[clap(long)]
    pub js_engine: Option<String>,
}

impl Build {
    pub async fn run(&self) {
        debug!("Build: {self:?}");

        // read metadata from file
        let meta = Metadata::from_file(DEFAULT_METADATA_FILE).expect("Project meta.toml not found");
        let arch = meta.get_arch();
        info!("Build arch: {}", arch);

        let target = meta.get_target();
        info!("Build target: {}", target);

        // call cargo to build wasm
        match meta.language.as_str() {
            "rust" => {
                land_worker::compiler::compile_rust(&arch, &target).expect("Build failed");
            }
            "js" | "javascript" => {
                land_worker::compiler::compile_js(&target, "src/index.js", self.js_engine.clone())
                    .expect("Build failed");
            }
            _ => {
                panic!("Unsupported language: {}", meta.language);
            }
        }

        // convert wasm module to component
        let output = meta.get_output();
        land_worker::compiler::convert_component(&target, Some(output)).expect("Convert failed");
    }
}

/// Command Serve
#[derive(Args, Debug)]
pub struct Serve {
    /// The port to listen on
    #[clap(long, default_value("127.0.0.1:8888"))]
    pub addr: Option<std::net::SocketAddr>,
}

impl Serve {
    pub async fn run(&self) {
        debug!("Serve: {self:?}");

        let meta = Metadata::from_file(DEFAULT_METADATA_FILE).expect("Project meta.toml not found");
        debug!("Meta: {meta:?}");

        // start server
        let addr = self.addr.unwrap();
        crate::server::start(addr, &meta)
            .instrument(debug_span!("[Server]"))
            .await
            .unwrap();
    }
}

/// Command Deploy
#[derive(Args, Debug)]
pub struct Deploy {
    /// The token
    #[clap(long, env("TOKEN"))]
    pub token: String,
    /// Publish this deployment to production
    #[clap(long, default_value("false"))]
    pub production: bool,
    /// The api address
    #[clap(
        long,
        env("API_ADDR"),
        default_value("https://center-api.runtime.land")
    )]
    pub api_addr: Option<String>,
    /// The project name override meta.toml
    #[clap(long)]
    pub project: Option<String>,
}

impl Deploy {
    pub async fn run(&self) {
        debug!("Deploy: {self:?}");
        let addr = self.api_addr.as_ref().unwrap();

        let meta = Metadata::from_file(DEFAULT_METADATA_FILE).expect("Project meta.toml not found");
        debug!("Meta: {meta:?}");

        let project = deploy::load_project(self.project.clone(), &meta, addr, &self.token)
            .await
            .expect("Load project failed");
        info!("Project name: {}", project.name);

        let output = meta.get_output();
        let content = std::fs::read(output).expect("Read compiled target failed");

        let deployment = deploy::create_deployment(
            project,
            content,
            String::from("application/wasm"),
            addr,
            &self.token,
        )
        .await
        .expect("Create deployment failed");

        if self.production {
            let deployment = deploy::publish_deployment(deployment.uuid, addr, &self.token)
                .await
                .expect("Publish deployment failed");
            info!("Deployment url: \t\n{}", deployment.prod_url);
            return;
        }

        info!("Deployment url: \t\n{}", deployment.domain_url);
    }
}
