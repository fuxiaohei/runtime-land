use super::RegionTrait;
use anyhow::Result;
use async_trait::async_trait;
use envconfig::Envconfig;
use opendal::Operator;
use tracing::{debug, info, warn};

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

#[derive(Debug)]
pub struct LocalRegion {
    operator: Option<Operator>,
    service_name: String,
}

impl LocalRegion {
    pub fn new() -> Self {
        Self {
            operator: None,
            service_name: "".to_string(),
        }
    }
    async fn deploy_inner(&self, uuid: String, domain: String, storage_path: String) -> Result<()> {
        //  redis rules for traefik proxy, set key=value
        let mut commands: Vec<(String, String)> = vec![];

        // generate Host(domain) url
        let runtime_domain = crate::PROD_DOMAIN.get().unwrap().clone();
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

    async fn remove_inner(&self, uuid: String) -> Result<()> {
        let keys = vec![
            format!("traefik/http/routers/{}/rule", uuid),
            format!("traefik/http/routers/{}/service", uuid),
            format!(
                "traefik/http/middlewares/m-{}/headers/customrequestheaders/x-land-wasm",
                uuid
            ),
            format!(
                "traefik/http/middlewares/m-{}/headers/customrequestheaders/x-land-uuid",
                uuid
            ),
            format!("traefik/http/routers/{}/middlewares/0", uuid),
        ];
        let op = self.operator.as_ref().unwrap();
        for k in keys {
            debug!("remove: {}", k);
            op.delete(&k).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl RegionTrait for LocalRegion {
    #[tracing::instrument(skip_all, name = "[LOCAL_REGION]")]
    async fn init(&mut self) -> Result<()> {
        let cfg = LocalConfig::init_from_env()?;
        // init redis operator
        let mut builder = opendal::services::Redis::default();
        builder
            .endpoint(&cfg.redis_addr)
            .password(&cfg.redis_password)
            .db(cfg.redis_db);

        let op = Operator::new(builder)?.finish();
        let now = chrono::Utc::now().timestamp();
        op.write("land-serverless", now.to_string()).await?;
        self.operator = Some(op);
        self.service_name = cfg.runtime;
        Ok(())
    }

    #[tracing::instrument(skip(self), name = "[LOCAL_REGION]")]
    async fn deploy(&self, deploy_id: i32) -> Result<()> {
        let deployment = crate::dao::deployment::find_by_id(deploy_id).await?;
        if deployment.is_none() {
            return Err(anyhow::anyhow!("deployment not found"));
        }
        let deployment = deployment.unwrap();

        let res = self
            .deploy_inner(deployment.uuid, deployment.domain, deployment.storage_path)
            .await;
        if res.is_err() {
            warn!("deploy failed: {:?}", res);
            crate::dao::deployment::update_failure(deploy_id).await?;
        } else {
            info!("deploy success");
            crate::dao::deployment::update_success(deploy_id).await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self), name = "[LOCAL_REGION]")]
    async fn publish(&self, deploy_id: i32) -> Result<()> {
        let deployment = crate::dao::deployment::find_by_id(deploy_id).await?;
        if deployment.is_none() {
            return Err(anyhow::anyhow!("deployment not found"));
        }
        let deployment = deployment.unwrap();
        let project =
            crate::dao::project::find_by_id(deployment.owner_id, deployment.project_id).await?;
        if project.is_none() {
            return Err(anyhow::anyhow!("project not found"));
        }
        let project = project.unwrap();
        let deploy_uuid = format!("PROD-{}", project.uuid);
        let res = self
            .deploy_inner(deploy_uuid, deployment.prod_domain, deployment.storage_path)
            .await;
        if res.is_err() {
            warn!("deploy failed: {:?}", res);
            crate::dao::deployment::update_failure(deploy_id).await?;
        } else {
            info!("deploy success");
            crate::dao::deployment::update_success(deploy_id).await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self), name = "[LOCAL_REGION]")]
    async fn remove(&self, deploy_id: i32) -> Result<()> {
        let deployment = crate::dao::deployment::find_by_id(deploy_id).await?;
        if deployment.is_none() {
            return Err(anyhow::anyhow!("deployment not found"));
        }
        let deployment = deployment.unwrap();
        let mut uuid = deployment.uuid;
        if deployment.prod_status == crate::dao::deployment::ProdStatus::Prod as i32 {
            let project =
                crate::dao::project::find_by_id(deployment.owner_id, deployment.project_id).await?;
            if project.is_none() {
                return Err(anyhow::anyhow!("project not found"));
            }
            uuid = format!("PROD-{}", project.unwrap().uuid);
        }
        let res = self.remove_inner(uuid).await;
        if res.is_err() {
            warn!("remove failed: {:?}", res);
        } else {
            info!("remove success");
            // No need to update status. When remove button click in dashboard, the status is already Deleted.
            // TODO: If the removing action is failed, we need another background job to check the status and update it.
            // crate::dao::deployment::remove(deploy_id).await?;
        }
        Ok(())
    }
}
