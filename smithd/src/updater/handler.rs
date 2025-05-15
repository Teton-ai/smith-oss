use super::actor::Actor;
use super::actor::ActorMessage;
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

    pub async fn check_for_updates(&self) -> bool {
        // unwrap because if this fails then we are in a bad state
        self.sender.send(ActorMessage::Update).await.unwrap();
        true
    }

    pub async fn upgrade_device(&self) {
        // unwrap because if this fails then we are in a bad state
        self.sender.send(ActorMessage::Upgrade).await.unwrap();
    }

    pub async fn status(&self) -> String {
        let (rpc, receiver) = oneshot::channel();
        // unwrap because if this fails then we are in a bad state
        self.sender
            .send(ActorMessage::StatusReport { rpc })
            .await
            .unwrap();
        receiver.await.unwrap()
    }
}
