use super::{
    auth::SessionUser,
    vars::{PageVars, UserVars},
    AppEngine,
};
use crate::pages::vars::TokenVars;
use axum::extract::Query;
use axum::Form;
use axum::{
    response::{IntoResponse, Redirect},
    Extension,
};
use axum_template::RenderHtml;
use land_dao::user_token::{self, CreatedByCases};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsVars {
    pub page: PageVars,
    pub user: UserVars,
    pub tokens: Vec<TokenVars>,
    pub new_token: Option<TokenVars>,
}

#[derive(Debug, Deserialize)]
pub struct TokenOpsParams {
    token: Option<String>,
}

pub async fn render_settings(
    engine: AppEngine,
    Extension(current_user): Extension<SessionUser>,
    Query(query): Query<TokenOpsParams>,
) -> impl IntoResponse {
    println!("render_settings, current_user: {:?}", current_user);
    let page_vars = PageVars::new(
        "Account Settings".to_string(),
        "/account/settings".to_string(),
    );
    let tokens = user_token::list_by_created(current_user.id, CreatedByCases::Deployment)
        .await
        .unwrap();
    let (token_vars, new_token) = TokenVars::from_models(&tokens, query.token).await;
    println!("--new-token: {:?}", new_token);
    let user_vars = UserVars::new(&current_user);
    let vars = SettingsVars {
        page: page_vars,
        user: user_vars,
        tokens: token_vars,
        new_token,
    };
    RenderHtml("account-settings.hbs", engine, vars)
}

#[derive(Debug, Deserialize)]
pub struct TokenDeleteParams {
    uuid: String,
}

#[instrument(skip_all, "token_delete")]
pub async fn handle_delete_token(
    Extension(current_user): Extension<SessionUser>,
    Query(query): Query<TokenDeleteParams>,
) -> Redirect {
    let token = user_token::find_by_uuid(
        current_user.id,
        query.uuid.clone(),
        CreatedByCases::Deployment,
    )
    .await
    .unwrap();
    if let Some(token) = token {
        user_token::remove(current_user.id, &token.uuid)
            .await
            .unwrap();
        info!("token deleted: {:?}", token);
    }
    Redirect::to("/account/settings")
}

#[derive(Debug, Deserialize)]
pub struct CreateTokenParams {
    name: String,
}

#[instrument(skip_all, "token_create")]
pub async fn handle_create_token(
    Extension(current_user): Extension<SessionUser>,
    Form(query): Form<CreateTokenParams>,
) -> Redirect {
    let token = user_token::create(
        current_user.id,
        query.name,
        3600 * 24 * 365,
        CreatedByCases::Deployment,
    )
    .await
    .unwrap();
    info!("token created: {:?}", token);
    Redirect::to(format!("/account/settings?token={}", token.uuid).as_str())
}
