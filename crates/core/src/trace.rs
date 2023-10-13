use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::EnvFilter;

/// init initializes the tracing subscriber.
/// It supports RUN_LOG env var to set the log level.
pub fn init() {
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
            "[month]-[day] [hour]:[minute]:[second].[subsecond digits:2]",
        )
        .unwrap(),
    );

    tracing_subscriber::fmt()
        .with_timer(timer)
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();
}

/// init_for_cli initializes the tracing subscriber for cli.
pub fn init_for_cli() {
    if std::env::var("RUST_LOG").ok().is_none() {
        if cfg!(debug_assertions) {
            std::env::set_var("RUST_LOG", "debug")
        } else {
            std::env::set_var("RUST_LOG", "info")
        }
    }

    tracing_subscriber::fmt()
        .compact()
        .with_level(true)
        .without_time()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();
}
