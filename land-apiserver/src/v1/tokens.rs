use crate::v1::Data;
use axum::response::IntoResponse;
use axum::Json;
use land_core_service::httputil::ServerJsonError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateTokenUserParam {
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    #[serde(rename = "imageUrl")]
    pub image_url: String,
    #[serde(rename = "hasImage")]
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
    Ok(Json(Data {
        data: "ok".to_string(),
    }))
}
