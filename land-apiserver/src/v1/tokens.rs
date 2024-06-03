use axum::response::IntoResponse;
use axum::Json;
use land_core_service::httputil::ServerJsonError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTokenUserParam {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub image_url: String,
    pub has_image: bool,
    pub identifier: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTokenSessionParam {
    pub id: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTokenParam {
    pub user: CreateTokenUserParam,
    pub session: CreateTokenSessionParam,
}

pub async fn create(Json(j): Json<CreateTokenParam>) -> Result<impl IntoResponse, ServerJsonError> {
    println!("{:?}", j);
    Ok(Json("abc"))
}
