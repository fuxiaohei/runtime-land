use crate::region::conf;
use anyhow::Result;
use land_core::confdata::DomainSetting;
use land_dao::settings;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::info;

mod storage;
pub use storage::init as init_storage;
pub use storage::load_settings as load_storage_settings;
pub use storage::reload_s3;

/// DOMAIN is the domain to access the function
pub static DOMAIN: Lazy<Mutex<DomainSetting>> = Lazy::new(|| {
    Mutex::new(DomainSetting {
        domain: "".to_string(),
        protocol: "".to_string(),
    })
});

/// init settings
#[tracing::instrument(name = "[SETTING]", skip_all)]
pub async fn init() -> Result<()> {
    let domain_key = settings::Key::ProductionDomain.to_string();
    let protocol_key = settings::Key::ProductionProtocol.to_string();
    // let s3_key = settings::Key::S3Storage.to_string();
    // let local_storage_key = settings::Key::LocalStorage.to_string();

    let keys = vec![
        domain_key.clone(),
        protocol_key.clone(),
        // s3_key.clone(),
        // local_storage_key.clone(),
    ];
    let settings_map = settings::list_maps(keys).await?;

    // init production domain settings to db
    if !settings_map.contains_key(&domain_key) && !settings_map.contains_key(&protocol_key) {
        let values: HashMap<String, String> = vec![
            (domain_key.clone(), "runtime.127-0-0-1.nip.io".to_string()),
            (protocol_key.clone(), "http".to_string()),
        ]
        .into_iter()
        .collect();
        settings::update_maps(values).await?;

        let mut d = DOMAIN.lock().await;
        d.domain = "runtime.127-0-0-1.nip.io".to_string();
        d.protocol = "http".to_string();

        info!(
            "Init, DOMAIN:{}, PROTOCOL:{}",
            "runtime.127-0-0-1.nip.io", "http"
        );
        return Ok(());
    }

    /*
    // init s3 storage settings to db
    if !settings_map.contains_key(&s3_key) {
        storage::first_init_s3().await?;
        debug!("Init, S3:default");
    }

    // init local storage settings to db
    if !settings_map.contains_key(&local_storage_key) {
        storage::first_init_local().await?;
        debug!("Init, Local:default");
    }*/

    let domain_value = settings_map.get(&domain_key).unwrap();
    let protocol_value = settings_map.get(&protocol_key).unwrap();

    let mut d = DOMAIN.lock().await;
    d.domain = domain_value.clone();
    d.protocol = protocol_value.clone();

    info!(
        "Loaded, DOMAIN:{}, PROTOCOL:{}",
        domain_value, protocol_value
    );

    Ok(())
}

/// update_domains updates production domains settings
pub async fn update_domains(domain: String, protocol: String) -> Result<()> {
    let map_values: HashMap<String, String> = vec![
        (settings::Key::ProductionDomain.to_string(), domain.clone()),
        (
            settings::Key::ProductionProtocol.to_string(),
            protocol.clone(),
        ),
    ]
    .into_iter()
    .collect();
    land_dao::settings::update_maps(map_values).await?;

    conf::trigger().await;

    let mut d = DOMAIN.lock().await;
    d.domain = domain;
    d.protocol = protocol;

    Ok(())
}

/// get_domains returns the domain and protocol
pub async fn get_domains() -> (String, String) {
    let d = DOMAIN.lock().await;
    (d.domain.clone(), d.protocol.clone())
}
