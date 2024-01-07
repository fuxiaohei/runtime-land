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

/// storage is the handler for /admin/storage
pub async fn storage(
    engine: RenderEngine,
    Extension(user): Extension<SessionUser>,
) -> impl IntoResponse {
    #[derive(Debug, Serialize, Deserialize)]
    struct Vars {
        pub page: PageVars,
        pub user: SessionUser,
        pub storage: land_dblayer::storage::Storage,
    }

    let s = land_dblayer::settings::get("storage")
        .await
        .unwrap()
        .unwrap(); // it must be initialzed
    let storage: land_dblayer::storage::Storage = serde_json::from_str(&s.value).unwrap();

    RenderHtml(
        "admin/storage.hbs",
        engine,
        Vars {
            page: PageVars::new("Storage - Management", "/admin/storage"),
            user,
            storage,
        },
    )
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
    Ok("ok".to_string())
}
