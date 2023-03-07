mod meta;
pub use meta::Meta;
pub use meta::MetadataBuild;
pub use meta::DEFAULT_METADATA_FILE;

use lazy_static::lazy_static;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    if std::env::var("RUST_LOG").ok().is_none() {
        if cfg!(debug_assertions) {
            std::env::set_var("RUST_LOG", "debug")
        } else {
            std::env::set_var("RUST_LOG", "info")
        }
    }

    let timer = OffsetTime::new(
        time::UtcOffset::from_hms(8, 0, 0).unwrap(),
        time::format_description::parse(
            "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]",
        )
        .unwrap(),
    );

    tracing_subscriber::fmt()
        .with_timer(timer)
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();
}

/// build_info returns the version information of the current build.
pub fn build_info() -> String {
    format!(
        "{} ({} {})",
        env!("VERGEN_BUILD_SEMVER"),
        env!("VERGEN_GIT_SHA_SHORT"),
        env!("VERGEN_GIT_COMMIT_DATE")
    )
}

lazy_static! {
    pub static ref VERSION: String = build_info();
}

// get_version returns the version of the current build.
pub fn get_version() -> &'static str {
    &VERSION
}

pub mod keyvalue;