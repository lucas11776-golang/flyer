use tokio::signal;

pub(crate) async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(windows)]
    let terminate = async {
        // Windows console close / Ctrl+Break signal handling
        let mut ctrl_break = signal::windows::ctrl_break()
            .expect("failed to install Ctrl+Break handler");
        let mut ctrl_close = signal::windows::ctrl_close()
            .expect("failed to install Ctrl+Close handler");
        let mut ctrl_shutdown = signal::windows::ctrl_shutdown()
            .expect("failed to install Ctrl+Shutdown handler");

        tokio::select! {
            _ = ctrl_break.recv() => {},
            _ = ctrl_close.recv() => {},
            _ = ctrl_shutdown.recv() => {},
        }
    };

    // #[cfg(not(any(unix, windows)))]
    // let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("==> Received Ctrl+C, starting graceful shutdown...");
        }
        _ = terminate => {
            println!("==> Received termination signal, starting graceful shutdown...");
        }
    }
}