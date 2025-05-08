//! Welcome to the documentation center for Agent smith
//!
//! Agent smith is a binary that is tasked with running in our embedded devices
//! its responsible for monitoring the health of the device and reporting it to
//! our backend.
//! The binary is run as a systemd service on the devices.
//!

use smith::control;
use smith::daemon;

#[tokio::main]
async fn main() {
    // setup logging
    tracing_subscriber::fmt::init();

    let daemon_should_run = control::execute().await;

    if daemon_should_run {
        daemon::run().await;
    }
}
