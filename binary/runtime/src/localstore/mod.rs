use anyhow::Result;
use once_cell::sync::OnceCell;
use opendal::Operator;

pub static LOCAL_STORE: OnceCell<Operator> = OnceCell::new();

pub async fn init() -> Result<()> {
    let local_op = land_storage::build_operator("fs").await?;
    LOCAL_STORE
        .set(local_op)
        .map_err(|_| anyhow::anyhow!("set local store error"))?;

    Ok(())
}