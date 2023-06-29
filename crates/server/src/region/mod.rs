use anyhow::Result;
use axum::async_trait;

mod local;

#[async_trait]
pub trait RegionTrait {
    async fn init(&mut self) -> Result<()>;
    async fn deploy(&self, deploy_id: i32) -> Result<()>;
    async fn publish(&self, deploy_id: i32) -> Result<()>;
    async fn drop(&self, deploy_id: i32) -> Result<()>;
}
