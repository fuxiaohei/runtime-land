use anyhow::Result;
use std::collections::HashMap;
use std::process::Command;
use tracing::{error, info};
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::EnvFilter;

mod case;

fn init_tracing() {
    if std::env::var("RUST_LOG").ok().is_none() {
        if cfg!(debug_assertions) {
            std::env::set_var("RUST_LOG", "debug")
        } else {
            std::env::set_var("RUST_LOG", "info")
        }
    }

    let timer = OffsetTime::new(
        time::UtcOffset::from_hms(8, 0, 0).unwrap(),
        time::format_description::parse(
            "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]",
        )
        .unwrap(),
    );

    tracing_subscriber::fmt()
        .with_timer(timer)
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();
}

fn get_cli_path() -> String {
    // cli path from CLI_PATH env, if not exist, use './land-cli'
    let cli_path = std::env::var("CLI_PATH").unwrap_or(String::from("./land-cli"));
    // windows use 'land-cli.exe'
    #[cfg(windows)]
    if !cli_path.ends_with(".exe") {
        return format!("{}.exe", cli_path);
    }
    // if cli_path is absolute path, return it
    if cli_path.starts_with('/') || cli_path.starts_with('\\') {
        return cli_path;
    }
    // if cli_path is relative path, use current dir as prefix
    let cwd = std::env::current_dir().unwrap();
    cwd.join(cli_path).to_str().unwrap().to_string()
}

fn get_runtime_path() -> String {
    // runtime path from RUNTIME_PATH env, if not exist, use './land-runtime'
    let runtime_path = std::env::var("RUNTIME_PATH").unwrap_or(String::from("./land-runtime"));
    // windows use 'land-runtime.exe'
    #[cfg(windows)]
    if !runtime_path.ends_with(".exe") {
        return format!("{}.exe", runtime_path);
    }
    // if runtime_path is absolute path, return it
    if runtime_path.starts_with('/') || runtime_path.starts_with('\\') {
        return runtime_path;
    }
    // if runtime_path is relative path, use current dir as prefix
    let cwd = std::env::current_dir().unwrap();
    cwd.join(runtime_path).to_str().unwrap().to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    // get cli and runtime path
    let cli_path = get_cli_path();
    let runtime_path = get_runtime_path();
    info!("cli_path: {}", cli_path);
    info!("runtime_path: {}", runtime_path);

    // mkdir wasm-dist if not exist
    let wasm_dist = std::env::current_dir().unwrap().join("wasm-dist");
    if !wasm_dist.exists() {
        std::fs::create_dir(&wasm_dist).unwrap();
        info!("create wasm-dist dir: {:?}", wasm_dist);
    }

    let test_templates = ["rust-basic", "rust-fetch", "rust-router", "js-basic"];
    let mut target_files = HashMap::new();
    for name in test_templates.iter() {
        // build template project
        let target = build_template_project(name, &cli_path, wasm_dist.to_str().unwrap()).unwrap();
        info!("build template project success: {}", target);
        // push target file to target_files
        target_files.insert(name, target);
    }

    // start runtime server in spawn
    let mut runtime_server = Command::new(&runtime_path)
        .env("FS_PATH", &wasm_dist)
        .env("EDGE_SYNC_ENABLED", "false")
        .spawn()
        .expect("failed to start runtime server");
    info!("runtime server started");

    for (name, target) in target_files.iter() {
        // test template runtime
        match test_template_runtime(name, target).await {
            Ok(_) => {
                info!("test template runtime success: {}", name)
            }
            Err(e) => {
                error!("test template runtime failed: {}, {}", name, e);
                runtime_server.kill().unwrap();
                return Err(e);
            }
        }
    }

    runtime_server.kill().unwrap();
    Ok(())
}

fn build_template_project(name: &str, cli_path: &str, dist_dir: &str) -> Result<String> {
    let _span = tracing::info_span!("build_template_project", name).entered();

    let envs: HashMap<String, String> = std::env::vars().collect();
    // call cli_path binary to create project with template name, project name as name-demo
    let project_name = format!("{}-demo", name);
    let output = Command::new(cli_path)
        .envs(&envs)
        .arg("init")
        .arg(&project_name)
        .arg("--template")
        .arg(name)
        .output()
        .expect("failed to execute init command");
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "create project failed: {}",
            String::from_utf8(output.stderr)?
        ));
    }
    info!(
        "create project success: {},\n{}",
        project_name,
        String::from_utf8(output.stdout)?
    );

    let cwd = std::env::current_dir()?;
    let dir_name = cwd.join(&project_name);
    info!("workdir: {:?}", dir_name);

    // use dir_name as workdir , call cli_path build command
    let output = Command::new(cli_path)
        .envs(&envs)
        .current_dir(&dir_name)
        .arg("build")
        .output()
        .expect("failed to execute build command");
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "build project failed: {}",
            String::from_utf8(output.stderr)?
        ));
    }
    info!(
        "build project success: {},\n{}",
        project_name,
        String::from_utf8(output.stdout)?
    );

    // target wasm32-wasi component wasm file
    let target_name = format!("{}.component.wasm", project_name.replace('-', "_"));
    let target_file = dir_name
        .join("target/wasm32-wasi/release")
        .join(target_name);
    if !target_file.exists() {
        return Err(anyhow::anyhow!("target file not exist: {:?}", target_file));
    }
    info!("target_file: {:?}", target_file);
    // cp target wasm file to wasm-dist dir with name as project_name
    let dist_file = format!("{}/{}.wasm", dist_dir, project_name);
    std::fs::copy(&target_file, &dist_file)?;
    info!("dist_file: {:?}", dist_file);

    Ok(format!("{}.wasm", project_name))
}

async fn test_template_runtime(name: &str, wasm: &str) -> Result<()> {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    match name {
        "rust-basic" => case::test_rust_basic(wasm).await?,
        "rust-fetch" => case::test_rust_fetch(wasm).await?,
        "rust-router" => case::test_rust_router(wasm).await?,
        _ => return Err(anyhow::anyhow!("unknown template name: {}", name)),
    }

    Ok(())
}
