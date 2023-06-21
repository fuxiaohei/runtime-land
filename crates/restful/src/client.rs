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
    addr_url: Url,
    client: reqwest::Client,
}

impl Client {
    pub fn new(addr: String, auth_token: String) -> anyhow::Result<Self> {
        let addr_url = Url::parse(&addr)?;
        let mut auth_headers = header::HeaderMap::new();
        auth_headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(format!("Bearer {}", auth_token).as_str()).unwrap(),
        );
        let client = reqwest::Client::builder()
            .default_headers(auth_headers)
            .build()?;
        Ok(Client { addr_url, client })
    }

    pub async fn fetch_project(
        &self,
        name: String,
        language: String,
    ) -> Result<Option<params::ProjectData>, ClientError> {
        let mut url = self.addr_url.clone();
        url.set_path("/v1/project");
        url.query_pairs_mut().append_pair("name", &name);
        url.query_pairs_mut().append_pair("language", &language);

        let resp = self.client.get(url).send().await?;
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
        let mut url = self.addr_url.clone();
        url.set_path("/v1/project");
        let payload = params::FetchProjectRequest { name, language };
        let resp = self.client.post(url).json(&payload).send().await?;
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
        let mut url = self.addr_url.clone();
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
        let resp = self.client.post(url).json(&payload).send().await?;
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
        let mut url = self.addr_url.clone();
        url.set_path("/v1/deployment/publish");
        let payload = params::PublishDeployRequest {
            deploy_id,
            deploy_uuid,
        };
        let resp = self.client.post(url).json(&payload).send().await?;
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
