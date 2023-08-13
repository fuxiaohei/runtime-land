use anyhow::Result;
use land_dao::settings;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use tracing::info;

/// DOMAIN is the domain to access the function
pub static DOMAIN: OnceCell<String> = OnceCell::new();
/// PROTOCOL is the protocol to access the function
pub static PROTOCOL: OnceCell<String> = OnceCell::new();

/// init settings
#[tracing::instrument(name = "[SETTING]", skip_all)]
pub async fn init() -> Result<()> {
    let domain_key = settings::Key::ProductionDomain.to_string();
    let protocol_key = settings::Key::ProductionProtocol.to_string();

    let keys = vec![domain_key.clone(), protocol_key.clone()];
    let settings_map = settings::list_maps(keys).await?;

    if !settings_map.contains_key(&domain_key) && !settings_map.contains_key(&protocol_key) {
        let values: HashMap<String, String> = vec![
            (
                settings::Key::ProductionDomain.to_string(),
                "runtime.127-0-0-1.nip.io".to_string(),
            ),
            (
                settings::Key::ProductionProtocol.to_string(),
                "http".to_string(),
            ),
        ]
        .into_iter()
        .collect();
        settings::update_maps(values).await?;

        DOMAIN.set("runtime.127-0-0-1.nip.io".to_string()).unwrap();
        PROTOCOL.set("http".to_string()).unwrap();

        info!(
            "Init, DOMAIN:{}, PROTOCOL:{}",
            "runtime.127-0-0-1.nip.io", "http"
        );
        return Ok(());
    }

    let domain = settings_map.get(&domain_key).unwrap();
    let protocol = settings_map.get(&protocol_key).unwrap();

    DOMAIN.set(domain.clone()).unwrap();
    PROTOCOL.set(protocol.clone()).unwrap();
    info!("Loaded, DOMAIN:{}, PROTOCOL:{}", domain, protocol);

    Ok(())
}
