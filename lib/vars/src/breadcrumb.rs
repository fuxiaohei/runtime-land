use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct BreadCrumb {
    pub title: String,
    pub link: Option<String>,
}

impl BreadCrumb {
    pub fn new(key: &BreadCrumbKey) -> Vec<BreadCrumb> {
        match key {
            BreadCrumbKey::Home => vec![BreadCrumb {
                title: "Dashboard".to_string(),
                link: None,
            }],
            BreadCrumbKey::Projects => vec![BreadCrumb {
                title: "Projects".to_string(),
                link: None,
            }],
            BreadCrumbKey::ProjectNew => vec![
                BreadCrumb {
                    title: "Projects".to_string(),
                    link: Some("/projects".to_string()),
                },
                BreadCrumb {
                    title: "New".to_string(),
                    link: None,
                },
            ],
            BreadCrumbKey::ProjectSingle
            | BreadCrumbKey::ProjectTraffic
            | BreadCrumbKey::ProjectSettings => vec![BreadCrumb {
                title: "Projects".to_string(),
                link: Some("/projects".to_string()),
            }],
            BreadCrumbKey::SignIn | BreadCrumbKey::NotFound => vec![],
            BreadCrumbKey::Settings => vec![BreadCrumb {
                title: "Settings".to_string(),
                link: None,
            }],
            BreadCrumbKey::Admin => vec![BreadCrumb {
                title: "Admin".to_string(),
                link: Some("/admin".to_string()),
            }],
            BreadCrumbKey::AdminProjects => vec![
                BreadCrumb {
                    title: "Admin".to_string(),
                    link: Some("/admin".to_string()),
                },
                BreadCrumb {
                    title: "Projects".to_string(),
                    link: None,
                },
            ],
            BreadCrumbKey::AdminUsers => vec![
                BreadCrumb {
                    title: "Admin".to_string(),
                    link: Some("/admin".to_string()),
                },
                BreadCrumb {
                    title: "Users".to_string(),
                    link: None,
                },
            ],
            BreadCrumbKey::AdminWorkers => vec![
                BreadCrumb {
                    title: "Admin".to_string(),
                    link: Some("/admin".to_string()),
                },
                BreadCrumb {
                    title: "Workers".to_string(),
                    link: None,
                },
            ],
            BreadCrumbKey::AdminSettings => vec![
                BreadCrumb {
                    title: "Admin".to_string(),
                    link: Some("/admin".to_string()),
                },
                BreadCrumb {
                    title: "Settings".to_string(),
                    link: None,
                },
            ],
            BreadCrumbKey::AdminDeployLogs => vec![
                BreadCrumb {
                    title: "Admin".to_string(),
                    link: Some("/admin".to_string()),
                },
                BreadCrumb {
                    title: "Deploy Logs".to_string(),
                    link: None,
                },
            ],
        }
    }
}

pub fn handle_nav_active(breadcrumb: &BreadCrumbKey) -> HashMap<String, String> {
    let mut nav_active = HashMap::new();
    nav_active.insert(breadcrumb.to_string(), "active".to_string());
    nav_active
}

/// BreadCrumb enum
#[derive(strum::Display, Clone, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum BreadCrumbKey {
    Home,
    Settings,
    Projects,
    ProjectNew,
    ProjectSingle,
    ProjectTraffic,
    ProjectSettings,
    SignIn,
    NotFound,
    Admin,
    AdminProjects,
    AdminUsers,
    AdminWorkers,
    AdminSettings,
    AdminDeployLogs,
}
