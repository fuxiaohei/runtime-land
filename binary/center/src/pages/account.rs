use super::{
    auth::SessionUser,
    vars::{PageVars, UserVars},
    AppEngine,
};
use axum::{response::IntoResponse, Extension};
use axum_template::RenderHtml;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsVars {
    pub page: PageVars,
    pub user: UserVars,
}

pub async fn render_settings(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
) -> impl IntoResponse {
    println!("render_settings, current_user: {:?}", current_user);
    let page_vars = PageVars::new(
        "Account Settings".to_string(),
        "/account/settings".to_string(),
    );
    let user_vars = UserVars::new(&current_user);
    let vars = SettingsVars {
        page: page_vars,
        user: user_vars,
    };
    RenderHtml("account-settings.hbs", engine, vars)
}
