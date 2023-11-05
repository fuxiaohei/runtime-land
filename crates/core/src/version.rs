use lazy_static::lazy_static;

/// build_info returns the version information of the current build.
fn build_info() -> String {
    format!(
        "{} ({} {})",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA"),
        env!("VERGEN_BUILD_DATE"),
    )
}

fn build_fullinfo() -> String {
    format!(
        "{} ({} {})",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA"),
        chrono::Utc::now().to_rfc3339(),
    )
}

lazy_static! {
    pub static ref VERSION: String = build_info();
    pub static ref FULLVERSION: String = build_fullinfo();
}

/// get returns the version of the current build.
pub fn get() -> &'static str {
    &VERSION
}

/// get_full returns the version of the current build.
pub fn get_full() -> &'static str {
    &FULLVERSION
}

/// get_about returns the version of the current build.
pub fn get_about() -> String {
    format!("{}\nThe Runtime.land command line tool", get())
}
