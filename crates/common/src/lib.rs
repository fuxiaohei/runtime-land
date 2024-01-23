mod version;
pub use version::{build_date, build_info, print_version};

mod logging;
pub use logging::init as init_logging;

mod metadata;
pub use metadata::{js_real_target_path, MetaData, MANIFEST_FILE, MANIFEST_VERSION};
