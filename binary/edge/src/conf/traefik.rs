use super::ConfOperatorTrait;
use anyhow::Result;
use async_trait::async_trait;
use envconfig::Envconfig;
use land_core::confdata::RouteConfItem;
use opendal::Operator;
use tracing::debug;

#[derive(Envconfig, Debug)]
pub struct TraefikConfig {
    #[envconfig(from = "TRAEFIK_REDIS_ADDR", default = "127.0.0.1:6379")]
    pub redis_addr: String,
    #[envconfig(from = "TRAEFIK_REDIS_PASSWORD", default = "")]
    pub redis_password: String,
    #[envconfig(from = "TRAEFIK_REDIS_DB", default = "0")]
    pub redis_db: i64,
    // runtime is the runtime service name in docker-compose.yml
    #[envconfig(from = "TRAEFIK_RUNTIME_NAME", default = "land-runtime")]
    pub runtime: String,
}

#[derive(Debug)]
pub struct TraefikOperator {
    operator: Option<Operator>,
    service_name: String,
}

impl TraefikOperator {
    pub fn new() -> Self {
        Self {
            operator: None,
            service_name: "".to_string(),
        }
    }
    async fn deploy_inner(&self, item: &RouteConfItem) -> Result<()> {
        let mut commands: Vec<(String, String)> = vec![];
        commands.push((
            format!("traefik/http/routers/{}/rule", item.key),
            format!("Host(`{}`)", item.domain),
        ));
        commands.push((
            format!("traefik/http/routers/{}/service", item.key),
            self.service_name.clone(),
        ));
        commands.push((
            format!(
                "traefik/http/middlewares/m-{}/headers/customrequestheaders/x-land-wasm",
                item.key
            ),
            item.module.clone(),
        ));
        commands.push((
            format!(
                "traefik/http/middlewares/m-{}/headers/customrequestheaders/x-land-uuid",
                item.key
            ),
            item.key.clone(),
        ));
        commands.push((
            format!("traefik/http/routers/{}/middlewares/0", item.key),
            format!("m-{}", item.key),
        ));

        let op = self.operator.as_ref().unwrap();
        for (k, v) in commands {
            debug!("traefik deploy: {} : {}", k, v);
            op.write(&k, v.clone()).await?;
        }

        Ok(())
    }

    async fn remove_inner(&self, item: &RouteConfItem) -> Result<()> {
        let keys = vec![
            format!("traefik/http/routers/{}/rule", item.key),
            format!("traefik/http/routers/{}/service", item.key),
            format!(
                "traefik/http/middlewares/m-{}/headers/customrequestheaders/x-land-wasm",
                item.key
            ),
            format!(
                "traefik/http/middlewares/m-{}/headers/customrequestheaders/x-land-uuid",
                item.key
            ),
            format!("traefik/http/routers/{}/middlewares/0", item.key),
        ];
        let op = self.operator.as_ref().unwrap();
        for k in keys {
            debug!("traefik remove: {}", k);
            op.delete(&k).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl ConfOperatorTrait for TraefikOperator {
    #[tracing::instrument(skip_all, name = "[TRAEFIK_REDIS]")]
    async fn init(&mut self) -> Result<()> {
        let cfg = TraefikConfig::init_from_env()?;
        // init redis operator
        let mut builder = opendal::services::Redis::default();
        builder
            .endpoint(&cfg.redis_addr)
            .password(&cfg.redis_password)
            .db(cfg.redis_db);

        let op = Operator::new(builder)?.finish();
        let now = chrono::Utc::now().timestamp();
        op.write("runtime-land-traefik-redis", now.to_string())
            .await?;
        self.operator = Some(op);
        self.service_name = cfg.runtime;
        Ok(())
    }

    #[tracing::instrument(skip_all, name = "[TRAEFIK_REDIS]")]
    async fn deploy(&self, item: RouteConfItem) -> Result<()> {
        self.deploy_inner(&item).await
    }

    #[tracing::instrument(skip_all, name = "[TRAEFIK_REDIS]")]
    async fn remove(&self, item: RouteConfItem) -> Result<()> {
        self.remove_inner(&item).await
    }
}
