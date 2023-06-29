use super::RegionTrait;
use anyhow::Result;
use axum::async_trait;
use envconfig::Envconfig;
use opendal::Operator;
use tracing::debug;

#[derive(Envconfig, Debug)]
pub struct LocalConfig {
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

pub struct LocalRegion {
    operator: Option<Operator>,
    service_name: String,
}

impl LocalRegion {
    async fn deploy_inner(&self, uuid: String, domain: String, storage_path: String) -> Result<()> {
        //  redis rules for traefik proxy, set key=value
        let mut commands: Vec<(String, String)> = vec![];

        // generate Host(domain) url
        let runtime_domain = land_core::PROD_DOMAIN.get().unwrap().clone();
        let deploy_domain = format!("{}.{}", domain, runtime_domain);
        commands.push((
            format!("traefik/http/routers/{}/rule", uuid),
            format!("Host(`{}`)", deploy_domain),
        ));

        // set routes backend service
        commands.push((
            format!("traefik/http/routers/{}/service", uuid),
            self.service_name.clone(),
        ));

        // add custom-header for land-wasm
        commands.push((
            format!(
                "traefik/http/middlewares/m-{}/headers/customrequestheaders/x-land-wasm",
                uuid
            ),
            storage_path,
        ));
        commands.push((
            format!(
                "traefik/http/middlewares/m-{}/headers/customrequestheaders/x-land-uuid",
                uuid
            ),
            uuid.clone(),
        ));

        // add custom-header to middleware
        commands.push((
            format!("traefik/http/routers/{}/middlewares/0", uuid),
            format!("m-{}", uuid),
        ));

        let op = self.operator.as_ref().unwrap();
        for (k, v) in commands {
            debug!("deploy: {} : {}", k, v);
            op.write(&k, v.clone()).await?;
        }

        Ok(())
    }

}

#[async_trait]
impl RegionTrait for LocalRegion {
    async fn init(&mut self) -> Result<()> {
        let cfg = LocalConfig::init_from_env()?;
        // init redis operator
        let mut builder = opendal::services::Redis::default();
        builder
            .endpoint(&cfg.redis_addr)
            .password(&cfg.redis_password)
            .db(cfg.redis_db);

        let op = Operator::new(builder)?.finish();
        op.write("land-serverless", "setup").await?;
        self.operator = Some(op);
        self.service_name = cfg.runtime;
        Ok(())
    }
    async fn deploy(&self, deploy_id: i32) -> Result<()> {
        let deployment = land_core::dao::deployment::find_by_id(deploy_id).await?;
        if deployment.is_none() {
            return Err(anyhow::anyhow!("deployment not found"));
        }
        let deployment = deployment.unwrap();

        self.deploy_inner(deployment.uuid, deployment.domain, deployment.storage_path)
            .await
    }
    async fn publish(&self, deploy_id: i32) -> Result<()> {
        let deployment = land_core::dao::deployment::find_by_id(deploy_id).await?;
        if deployment.is_none() {
            return Err(anyhow::anyhow!("deployment not found"));
        }
        let deployment = deployment.unwrap();
        let project =
            land_core::dao::project::find_by_id(deployment.owner_id, deployment.project_id).await?;
        if project.is_none() {
            return Err(anyhow::anyhow!("project not found"));
        }
        let project = project.unwrap();

        self.deploy_inner(
            project.uuid,
            deployment.prod_domain,
            deployment.storage_path,
        )
        .await
    }

    async fn drop(&self, _deploy_id: i32) -> Result<()> {
        todo!()
    }
}
