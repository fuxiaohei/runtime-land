use breadcrumb::{handle_nav_active, BreadCrumb};
use land_common::version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod breadcrumb;
pub use breadcrumb::BreadCrumbKey;

mod user;
pub use user::AuthUser;

mod project;
pub use project::Project;

mod token;
pub use token::Token;

mod worker;
pub use worker::Worker;

mod task;
pub use task::Task;

mod pagination;
pub use pagination::{Pagination, PaginationItem};

#[derive(Serialize, Deserialize, Debug)]
pub struct Page {
    pub title: String,
    pub nav_active: HashMap<String, String>,
    pub breadcrumb: Vec<BreadCrumb>,
    pub user: Option<AuthUser>,
    pub version: String,
}

impl Page {
    pub fn new(title: &str, breadcrumb: BreadCrumbKey, user: Option<AuthUser>) -> Self {
        Page {
            title: title.to_string(),
            nav_active: handle_nav_active(&breadcrumb),
            breadcrumb: BreadCrumb::new(&breadcrumb),
            user,
            version: version::short(),
        }
    }
}
