use super::host::land::http::outgoing::{Host, Request, RequestError, RequestOptions, Response};
use super::HttpContext;

#[async_trait::async_trait]
impl Host for HttpContext {
    async fn fetch_request(
        &mut self,
        _req: Request,
        _options: RequestOptions,
    ) -> wasmtime::Result<Result<Response, RequestError>> {
        return Ok(Err(RequestError::NetworkError));
    }
}
