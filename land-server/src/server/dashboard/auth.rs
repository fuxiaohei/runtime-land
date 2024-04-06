use crate::server::{
    templates::{RenderHtmlMinified, TemplateEngine},
    PageVars,
};
use axum::response::IntoResponse;
use land_core::auth::{get_clerk_env, ClerkEnv};

pub async fn sign_in(engine: TemplateEngine) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    struct SignInVars {
        page: PageVars,
        clerk: ClerkEnv,
    }
    RenderHtmlMinified(
        "auth/sign_in.hbs",
        engine,
        SignInVars {
            page: PageVars::new("Sign-In", ""),
            clerk: get_clerk_env(),
        },
    )
}
