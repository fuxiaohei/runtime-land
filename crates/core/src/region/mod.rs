use anyhow::Result;
use async_trait::async_trait;
use tracing::info;

mod local;

#[async_trait]
pub trait RegionTrait {
    async fn init(&mut self) -> Result<()>;
    async fn deploy(&self, deploy_id: i32) -> Result<()>;
    async fn publish(&self, deploy_id: i32) -> Result<()>;
    async fn offline(&self, deploy_id: i32) -> Result<()>;
}

impl std::fmt::Debug for Box<dyn RegionTrait + Send + Sync> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(" RegionTrait").finish()
    }
}

pub static REGION: once_cell::sync::OnceCell<Box<dyn RegionTrait + Send + Sync>> =
    once_cell::sync::OnceCell::new();

#[tracing::instrument(name = "[REGION]")]
pub async fn init() -> Result<()> {
    let region_type = std::env::var("LAND_REGION").unwrap_or_else(|_| "local".to_string());
    info!("region type: {}", region_type);

    match region_type.as_str() {
        "local" => {
            let mut region = local::LocalRegion::new();
            region.init().await?;
            REGION.set(Box::new(region)).unwrap();
            info!("local initialized");
        }
        _ => {
            return Err(anyhow::anyhow!("region not found"));
        }
    }
    Ok(())
}
