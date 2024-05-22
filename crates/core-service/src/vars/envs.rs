use land_dao::models;
use serde::Serialize;

#[derive(Serialize)]
pub struct EnvVar {
    pub id: i32,
    pub key: String,
}

impl EnvVar {
    pub async fn from_models_vec(
        envs: Vec<models::project_envs::Model>,
    ) -> anyhow::Result<Vec<EnvVar>> {
        Ok(envs
            .into_iter()
            .map(|e| EnvVar {
                id: e.id,
                key: e.env_key,
            })
            .collect())
    }
}
