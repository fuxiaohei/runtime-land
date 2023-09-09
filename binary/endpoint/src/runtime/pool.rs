use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use deadpool::managed;
use land_worker::Worker;
use lazy_static::lazy_static;
use moka::sync::Cache;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use tracing::{debug, debug_span, info};

#[derive(Debug)]
pub struct Manager {
    path: String,
}

impl Manager {
    pub fn new(path: &str) -> Self {
        Self {
            path: String::from(path),
        }
    }
}

#[async_trait]
impl managed::Manager for Manager {
    type Type = Worker;
    type Error = anyhow::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let start_time = Instant::now();
        let worker = Worker::new(&self.path).await?;
        debug_span!("[Worker]", path = &self.path).in_scope(|| {
            debug!(eplased = ?start_time.elapsed(), "create, ok");
        });
        Ok(worker)
    }

    async fn recycle(&self, _obj: &mut Self::Type) -> managed::RecycleResult<Self::Error> {
        Ok(())
    }
}

pub type WorkerPool = managed::Pool<Manager>;

/// create_pool creates a pool
fn create_pool(path: &str) -> Result<WorkerPool> {
    let mgr = Manager::new(path);
    Ok(managed::Pool::builder(mgr).build().unwrap())
}

lazy_static! {
    pub static ref WASM_INSTANCES: Cache<String,Arc<WorkerPool> > = Cache::builder()
    // Time to live (TTL): 1 hours
    .time_to_live(Duration::from_secs( 60 * 60))
    // Time to idle (TTI):  10 minutes
    .time_to_idle(Duration::from_secs(10 * 60))
    // Create the cache.
    .build();
}

pub async fn prepare_worker_pool(key: &str) -> Result<Arc<WorkerPool>> {
    let mut instances_pool = WASM_INSTANCES.get(key);

    if instances_pool.is_some() {
        return Ok(instances_pool.unwrap());
    }

    if !land_storage::is_exist(key).await? {
        return Err(anyhow!("pool key not found: {}", key));
    }
    let binary = land_storage::read(key).await?;

    // write binary to local file
    let mut path = std::env::temp_dir();
    path.push(key);
    // create parent dir
    let parent = path.parent().unwrap();
    std::fs::create_dir_all(parent)?;
    std::fs::write(&path, binary)?;
    debug!("wasm temp binary write to {}", path.display());

    // create wasm worker pool
    let pool = create_pool(path.to_str().unwrap())?;
    WASM_INSTANCES.insert(key.to_string(), Arc::new(pool));

    instances_pool = WASM_INSTANCES.get(key);
    info!("worker pool created");

    Ok(instances_pool.unwrap())
}

#[cfg(test)]
mod tests {
    use hyper::Body;
    use land_worker::hostcall::Request;
    use land_worker::Context;

    #[tokio::test]
    async fn run_worker_pool_test() {
        let wasm_file = "../../tests/rust_test.component.wasm";
        let pool = super::create_pool(wasm_file).unwrap();

        let status = pool.status();
        assert_eq!(status.size, 0);
        assert_eq!(status.available, 0);

        {
            let mut worker = pool.get().await.unwrap();
            let worker = worker.as_mut();

            let mut context = Context::default();
            let body = Body::from("test request body");
            let body_handle = context.set_body(body);

            let headers: Vec<(String, String)> = vec![];
            let req = Request {
                method: "GET".to_string(),
                uri: "/abc".to_string(),
                headers,
                body: Some(body_handle),
            };

            let (resp, _body) = worker.handle_request(req, context).await.unwrap();
            assert_eq!(resp.status, 200);
            assert_eq!(resp.body, Some(2));

            let headers = resp.headers;
            for (key, value) in headers {
                if key == "X-Request-Method" {
                    assert_eq!(value, "GET");
                }
                if key == "X-Request-Url" {
                    assert_eq!(value, "/abc");
                }
            }
        }

        let status = pool.status();
        assert_eq!(status.size, 1);
        assert_eq!(status.available, 1);
    }
}
