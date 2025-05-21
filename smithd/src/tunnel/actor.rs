use crate::magic::MagicHandle;
use crate::shutdown::ShutdownSignals;
use bore_cli::client::Client;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{self, Duration};
use tracing::{error, info};

struct ForwardConnection {
    created_at: time::Instant,
    remote: u16,
    task: tokio::task::JoinHandle<()>,
}

pub enum ActorMessage {
    ForwardPort {
        local: u16,
        remote: oneshot::Sender<u16>,
    },
    ClosePort {
        local: u16,
    },
}

// Tunnel actor
pub struct Actor {
    shutdown: ShutdownSignals,
    receiver: mpsc::Receiver<ActorMessage>,
    magic: MagicHandle,
    ports: HashMap<u16, ForwardConnection>,
}

impl Actor {
    pub fn new(
        shutdown: ShutdownSignals,
        receiver: mpsc::Receiver<ActorMessage>,
        magic: MagicHandle,
    ) -> Self {
        Self {
            shutdown,
            receiver,
            magic,
            ports: HashMap::new(),
        }
    }

    async fn handle_message(&mut self, msg: ActorMessage, server: &str, secret: &str) {
        match msg {
            ActorMessage::ForwardPort { local, remote } => {
                // check if there is already a ForwardConnection for this port
                if self.ports.contains_key(&local) {
                    error!("Port {} is already forwarded", local);
                    let remote_port = self.ports.get(&local).unwrap().remote;
                    remote.send(remote_port).unwrap();
                    return;
                }

                let server = server.to_owned();
                let secret = secret.to_owned();
                let (tx, rx) = oneshot::channel();
                let handle = tokio::spawn(async move {
                    let client = Client::new("localhost", local, &server, 0, Some(&secret)).await;

                    match client {
                        Ok(client) => {
                            info!("Forwarding port {} to {}", local, client.remote_port());
                            _ = tx.send(client.remote_port());
                            // this will block until the connection is closed
                            _ = client.listen().await;
                        }
                        Err(e) => {
                            error!("Failed to forward port {}: {}", local, e);
                            _ = tx.send(0);
                        }
                    }
                });

                let port = rx.await.unwrap_or_default();
                _ = remote.send(port);

                if port == 0 {
                    return;
                }

                self.ports.insert(
                    local,
                    ForwardConnection {
                        remote: port,
                        task: handle,
                        created_at: time::Instant::now(),
                    },
                );
            }
            ActorMessage::ClosePort { local } => {
                if let Some(conn) = self.ports.remove(&local) {
                    conn.task.abort();
                }
            }
        }
    }

    async fn timeout_old_tunnels(&mut self) {
        let now = time::Instant::now();
        let mut to_remove = Vec::new();
        let timeout_duration = Duration::from_secs(60 * 30);

        for (port, conn) in &self.ports {
            if now.duration_since(conn.created_at) > timeout_duration {
                to_remove.push(*port);
            }
        }

        for port in to_remove {
            info!("Closing port {} due to timeout", port);
            if let Some(conn) = self.ports.remove(&port) {
                conn.task.abort();
            }
        }
    }

    pub async fn run(&mut self) {
        info!("Tunnel task is runnning");

        let details = self.magic.get_tunnel_details().await;

        // check tunnels still open every 10 minutes
        let mut timeout_tunnels = time::interval(Duration::from_secs(60 * 10));
        timeout_tunnels.tick().await;

        loop {
            tokio::select! {
                Some(msg) = self.receiver.recv() => {
                    self.handle_message(msg, &details.server, &details.secret).await;
                }
                _ = timeout_tunnels.tick() => {
                    self.timeout_old_tunnels().await;
                }
                _ = self.shutdown.token.cancelled() => {
                    break;
                }
            }
        }

        info!("Tunnel task shutting down");
    }
}
