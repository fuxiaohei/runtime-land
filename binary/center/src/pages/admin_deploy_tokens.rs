use super::auth::SessionUser;
use super::vars::{format_time, PageVars, PaginationVars, UserVars};
use super::{admin, AppEngine};
use anyhow::Result;
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::Extension;
use axum_csrf::CsrfToken;
use axum_template::RenderHtml;
use land_dao::user;
use land_dao::user_token::{self, CreatedByCases};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AdminTokenVars {
    pub name: String,
    pub uuid: String,
    pub value: String,
    pub expired_timeago: String,
    pub status: String,
    pub owner_id: i32,
    pub owner_name: String,
    pub owner_email: String,
    pub created_by: String,
}

impl AdminTokenVars {
    pub async fn from_db(
        created_by: CreatedByCases,
        page: u64,
        page_size: u64,
        base_uri: &str,
    ) -> Result<(Vec<AdminTokenVars>, PaginationVars, u64)> {
        let (tokens, total_pages, total_items) =
            user_token::list_with_page(created_by, page, page_size).await?;

        let ids: HashSet<i32> = tokens.iter().map(|t| t.owner_id).collect();
        let users = user::list_by_ids(ids.into_iter().collect()).await?;

        let mut vars = vec![];
        for token in tokens {
            let mut token_vars = AdminTokenVars {
                name: token.name.clone(),
                uuid: token.uuid.clone(),
                value: token.value.clone(),
                expired_timeago: format_time(token.expired_at.unwrap()),
                status: token.status.clone(),
                owner_id: token.owner_id,
                owner_name: "".to_string(),
                owner_email: "".to_string(),
                created_by: token.created_by.clone(),
            };
            if token.owner_id > 0 {
                let user = users.get(&token.owner_id);
                if user.is_none() {
                    continue;
                }
                let user = user.unwrap();
                token_vars.owner_name = user.nick_name.clone();
                token_vars.owner_email = user.email.clone();
            }
            vars.push(token_vars);
        }
        let pager = PaginationVars::new(page, total_pages, base_uri);
        Ok((vars, pager, total_items))
    }
}

type DeployTokensParams = admin::ProjectsQueryParams;

#[derive(Debug, Serialize, Deserialize)]
struct AllVars {
    pub page: PageVars,
    pub user: UserVars,
    pub tokens_count: u64,
    pub tokens: Vec<AdminTokenVars>,
    pub pagination: PaginationVars,
    pub search: String,
    pub csrf_token: String,
}

pub async fn render(
    engine: AppEngine,
    csrf_token: CsrfToken,
    Extension(current_user): Extension<SessionUser>,
    Query(query): Query<DeployTokensParams>,
) -> impl IntoResponse {
    let csrf_token_value = csrf_token.authenticity_token().unwrap();
    let page = query.page.unwrap_or(1);
    let page_size = query.size.unwrap_or(20);
    let (tokens, pager, alls) = AdminTokenVars::from_db(
        user_token::CreatedByCases::Deployment,
        page,
        page_size,
        "/admin/deploy-tokens",
    )
    .await
    .unwrap();

    let page_vars = PageVars::new(
        "Deploy Tokens".to_string(),
        "/admin/deploy-tokens".to_string(),
    );
    let user_vars = UserVars::new(&current_user);

    (
        csrf_token,
        RenderHtml(
            "admin/deploy-tokens.hbs",
            engine,
            AllVars {
                page: page_vars,
                user: user_vars,
                tokens_count: alls,
                tokens,
                pagination: pager,
                search: query.search.unwrap_or_default(),
                csrf_token: csrf_token_value,
            },
        ),
    )
        .into_response()
}
