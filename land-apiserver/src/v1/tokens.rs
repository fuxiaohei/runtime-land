use axum::response::IntoResponse;
use axum::Json;
use land_core_service::httputil::ServerJsonError;
use land_service::clerk;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTokenParam {
    pub id: String,
    pub session: String,
    pub user_id: String,
}

pub async fn create(Json(j): Json<CreateTokenParam>) -> Result<impl IntoResponse, ServerJsonError> {
    clerk::verify(&j.session).await?;

    let token = clerk::create_session_token(&j.user_id).await?;
    debug!("Token created: {:?}", token);
    Ok(Json(token))
}
