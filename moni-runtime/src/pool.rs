use crate::Worker;
use anyhow::Result;
use async_trait::async_trait;
use deadpool::managed;
use tokio::time::Instant;
use tracing::{debug, debug_span};

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
pub fn create_pool(path: &str) -> Result<WorkerPool> {
    let mgr = Manager::new(path);
    Ok(managed::Pool::builder(mgr).build().unwrap())
}

#[cfg(test)]
mod tests {
    use crate::host_call::http_impl::http_handler::Request;
    use crate::Context;

    #[tokio::test]
    async fn run_worker_pool_test() {
        let wasm_file = "../tests/data/rust_basic.component.wasm";
        let pool = super::create_pool(wasm_file).unwrap();

        let status = pool.status();
        assert_eq!(status.size, 0);
        assert_eq!(status.available, 0);

        {
            let mut worker = pool.get().await.unwrap();
            let worker = worker.as_mut();

            let headers: Vec<(&str, &str)> = vec![];
            let req = Request {
                method: "GET",
                uri: "/abc",
                headers: &headers,
                body: Some("xxxyyy".as_bytes()),
            };

            let resp = worker.handle_request(req, Context::new()).await.unwrap();
            assert_eq!(resp.status, 200);
            assert_eq!(resp.body, Some("Hello, World".as_bytes().to_vec()));

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
