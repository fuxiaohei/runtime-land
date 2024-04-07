#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum DeployStatus {
    Compiling,
    Deploying,
    Success,
    Failed,
}
