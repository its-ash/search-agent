use std::future::Future;

pub async fn run_named_worker<F, Fut>(name: &str, f: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>,
{
    tracing::info!("worker-start: {name}");
    f().await;
    tracing::info!("worker-stop: {name}");
}
