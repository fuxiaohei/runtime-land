//! `asyncio` module provides async IO operations for Runtime.land functions.

use crate::http_service::land::asyncio;
use crate::http_service::land::asyncio::asyncio::TaskHandle;

/// sleep for ms milliseconds
pub fn sleep(ms: u32) -> TaskHandle {
    asyncio::asyncio::sleep(ms).unwrap()
}

/// is_ready returns true if the task is ready
pub fn is_ready(handle: TaskHandle) -> bool {
    asyncio::asyncio::take_ready(handle).unwrap()
}

/// select returns the latest ready task
pub fn select() -> Option<TaskHandle> {
    asyncio::asyncio::select().unwrap()
}

/// cancel the task
pub fn cancel(hande: TaskHandle) {
    asyncio::asyncio::cancel(hande);
}
