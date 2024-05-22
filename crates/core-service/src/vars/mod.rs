use serde::Serialize;

mod project;
pub use project::ProjectVar;
mod worker;
pub use worker::WorkerVar;
mod token;
pub use token::TokenVar;
mod envs;
pub use envs::EnvVar;

/// PageVars is the common variables for all pages
#[derive(Debug, Default, Serialize)]
pub struct PageVars {
    pub title: String,
    pub version: String,
    pub build_time: String,
    pub nav: String,
    pub admin: bool,
}

impl PageVars {
    pub fn new(title: &str, nav: &str) -> Self {
        Self {
            title: title.to_string(),
            version: land_common::version::SHORT.to_string(),
            build_time: land_common::version::date(),
            nav: nav.to_string(),
            admin: false,
        }
    }
    pub fn new_admin(title: &str, nav: &str) -> Self {
        Self {
            title: title.to_string(),
            version: land_common::version::SHORT.to_string(),
            build_time: land_common::version::date(),
            nav: nav.to_string(),
            admin: true,
        }
    }
}

#[derive(Serialize)]
pub struct PaginationVarItem {
    pub link: String,
    pub current: bool,
    pub page: u64,
}

#[derive(Serialize)]
pub struct PaginationVar {
    pub current: u64,
    pub count: u64,
    pub total: u64,
    pub items: Vec<PaginationVarItem>,
}

impl PaginationVar {
    pub fn new(current: u64, size: u64, count: u64, total: u64, link: &str) -> PaginationVar {
        let mut items = vec![];
        for i in 1..=total {
            let page_link = if link.contains('?') {
                format!("{}&page={}&size={}", link, i, size)
            } else {
                format!("{}?page={}&size={}", link, i, size)
            };
            items.push(PaginationVarItem {
                link: page_link,
                current: i == current,
                page: i,
            });
        }
        PaginationVar {
            current,
            count,
            total,
            items,
        }
    }
}
