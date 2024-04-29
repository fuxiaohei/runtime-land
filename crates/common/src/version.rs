use lazy_static::lazy_static;

lazy_static! {
    pub static ref SHORT: String = build_short();
    pub static ref LONG: String = build_long();
}

fn build_short() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// date of the build
pub fn date() -> String {
    env!("VERGEN_BUILD_TIMESTAMP").to_string()
}

fn build_long() -> String {
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

/// print the version of the binary
pub fn print(binary: &str, verbose: bool) {
    if verbose {
        println!("{} {}", binary, *LONG);
    } else {
        println!("{} {}", binary, *SHORT);
    }
}

/// hostname returns the hostname of the machine
pub fn hostname() -> String {
    // read HOSTNAME env
    let hostname = std::env::var("HOSTNAME");
    if let Ok(hostname) = hostname {
        return hostname.trim().to_string();
    }
    hostname::get()
        .unwrap_or("unknown".into())
        .to_string_lossy()
        .to_string()
}
