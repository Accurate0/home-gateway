use tokio::signal;
use tokio_util::sync::CancellationToken;

pub async fn axum_shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}

pub fn handle_cancellation(cancellation_token: CancellationToken) {
    tokio::spawn(async move {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        };

        cancellation_token.cancel();
    });
}
