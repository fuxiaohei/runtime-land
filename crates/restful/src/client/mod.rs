use crate::params;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use reqwest::header;
use url::Url;

#[derive(Debug)]
pub struct ClientError {
    pub status: u16,
    pub message: String,
}

impl<E> From<E> for ClientError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self {
            status: 0,
            message: err.into().to_string(),
        }
    }
}

pub struct Client {
    addr: String,
    auth_headers: reqwest::header::HeaderMap,
}

impl Client {
    pub fn new(addr: String, auth_token: String) -> Self {
        let mut auth_headers = header::HeaderMap::new();
        auth_headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(format!("Bearer {}", auth_token).as_str()).unwrap(),
        );
        Client { addr, auth_headers }
    }

    pub async fn fetch_project(
        &self,
        name: String,
        language: String,
    ) -> Result<Option<params::ProjectData>, ClientError> {
        let mut url = Url::parse(&self.addr).map_err(|e| ClientError {
            status: 0,
            message: e.to_string(),
        })?;
        // add value to query string
        url.set_path("/v1/project");
        url.query_pairs_mut().append_pair("name", &name);
        url.query_pairs_mut().append_pair("language", &language);

        let resp = reqwest::Client::builder()
            .default_headers(self.auth_headers.clone())
            .build()?
            .get(url)
            .send()
            .await?;
        if resp.status().is_success() {
            let data = resp.json::<params::ProjectData>().await?;
            return Ok(Some(data));
        }
        // if project is not exist, return None
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        Err(ClientError {
            status: resp.status().into(),
            message: resp.text().await?,
        })
    }

    pub async fn create_project(
        &self,
        name: String,
        language: String,
    ) -> Result<params::ProjectData, ClientError> {
        let mut url = Url::parse(&self.addr).map_err(|e| ClientError {
            status: 0,
            message: e.to_string(),
        })?;
        url.set_path("/v1/project");
        let payload = params::FetchProjectRequest { name, language };
        let resp = reqwest::Client::builder()
            .default_headers(self.auth_headers.clone())
            .build()?
            .post(url)
            .json(&payload)
            .send()
            .await?;
        if resp.status().is_success() {
            let data = resp.json::<params::ProjectData>().await?;
            return Ok(data);
        }
        Err(ClientError {
            status: resp.status().into(),
            message: resp.text().await?,
        })
    }

    pub async fn create_deploy(
        &self,
        project_name: String,
        project_uuid: String,
        deploy_chunk: Vec<u8>,
        deploy_content_type: String,
    ) -> Result<params::DeploymentData, ClientError> {
        let mut url = Url::parse(&self.addr).map_err(|e| ClientError {
            status: 0,
            message: e.to_string(),
        })?;
        url.set_path("/v1/deployment");
        let deploy_name: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        let payload = params::CreateDeployRequest {
            project_name,
            project_uuid,
            deploy_chunk,
            deploy_name: deploy_name.to_lowercase(),
            deploy_content_type,
        };
        let resp = reqwest::Client::builder()
            .default_headers(self.auth_headers.clone())
            .build()?
            .post(url)
            .json(&payload)
            .send()
            .await?;
        if resp.status().is_success() {
            let data = resp.json::<params::DeploymentData>().await?;
            return Ok(data);
        }
        Err(ClientError {
            status: resp.status().into(),
            message: resp.text().await?,
        })
    }

    pub async fn publish_deploy(
        &self,
        deploy_id: i32,
        deploy_uuid: String,
    ) -> Result<params::DeploymentData, ClientError> {
        let mut url = Url::parse(&self.addr).map_err(|e| ClientError {
            status: 0,
            message: e.to_string(),
        })?;
        url.set_path("/v1/deployment/publish");
        let payload = params::PublishDeployRequest {
            deploy_id,
            deploy_uuid,
        };
        let resp = reqwest::Client::builder()
        .default_headers(self.auth_headers.clone())
        .build()?
        .post(url)
        .json(&payload)
        .send()
        .await?;
    if resp.status().is_success() {
        let data = resp.json::<params::DeploymentData>().await?;
        return Ok(data);
    }
    Err(ClientError {
        status: resp.status().into(),
        message: resp.text().await?,
    })
    }
}
