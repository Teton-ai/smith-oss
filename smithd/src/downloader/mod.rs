use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
mod download;
use crate::magic::MagicHandle;
use crate::shutdown::ShutdownSignals;
use crate::utils::network::NetworkClient;
use anyhow;
use download::download_package;
use tokio::{
    sync::{mpsc, oneshot},
    time,
};
use tracing::info;

struct Downloader {
    shutdown: ShutdownSignals,
    receiver: mpsc::Receiver<DownloaderMessage>,
    magic: MagicHandle,
    is_downloading: Arc<AtomicBool>,
    network: NetworkClient,
    force_stop: Arc<AtomicBool>,
    timeout: u64,
}

#[derive(Debug)]

enum DownloaderMessage {
    Download {
        remote_file: String,

        local_file: String,

        rate: f64,

        rpc: oneshot::Sender<anyhow::Result<String>>,
    },
}

impl Downloader {
    fn new(
        shutdown: ShutdownSignals,

        receiver: mpsc::Receiver<DownloaderMessage>,

        magic: MagicHandle,

        timeout: u64,
    ) -> Self {
        let network = NetworkClient::new();

        let force_stop = Arc::new(AtomicBool::new(false));

        let is_downloading = Arc::new(AtomicBool::new(false));

        Self {
            shutdown,

            receiver,

            magic,

            network,

            is_downloading,

            force_stop,

            timeout,
        }
    }

    async fn handle_message(&mut self, msg: DownloaderMessage) {
        match msg {
            DownloaderMessage::Download {
                remote_file,

                local_file,

                rate,

                rpc,
            } => {
                self.is_downloading.store(true, Ordering::SeqCst);

                let magic = self.magic.clone();

                let force_stop = self.force_stop.clone();

                let is_downloading = self.is_downloading.clone();

                tokio::spawn(async move {
                    // Do the download

                    let result =
                        download_package(magic, remote_file, local_file, rate, force_stop).await;

                    // Send the result back

                    let _ = rpc.send(result);

                    // Reset status

                    is_downloading.store(false, Ordering::SeqCst);
                });
            }
        }
    }

    async fn run(&mut self) {
        info!("Download task is running");

        let hostname = self.magic.get_server().await;

        self.network.set_hostname(hostname);

        loop {
            tokio::select! {
                Some(msg) = self.receiver.recv() => {
                    info!("Received message: {:?}", msg);
                    self.handle_message(msg).await;
                }

                _ = self.shutdown.token.cancelled() => {
                    let mut count = 1;

                    loop {
                        if !self.is_downloading.load(Ordering::SeqCst) {
                            break;
                        } else {
                            info!("Waiting for download task to finish");
                            time::sleep(time::Duration::from_secs(1)).await;
                            if count > self.timeout {
                                self.force_stop.store(true, Ordering::SeqCst);
                            }
                            count += 1;
                        }
                    }
                    info!("Download task shutting down gracefully");
                    break;
                }
            }
        }
    }
}

#[derive(Clone)]

pub struct DownloaderHandle {
    sender: mpsc::Sender<DownloaderMessage>,
}

impl DownloaderHandle {
    pub fn new(shutdown: ShutdownSignals, magic: MagicHandle) -> Self {
        let (sender, receiver) = mpsc::channel(8);

        let timeout = 60; // 60 second timeout

        let mut actor = Downloader::new(shutdown, receiver, magic, timeout);

        tokio::spawn(async move { actor.run().await });

        Self { sender }
    }

    pub async fn download(
        &self,

        remote_file: &str,

        local_file: &str,

        rate: f64,
    ) -> anyhow::Result<String> {
        // unwrap because if this fails then we are in a bad state

        let (rpc, receiver) = oneshot::channel();

        self.sender
            .send(DownloaderMessage::Download {
                remote_file: remote_file.to_string(),

                local_file: local_file.to_string(),

                rate,

                rpc,
            })
            .await
            .unwrap();

        receiver.await.unwrap()
    }
}
