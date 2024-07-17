#[derive(strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum Status {
    Waiting,
    Compiling, // if compilation is long time, we need mark it as compiling
    Uploading,
    Deploying,
    Success,
    Failed,
}
