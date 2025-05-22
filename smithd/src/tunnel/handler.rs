use super::actor::{Actor, ActorMessage};
use crate::magic::MagicHandle;
use crate::shutdown::ShutdownSignals;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct Handler {
    sender: mpsc::Sender<ActorMessage>,
}

impl Handler {
    pub fn new(shutdown: ShutdownSignals, magic: MagicHandle) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let mut actor = Actor::new(shutdown, receiver, magic);
        tokio::spawn(async move { actor.run().await });

        Self { sender }
    }

    pub async fn start_tunnel(&self, port: Option<u16>) -> u16 {
        let local = port.unwrap_or(22);
        let (sender, receiver) = oneshot::channel();
        let msg = ActorMessage::ForwardPort {
            local,
            remote: sender,
        };
        _ = self.sender.send(msg).await;
        receiver.await.unwrap()
    }

    pub async fn stop_ssh_tunnel(&self) {
        let local = 22;
        let msg = ActorMessage::ClosePort { local };
        _ = self.sender.send(msg).await;
    }
}
