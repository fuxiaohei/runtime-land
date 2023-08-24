use anyhow::Result;
use async_trait::async_trait;
use land_core::confdata::RouteConfItem;
use tracing::info;

#[async_trait]
pub trait ConfOperatorTrait {
    async fn init(&mut self) -> Result<()>;
    async fn deploy(&self, item: RouteConfItem) -> Result<()>;
    async fn remove(&self, item: RouteConfItem) -> Result<()>;
}

/// OPERATOR is the conf operator
pub static OPERATOR: once_cell::sync::OnceCell<Box<dyn ConfOperatorTrait + Send + Sync>> =
    once_cell::sync::OnceCell::new();

pub async fn init_operator() -> Result<()> {
    // init operator
    let operator_type =
        std::env::var("CONF_OPERATOR_TYPE").unwrap_or_else(|_| "traefik-redis".to_string());
    info!("operator type: {}", operator_type);
    match operator_type.as_str() {
        "traefik-redis" => {
            let mut op = super::traefik::TraefikOperator::new();
            op.init().await?;
            OPERATOR
                .set(Box::new(op))
                .map_err(|_| anyhow::anyhow!("set operator error"))?;
            info!("init operator: {:?}", operator_type);
        }
        _ => {
            return Err(anyhow::anyhow!("operator unknown"));
        }
    }
    Ok(())
}
