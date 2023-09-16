mod confs_core;
mod confs_operator;
mod confs_traefik;
mod endpoint;
mod store;

pub use endpoint::ENDPOINT;

pub async fn init(addr: String, token: String) {
    endpoint::init().await.unwrap();

    let _ = store::init().await;
    confs_operator::init().await.unwrap();
    confs_core::init_conf_file().await.unwrap();

    tokio::spawn(async move {
        confs_core::start_sync(&addr, &token).await;
    });
}
