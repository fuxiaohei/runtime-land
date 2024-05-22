use land_dao::{models, DateTimeUTC};
use serde::Serialize;

#[derive(Serialize)]
pub struct WorkerVar {
    pub id: i32,
    pub ip: String,
    pub hostname: String,
    pub updated_at: DateTimeUTC,
    pub status: String,
}

impl WorkerVar {
    pub fn from_models_vec(workers: Vec<models::worker::Model>) -> Vec<WorkerVar> {
        workers
            .into_iter()
            .map(|w| WorkerVar {
                id: w.id,
                ip: w.ip,
                hostname: w.hostname,
                updated_at: w.updated_at.and_utc(),
                status: w.status.to_string(),
            })
            .collect()
    }
}