use crate::{sign::SessionUser, AppError, PageVars, RenderEngine};
use axum::{response::IntoResponse, Extension, Form};
use axum_template::RenderHtml;
use serde::{Deserialize, Serialize};

/// dashboard is the handler for /admin/dashboard
pub async fn dashboard(
    engine: RenderEngine,
    Extension(user): Extension<SessionUser>,
) -> impl IntoResponse {
    #[derive(Debug, Serialize, Deserialize)]
    struct Vars {
        pub page: PageVars,
        pub user: SessionUser,
    }

    RenderHtml(
        "admin/dashboard.hbs",
        engine,
        Vars {
            page: PageVars::new("Dashboard - Management", "/admin/dashboard"),
            user,
        },
    )
}

/// settings is the handler for /admin/settings
pub async fn settings(
    engine: RenderEngine,
    Extension(user): Extension<SessionUser>,
) -> impl IntoResponse {
    #[derive(Debug, Serialize, Deserialize)]
    struct Vars {
        pub page: PageVars,
        pub user: SessionUser,
        pub storage: land_dblayer::storage::Storage,
        pub prod_domain: String,
        pub prod_protocol: String,
    }

    let (prod_domain, prod_protocol) = land_dblayer::settings::get_domain_settings().await.unwrap();

    let s = land_dblayer::settings::get("storage")
        .await
        .unwrap()
        .unwrap(); // it must be initialzed
    let storage: land_dblayer::storage::Storage = serde_json::from_str(&s.value).unwrap();

    RenderHtml(
        "admin/settings.hbs",
        engine,
        Vars {
            page: PageVars::new("Settings | Dashboard", "/admin/settings"),
            user,
            storage,
            prod_domain,
            prod_protocol,
        },
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DomainUpdateForm {
    pub domain: String,
    pub protocol: String,
}

/// domain_update is the handler for POST /admin/domain
pub async fn domain_update(Form(form): Form<DomainUpdateForm>) -> Result<String, AppError> {
    land_dblayer::settings::set_domain_settings(form.domain, form.protocol).await?;
    land_dblayer::settings::set_confs_refresh_flag().await?; // trigger refresh confs
    Ok("ok".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageUpdateForm {
    /*
        current: fs
    fs_path: /tmp/runtime-land-data/
    r2_endpoint: http://r2.local
    r2_bucket: runtime-land
    r2_region: auto
    r2_access_key: access_key
    r2_secret_key: secret_key
    r2_base_path: runtime-land-data
    r2_url: */
    pub current: String,
    pub fs_path: String,
    pub r2_endpoint: String,
    pub r2_bucket: String,
    pub r2_region: String,
    pub r2_access_key: String,
    pub r2_secret_key: String,
    pub r2_base_path: String,
    pub r2_url: String,
}

/// storage_update is the handler for POST /admin/storage
pub async fn storage_update(Form(form): Form<StorageUpdateForm>) -> Result<String, AppError> {
    let fs_storage = land_dblayer::storage::FsStorage {
        directory: form.fs_path,
    };
    let r2_storage = land_dblayer::storage::R2Storage {
        endpoint: form.r2_endpoint,
        bucket: form.r2_bucket,
        region: form.r2_region,
        access_key: form.r2_access_key,
        secret_key: form.r2_secret_key,
        base_path: form.r2_base_path,
        url: Some(form.r2_url),
    };
    let storage = land_dblayer::storage::Storage {
        current: form.current,
        fs: fs_storage,
        r2: r2_storage,
    };
    let value = serde_json::to_string(&storage)?;
    land_dblayer::settings::set("storage", &value).await?;
    land_dblayer::settings::set_confs_refresh_flag().await?; // trigger refresh confs
    Ok("ok".to_string())
}

/// runners is the handler for /admin/runners
pub async fn runners(
    engine: RenderEngine,
    Extension(user): Extension<SessionUser>,
) -> impl IntoResponse {
    #[derive(Debug, Serialize, Deserialize)]
    struct Vars {
        pub page: PageVars,
        pub user: SessionUser,
    }

    RenderHtml(
        "admin/runners.hbs",
        engine,
        Vars {
            page: PageVars::new("Runners | Dashboard", "/admin/runners"),
            user,
        },
    )
}
