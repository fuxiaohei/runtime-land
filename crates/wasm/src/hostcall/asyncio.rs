use super::host::land::asyncio::asyncio::Host;
use super::host::land::asyncio::asyncio::TaskHandle;
use super::HttpContext;

#[async_trait::async_trait]
impl Host for HttpContext {
    async fn sleep(&mut self, ms: u32) -> Result<TaskHandle, ()> {
        let handle = self.new_asyncio_task(ms as u64 * 1000000); // convert ms to nanoseconds
        Ok(handle)
    }
    async fn take_ready(&mut self, handle: TaskHandle) -> Result<bool, ()> {
        Ok(self.is_asyncio_task_ready(handle))
    }
    async fn select(&mut self) -> Result<Option<TaskHandle>, ()> {
        Ok(self.select_asyncio_task())
    }
    async fn cancel(&mut self, handle: TaskHandle) {
        self.cancel_asyncio_task(handle)
    }
}
