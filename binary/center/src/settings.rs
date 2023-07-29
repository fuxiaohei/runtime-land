use anyhow::Result;
use land_dao::settings;
use land_dao::Setting;
use once_cell::sync::OnceCell;
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
        let now = chrono::Utc::now();
        let values = vec![
            Setting {
                id: 0,
                key: domain_key.clone(),
                name: "Domain".to_string(),
                value: "runtime.127-0-0-1.nip.io".to_string(),
                created_at: now,
                updated_at: now,
            },
            Setting {
                id: 0,
                key: protocol_key.clone(),
                value: "http".to_string(),
                name: "Protocol".to_string(),
                created_at: now,
                updated_at: now,
            },
        ];
        settings::update(values).await?;

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
