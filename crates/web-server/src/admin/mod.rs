use crate::{sign::SessionUser, PageVars, RenderEngine};
use axum::{response::IntoResponse, Extension};
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

/// storage is the handler for /admin/storage
pub async fn storage(
    engine: RenderEngine,
    Extension(user): Extension<SessionUser>,
) -> impl IntoResponse {
    #[derive(Debug, Serialize, Deserialize)]
    struct Vars {
        pub page: PageVars,
        pub user: SessionUser,
        pub storage: land_dblayer::settings::Storage,
    }

    let s = land_dblayer::settings::get("storage")
        .await
        .unwrap()
        .unwrap(); // it must be initialzed
    let storage: land_dblayer::settings::Storage = serde_json::from_str(&s.value).unwrap();

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
