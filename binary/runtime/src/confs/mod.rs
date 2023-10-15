use anyhow::Result;

mod confs_core;
mod confs_operator;
mod confs_traefik;
mod endpoint;
mod store;

pub use endpoint::ENDPOINT;

pub async fn init(addr_values: String, token_values: String) -> Result<()> {
    let addrs = addr_values
        .split(",")
        .map(str::to_string)
        .collect::<Vec<String>>();
    let tokens = token_values
        .split(",")
        .map(str::to_string)
        .collect::<Vec<String>>();
    if addrs.len() != tokens.len() {
        return Err(anyhow::anyhow!("addrs and tokens not match"));
    }

    endpoint::init().await.unwrap();
    confs_operator::init().await.unwrap();

    for (i, addr) in addrs.iter().enumerate() {
        confs_core::init_conf_file(addr).await.unwrap();
        let token = tokens[i].clone();
        let addr2 = addr.clone();
        tokio::spawn(async move {
            confs_core::start_sync(addr2, token).await;
        });
    }

    Ok(())
}
