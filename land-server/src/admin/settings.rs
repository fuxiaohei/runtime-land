use crate::{
    dash::{ok_html, ServerError},
    templates::Engine,
};
use axum::{response::IntoResponse, Extension, Form};
use axum_template::RenderHtml;
use land_core::traffic;
use land_dao::settings::{self, DomainSettings};
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
        pub domain_settings: DomainSettings,
        pub prometheus: traffic::Settings,
    }
    let domain_settings = settings::get_domain_settings().await?;
    let prometheus = traffic::get_settings().await?;
    Ok(RenderHtml(
        "admin/settings.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new("Admin Settings", BreadCrumbKey::AdminSettings, Some(user)),
            prometheus,
            domain_settings,
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

/// update_prometheus updates the prometheus settings, /admin/settings/prometheus
pub async fn update_prometheus(
    Form(f): Form<traffic::Settings>,
) -> Result<impl IntoResponse, ServerError> {
    traffic::set_settings(f).await?;
    Ok(ok_html("Updated successfully"))
}
