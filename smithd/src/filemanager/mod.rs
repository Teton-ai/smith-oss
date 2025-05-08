use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
mod systemactions;
use crate::magic::MagicHandle;
use crate::shutdown::ShutdownSignals;
use crate::utils::network::NetworkClient;
use anyhow;
use tokio::{
    sync::{mpsc, oneshot},
    time,
};
use tracing::info;

struct FileManager {
    shutdown: ShutdownSignals,

    receiver: mpsc::Receiver<FileManagerMessage>,

    magic: MagicHandle,

    is_processing: Arc<AtomicBool>,

    network: NetworkClient,
}

enum FileManagerMessage {
    ExtractHere {
        file: String,

        rpc: oneshot::Sender<anyhow::Result<String>>,
    },

    Extract {
        file: String,

        target: String,

        rpc: oneshot::Sender<anyhow::Result<String>>,
    },

    ExecuteScript {
        file: String,

        arguments: Vec<String>,

        folder: Option<String>,

        rpc: oneshot::Sender<anyhow::Result<String>>,
    },

    ExecuteSystemCommand {
        command: String,

        arguments: Vec<String>,

        folder: Option<String>,

        rpc: oneshot::Sender<anyhow::Result<String>>,
    },
}

impl FileManager {
    fn new(
        shutdown: ShutdownSignals,

        receiver: mpsc::Receiver<FileManagerMessage>,

        magic: MagicHandle,
    ) -> Self {
        let network = NetworkClient::new();

        let is_processing = Arc::new(AtomicBool::new(false));

        Self {
            shutdown,

            receiver,

            magic,

            network,

            is_processing,
        }
    }

    async fn handle_message(&mut self, msg: FileManagerMessage) {
        match msg {
            FileManagerMessage::ExtractHere { file, rpc } => {
                self.is_processing.store(true, Ordering::SeqCst);

                let is_processing = self.is_processing.clone();

                tokio::spawn(async move {
                    let result = systemactions::extract_file_here(file.as_str()).await;

                    // Return results

                    _ = rpc.send(result);

                    // Reset status

                    is_processing.store(false, Ordering::SeqCst);
                });
            }

            FileManagerMessage::Extract { file, target, rpc } => {
                self.is_processing.store(true, Ordering::SeqCst);

                let is_processing = self.is_processing.clone();

                tokio::spawn(async move {
                    let result = systemactions::extract_file(file.as_str(), target.as_str()).await;

                    // Return results

                    _ = rpc.send(result);

                    // Reset status

                    is_processing.store(false, Ordering::SeqCst);
                });
            }

            FileManagerMessage::ExecuteScript {
                file,

                arguments,

                folder,

                rpc,
            } => {
                self.is_processing.store(true, Ordering::SeqCst);

                let is_processing = self.is_processing.clone();

                tokio::spawn(async move {
                    let result =
                        systemactions::execute_script(file.as_str(), arguments, folder.as_deref())
                            .await;

                    // Return results

                    _ = rpc.send(result);

                    // Reset status

                    is_processing.store(false, Ordering::SeqCst);
                });
            }

            FileManagerMessage::ExecuteSystemCommand {
                command,

                arguments,

                folder,

                rpc,
            } => {
                self.is_processing.store(true, Ordering::SeqCst);

                let is_processing = self.is_processing.clone();

                tokio::spawn(async move {
                    let result = systemactions::execute_system_command(
                        command.as_str(),
                        arguments,
                        folder.as_deref(),
                    )
                    .await;

                    // Return results

                    _ = rpc.send(result);

                    // Reset status

                    is_processing.store(false, Ordering::SeqCst);
                });
            }
        }
    }

    async fn run(&mut self) {
        info!("File manager task is running");

        let hostname = self.magic.get_server().await;

        self.network.set_hostname(hostname);

        loop {
            tokio::select! {


                Some(msg) = self.receiver.recv() => {


                    self.handle_message(msg).await;


                }


                _ = self.shutdown.token.cancelled() => {


                    loop {


                        if !self.is_processing.load(Ordering::SeqCst) {


                            break;


                        } else {


                            info!("Waiting for file manager task to finish");


                            time::sleep(time::Duration::from_secs(1)).await;





                        }


                    }


                    info!("File manager task shutting down gracefully");


                    break;


                }


            }
        }
    }
}

#[derive(Clone)]

pub struct FileManagerHandle {
    sender: mpsc::Sender<FileManagerMessage>,
}

impl FileManagerHandle {
    pub fn new(shutdown: ShutdownSignals, magic: MagicHandle) -> Self {
        let (sender, receiver) = mpsc::channel(8);

        let mut actor = FileManager::new(shutdown, receiver, magic);

        tokio::spawn(async move { actor.run().await });

        Self { sender }
    }

    pub async fn extract_here(&self, file: &str) -> anyhow::Result<String> {
        let (rpc, receiver) = oneshot::channel();

        self.sender
            .send(FileManagerMessage::ExtractHere {
                file: file.to_string(),

                rpc,
            })
            .await
            .unwrap();

        receiver.await.unwrap()
    }

    pub async fn extract(&self, file: &str, target: &str) -> anyhow::Result<String> {
        let (rpc, receiver) = oneshot::channel();

        self.sender
            .send(FileManagerMessage::Extract {
                file: file.to_string(),

                target: target.to_string(),

                rpc,
            })
            .await
            .unwrap();

        receiver.await.unwrap()
    }

    pub async fn execute_script(
        &self,

        file: &str,

        arguments: Vec<String>,

        folder: Option<&str>,
    ) -> anyhow::Result<String> {
        let (rpc, receiver) = oneshot::channel();

        self.sender
            .send(FileManagerMessage::ExecuteScript {
                file: file.to_string(),

                arguments,

                folder: folder.map(|f| f.to_string()),

                rpc,
            })
            .await
            .unwrap();

        receiver.await.unwrap()
    }

    pub async fn execute_system_command(
        &self,

        command: &str,

        arguments: Vec<String>,

        folder: Option<&str>,
    ) -> anyhow::Result<String> {
        let (rpc, receiver) = oneshot::channel();

        self.sender
            .send(FileManagerMessage::ExecuteSystemCommand {
                command: command.to_string(),

                arguments,

                folder: folder.map(|f| f.to_string()),

                rpc,
            })
            .await
            .unwrap();

        receiver.await.unwrap()
    }
}
