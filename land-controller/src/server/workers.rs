use axum::response::IntoResponse;
use axum::{Extension, Form};
use axum_csrf::CsrfToken;
use land_core_service::clerkauth::SessionUser;
use land_core_service::httputil::{response_redirect, ServerError};
use land_core_service::template::{self, RenderHtmlMinified};
use land_core_service::vars::{PageVars, TokenVar, WorkerVar};
use land_dao::user::TokenUsage;
use serde::Deserialize;
use tracing::info;

/// index is a handler for GET /workers
pub async fn index(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    engine: template::Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(serde::Serialize)]
    struct IndexVars {
        page: PageVars,
        user: SessionUser,
        csrf: String,
        tokens: Vec<TokenVar>,
        workers: Vec<WorkerVar>,
    }

    let csrf = csrf_layer.authenticity_token()?;

    // list cmd line tokens
    let token_values =
        land_dao::user::list_tokens_by_user(user.id, Some(TokenUsage::Worker)).await?;
    let mut tokens = vec![];
    for token in token_values {
        // need to check if the token is new, unset it if it is
        let is_new = land_dao::user::is_new_token(token.id).await;
        if is_new {
            land_dao::user::unset_new_token(token.id).await;
        }
        tokens.push(TokenVar {
            id: token.id,
            name: token.name,
            value: token.value,
            is_new: true,
            updated_at: token.updated_at.and_utc(),
        });
    }

    // list workers
    let workers_value = land_dao::worker::list_all().await?;
    let workers = WorkerVar::from_models_vec(workers_value);

    Ok((
        csrf_layer,
        RenderHtmlMinified(
            "workers.hbs",
            engine,
            IndexVars {
                page: PageVars::new_admin("Workers", "admin-workers"),
                user,
                csrf,
                tokens,
                workers,
            },
        ),
    )
        .into_response())
}

#[derive(Deserialize, Debug)]
pub struct CreateWorkerTokenForm {
    pub name: String,
    pub csrf: String,
}

/// create_token is a handler for POST /create-worker-token
pub async fn create_token(
    Extension(user): Extension<SessionUser>,
    csrf_layer: CsrfToken,
    Form(form): Form<CreateWorkerTokenForm>,
) -> Result<impl IntoResponse, ServerError> {
    csrf_layer.verify(&form.csrf)?;
    let token =
        land_dao::user::create_new_token(user.id, &form.name, 365 * 24 * 3600, TokenUsage::Worker)
            .await?;
    info!("New worker token created: {:?}", token);
    Ok(response_redirect("/workers"))
}
