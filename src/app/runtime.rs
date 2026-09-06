use std::{
    future::Future,
    sync::{Mutex, OnceLock},
};

fn runtime() -> &'static Mutex<Option<Result<tokio::runtime::Runtime, String>>> {
    static RUNTIME: OnceLock<Mutex<Option<Result<tokio::runtime::Runtime, String>>>> =
        OnceLock::new();
    RUNTIME.get_or_init(|| {
        Mutex::new(Some(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|error| format!("could not start SunTray async runtime: {error}")),
        ))
    })
}

pub(crate) fn spawn<F>(future: F) -> Result<tokio::task::JoinHandle<F::Output>, String>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let guard = runtime().lock().unwrap_or_else(|error| error.into_inner());
    match guard.as_ref() {
        Some(Ok(runtime)) => Ok(runtime.handle().spawn(future)),
        Some(Err(error)) => Err(error.clone()),
        None => Err("SunTray async runtime has been shut down".into()),
    }
}

pub(crate) fn shutdown() {
    let runtime = runtime()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .take()
        .and_then(Result::ok);
    if let Some(runtime) = runtime {
        runtime.shutdown_background();
    }
}
