use crate::templates::Engine;
use axum::response::IntoResponse;
use axum_template::RenderHtml;
use land_core::clerk;
use land_vars::{BreadCrumbKey, Page};
use serde::Serialize;

pub(crate) async fn sign_in(engine: Engine) -> impl IntoResponse {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub clerk: clerk::Vars,
    }
    RenderHtml(
        "sign-in.hbs",
        engine,
        Vars {
            page: Page::new("Sign in", BreadCrumbKey::SignIn, None),
            clerk:clerk::get(),
        },
    )
}
