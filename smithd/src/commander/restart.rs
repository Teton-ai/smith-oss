use crate::utils::schema::{SafeCommandRequest, SafeCommandResponse, SafeCommandRx};
use tokio::process::Command;

pub(super) async fn execute(request: &SafeCommandRequest) -> SafeCommandResponse {
    let cmd = Command::new("shutdown").arg("-r").arg("+1").output().await;

    match cmd {
        Ok(output) => {
            let status = output.status.code().unwrap_or(1);
            // TODO: should we get stderr as well?
            let details = String::from_utf8_lossy(&output.stdout).to_string();
            SafeCommandResponse {
                id: request.id,
                command: SafeCommandRx::Restart { message: details },
                status,
            }
        }
        Err(e) => {
            let status = -1;
            let details = format!("Error executing command: {}", e);
            SafeCommandResponse {
                id: request.id,
                command: SafeCommandRx::Restart { message: details },
                status,
            }
        }
    }
}
