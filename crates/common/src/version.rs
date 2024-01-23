use lazy_static::lazy_static;

lazy_static! {
    pub static ref VERSION: String = build_info();
    pub static ref FULL_VERSION: String = build_fullinfo();
}

pub fn build_info() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn build_fullinfo() -> String {
    format!(
        "{} ({} {})\ncommit Hash: {}\ncommit Date: {}\nrust version: {}",
        env!("CARGO_PKG_VERSION"),
        env!("VERGEN_GIT_SHA"),
        env!("VERGEN_BUILD_DATE"),
        env!("VERGEN_GIT_SHA"),
        env!("VERGEN_GIT_COMMIT_DATE"),
        env!("VERGEN_RUSTC_SEMVER"),
    )
}

pub fn build_date() -> String {
    env!("VERGEN_BUILD_TIMESTAMP").to_string()
}

/// print_version prints the version of the binary.
pub fn print_version(binary: &str, verbose: bool) {
    if verbose {
        println!("{} {}", binary, *FULL_VERSION);
    } else {
        println!("{} {}", binary, *VERSION);
    }
}
