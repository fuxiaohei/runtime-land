use anyhow::Result;
use envconfig::Envconfig;
use once_cell::sync::OnceCell;
use opendal::Operator;
use tracing::{debug, info};

// LOCAL_REGION is the local region operator
pub static LOCAL_REGION: OnceCell<Operator> = OnceCell::new();

// LOCAL_REGION_RUNTIME is the local region runtime service name
pub static LOCAL_REGION_RUNTIME: OnceCell<String> = OnceCell::new();

#[derive(Envconfig, Debug)]
pub struct LocalConfig {
    #[envconfig(from = "LOCAL_REGION_ENABLED", default = "false")]
    pub enable: bool,
    #[envconfig(from = "LOCAL_REGION_REDIS_ADDR", default = "127.0.0.1:6379")]
    pub redis_addr: String,
    #[envconfig(from = "LOCAL_REGION_REDIS_PASSWORD", default = "")]
    pub redis_password: String,
    #[envconfig(from = "LOCAL_REGION_REDIS_DB", default = "0")]
    pub redis_db: i64,
    // runtime is the runtime service name in docker-compose.yml
    #[envconfig(from = "LOCAL_REGION_RUNTIME", default = "land-runtime")]
    pub runtime: String,
}

// init initializes the local region
#[tracing::instrument(name = "[LOCAL_REGION]")]
pub async fn init() -> Result<()> {
    let cfg = LocalConfig::init_from_env()?;
    if !cfg.enable {
        info!("Disabled");
        return Ok(());
    }
    let mut builder = opendal::services::Redis::default();
    builder
        .endpoint(&cfg.redis_addr)
        .password(&cfg.redis_password)
        .db(cfg.redis_db);

    // write once as ping to validate redis connection
    let op = Operator::new(builder)?.finish();
    op.write("land-serverless", "setup").await?;

    LOCAL_REGION.set(op).unwrap();
    LOCAL_REGION_RUNTIME.set(cfg.runtime).unwrap();

    Ok(())
}

pub async fn deploy(deploy_id: u32, mut deploy_uuid: String, is_production: bool) -> Result<()> {
    let deployment = crate::dao::deployment::find(deploy_id as i32, deploy_uuid.clone()).await?;
    if deployment.is_none() {
        return Err(anyhow::anyhow!("deployment not found"));
    }
    let deployment = deployment.unwrap();
    let mut commands: Vec<(String, String)> = vec![];

    if is_production {
        // if is production, set deploy_uuid to PROD-project-uuid
        let project =
            crate::dao::project::find_by_id(deployment.owner_id, deployment.project_id).await?;
        if project.is_none() {
            return Err(anyhow::anyhow!("project not found"));
        }
        let project = project.unwrap();
        deploy_uuid = format!("PROD-{}", project.uuid);
    }

    // generate Host(domain) url
    let prod_domain = crate::PROD_DOMAIN.get().unwrap().clone();
    let mut domain = format!("{}.{}", deployment.domain, prod_domain);
    if is_production {
        domain = format!("{}.{}", deployment.prod_domain, prod_domain);
    }
    commands.push((
        format!("traefik/http/routers/{}/rule", deploy_uuid),
        format!("Host(`{}`)", domain),
    ));

    // set routes backend service
    let service_name = LOCAL_REGION_RUNTIME.get().unwrap().clone();
    commands.push((
        format!("traefik/http/routers/{}/service", deploy_uuid),
        service_name,
    ));

    // add custom-header for land-wasm
    commands.push((
        format!(
            "traefik/http/middlewares/m-{}/headers/customrequestheaders/x-land-wasm",
            deploy_uuid
        ),
        deployment.storage_path,
    ));
    commands.push((
        format!(
            "traefik/http/middlewares/m-{}/headers/customrequestheaders/x-land-uuid",
            deploy_uuid
        ),
        deploy_uuid.clone(),
    ));

    // add custom-header to middleware
    commands.push((
        format!("traefik/http/routers/{}/middlewares/0", &deploy_uuid),
        format!("m-{}", deploy_uuid),
    ));

    let operator = LOCAL_REGION.get().unwrap();
    for (k, v) in commands {
        debug!("deploy: {} : {}", k, v);
        operator.write(&k, v.clone()).await?;
    }

    Ok(())
}
