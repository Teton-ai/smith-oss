use tokio::{select, sync::mpsc};
use tokio_util::sync::CancellationToken;
use tracing::info;

#[derive(Clone)]
pub struct ShutdownSignals {
    pub token: CancellationToken,
    pub _channel: mpsc::Sender<()>,
}

pub struct ShutdownHandler {
    shutdown_signals: ShutdownSignals,
    shutdown_wait: mpsc::Receiver<()>,
}

impl Default for ShutdownHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ShutdownHandler {
    pub fn new() -> Self {
        let (shutdown, shutdown_wait) = mpsc::channel(1);
        let token = CancellationToken::new();
        Self {
            shutdown_signals: ShutdownSignals {
                token,
                _channel: shutdown,
            },
            shutdown_wait,
        }
    }

    pub fn signals(&self) -> ShutdownSignals {
        self.shutdown_signals.clone()
    }

    pub async fn wait(mut self) {
        // create signal stream to handle SIGINT (aka ctrl+c)
        let mut sigint_sink =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).unwrap();

        // create signal stream to handle SIGTERM (aka how systemd stops this)
        let mut sigterm_sink =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();

        select! {
            _ = self.shutdown_signals.token.cancelled() => {
                info!("Shutdown requested by a task");
            }
            _ = sigint_sink.recv() => {
                info!("Received SIGINT, probably Ctrl+C was pressed");
                self.shutdown_signals.token.cancel();
            }
            _ = sigterm_sink.recv() => {
                info!("Received SIGTERM, probably systemd is stopping us");
                self.shutdown_signals.token.cancel();
            }
        }

        drop(self.shutdown_signals);
        self.shutdown_wait.recv().await;
    }
}
