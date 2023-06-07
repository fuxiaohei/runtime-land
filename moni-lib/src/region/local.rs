use anyhow::Result;
use envconfig::Envconfig;
use once_cell::sync::OnceCell;
use opendal::Operator;
use tracing::debug;

// LOCAL_REGION is the local region operator
pub static LOCAL_REGION: OnceCell<Operator> = OnceCell::new();

#[derive(Envconfig, Debug)]
pub struct LocalConfig {
    #[envconfig(from = "LOCAL_REGION_REDIS_ADDR", default = "127.0.0.1:6379")]
    pub redis_addr: String,
    #[envconfig(from = "LOCAL_REGION_REDIS_PASSWORD", default = "")]
    pub redis_password: String,
    #[envconfig(from = "LOCAL_REGION_REDIS_DB", default = "0")]
    pub redis_db: i64,
    #[envconfig(from = "LOCAL_REGION_RUNTIME", default = "127.0.0.1:38999")]
    pub runtime: String,
}

// init initializes the local region
pub async fn init() -> Result<()> {
    let cfg = LocalConfig::init_from_env()?;
    let mut builder = opendal::services::Redis::default();
    builder
        .endpoint(&cfg.redis_addr)
        .password(&cfg.redis_password)
        .db(cfg.redis_db);
    let op = Operator::new(builder)?.finish();
    op.write("moni-serverless", "setup").await?;
    LOCAL_REGION.set(op).unwrap();

    // register local runtime service to traefik
    register_runtime(&cfg.runtime).await?;

    Ok(())
}

async fn register_runtime(runtimes: &str) -> Result<()> {
    let values = runtimes.split(',').collect::<Vec<&str>>();
    println!("values: {:?}", values);

    let op = LOCAL_REGION.get().unwrap();
    for (i, x) in values.iter().enumerate() {
        let svc_key = format!(
            "traefik/http/services/moni-runtime/loadbalancer/servers/{}/url",
            i
        );
        op.write(&svc_key, String::from(*x)).await?;
        debug!("register local runtime: {} : {}", x, svc_key);
    }

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
    commands.push((
        format!("traefik/http/routers/{}/service", deploy_uuid),
        String::from("moni-runtime"),
    ));

    // add custom-header for moni-wasm
    commands.push((
        format!(
            "traefik/http/middlewares/m-{}/headers/customrequestheaders/x-moni-wasm",
            deploy_uuid
        ),
        deployment.storage_path,
    ));
    commands.push((
        format!(
            "traefik/http/middlewares/m-{}/headers/customrequestheaders/x-moni-uuid",
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
