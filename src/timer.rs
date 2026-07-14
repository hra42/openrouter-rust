//! Runtime-neutral sleep support for retry and stream backoff.

use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

#[cfg(all(target_arch = "wasm32", feature = "browser"))]
pub(crate) async fn sleep(duration: Duration) {
    let millis = duration.as_millis().min(u32::MAX as u128) as u32;
    gloo_timers::future::TimeoutFuture::new(millis).await;
}
