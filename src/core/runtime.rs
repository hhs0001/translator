use std::sync::OnceLock;

use tokio::runtime::{Handle, Runtime};

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("translator-tokio")
            .build()
            .expect("failed to create tokio runtime")
    })
}

/// Handle to the process-wide multi-thread Tokio runtime.
/// Spawn LLM / reqwest work onto this instead of blocking the GPUI UI thread.
pub fn handle() -> &'static Handle {
    runtime().handle()
}

/// Drive a future to completion on the background runtime.
/// Must not be called from a Tokio worker thread (or from the GPUI UI thread).
pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    runtime().block_on(future)
}

pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    handle().spawn(future)
}

#[cfg(test)]
mod tests {
    #[test]
    fn handle_is_available() {
        let _ = super::handle();
    }
}
