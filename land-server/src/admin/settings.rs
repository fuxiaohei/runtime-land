use crate::{
    dash::{ok_html, ServerError},
    templates::Engine,
};
use axum::{response::IntoResponse, Extension, Form};
use axum_template::RenderHtml;
use land_dao::settings;
use land_vars::{AuthUser, BreadCrumbKey, Page};
use serde::{Deserialize, Serialize};

pub async fn index(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub nav_admin: bool,
    }
    Ok(RenderHtml(
        "admin/settings.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new("Admin Settings", BreadCrumbKey::AdminSettings, Some(user)),
        },
    ))
}

#[derive(Deserialize)]
pub struct UpdateDomainForm {
    pub domain: String,
    pub protocol: Option<String>,
}

/// update_domains updates the domain settings, /admin/settings/domains
pub async fn update_domains(
    Form(f): Form<UpdateDomainForm>,
) -> Result<impl IntoResponse, ServerError> {
    settings::set_domain_settings(&f.domain, &f.protocol.unwrap_or("http".to_string())).await?;
    Ok(ok_html("Updated successfully"))
}
