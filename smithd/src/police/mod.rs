//! Police actor
//!
//! Others actors will send messages to the police actor when they think
//! that something is wrong. The police actor will then take action to solve
//! the problem. Right now the only action is to restart the agent.
//!
//! It does this by issuing a dealayed restart after 5 minutes. If the problem
//! is solved before the restart is issued, the restart is cancelled. 🤞
//!
use crate::shutdown::ShutdownSignals;
use tokio::process::Command;
use tokio::runtime::Handle;
use tokio::sync::{mpsc, oneshot};
use tracing::error;
use tracing::{info, warn};

struct Police {
    shutdown: ShutdownSignals,
    should_restart: bool,
    restart: Option<tokio::task::JoinHandle<()>>,
    receiver: mpsc::Receiver<PoliceMessage>,
    next_id: u32,
    problems: Vec<u32>,
}
enum PoliceMessage {
    ProblemStarting {
        respond_to: oneshot::Sender<Option<u32>>,
    },
    ProblemSolved {
        id: u32,
    },
}

impl Police {
    fn new(shutdown: ShutdownSignals, receiver: mpsc::Receiver<PoliceMessage>) -> Self {
        Police {
            shutdown,
            should_restart: false,
            restart: None,
            receiver,
            next_id: 0,
            problems: Vec::new(),
        }
    }
    fn handle_message(&mut self, msg: PoliceMessage) {
        match msg {
            PoliceMessage::ProblemStarting { respond_to } => {
                // There is no restart scheduled, so we will do it in 5 minutes
                let response = if self.should_restart {
                    self.next_id += 1;
                    self.problems.push(self.next_id);
                    if self.restart.is_none() {
                        let handle = Handle::current();
                        // spawn doesn need to be awaited in order to run
                        let restart_handle = handle.spawn(async {
                            warn!("Restarting in 5 minutes");
                            // Sleep for 5 minutes instead of scheduling reboot right away because
                            // when scheduling we will no longer be able to login via ssh as a user
                            // might locks us out of the system
                            tokio::time::sleep(std::time::Duration::from_secs(5 * 60)).await;
                            error!("Restarting now!");
                            Command::new("reboot")
                                .arg("now")
                                .spawn()
                                .expect("Failed to spawn shutdown command");
                        });
                        self.restart = Some(restart_handle);
                    } else {
                        warn!("Restart already scheduled");
                    }
                    Some(self.next_id)
                } else {
                    warn!("Restart not to be scheduled yet");
                    None
                };

                _ = respond_to.send(response);
            }
            PoliceMessage::ProblemSolved { id } => {
                // pop id from problems
                self.problems.retain(|&x| x != id);

                // If there are no more problems, cancel the restart
                if self.restart.is_some() && self.problems.is_empty() {
                    info!("Problem solved, restart aborted");
                    // unwrap here is safe because we just checked that it is not None
                    self.restart.take().unwrap().abort();
                }
            }
        }
    }

    async fn run(&mut self) {
        info!("Police runnning");

        let mut enable_by_default =
            tokio::time::interval(tokio::time::Duration::from_secs(60 * 15));

        // the first tick is immediate
        enable_by_default.tick().await;

        loop {
            tokio::select! {
                Some(msg) = self.receiver.recv() => {
                    self.handle_message(msg);
                }
                _ = enable_by_default.tick() => {
                    info!("Enabling police restarts by default");
                    self.should_restart = true;
                }
                _ = self.shutdown.token.cancelled() => {
                    break;
                }
            }
        }

        info!("Bouncer task shut down");
    }
}

#[derive(Clone)]
pub struct PoliceHandle {
    sender: mpsc::Sender<PoliceMessage>,
}

impl PoliceHandle {
    pub fn new(shutdown: ShutdownSignals) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let mut actor = Police::new(shutdown, receiver);
        tokio::spawn(async move { actor.run().await });

        Self { sender }
    }

    pub async fn report_problem_starting(&self) -> Option<u32> {
        let (send, recv) = oneshot::channel();
        let msg = PoliceMessage::ProblemStarting { respond_to: send };
        _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    pub async fn report_problem_solved(&self, id: u32) {
        let msg = PoliceMessage::ProblemSolved { id };
        _ = self.sender.send(msg).await;
    }
}
