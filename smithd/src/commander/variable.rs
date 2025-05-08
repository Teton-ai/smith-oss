use crate::utils::schema::{SafeCommandResponse, SafeCommandRx};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

pub(super) async fn execute(id: i32, variables: HashMap<String, String>) -> SafeCommandResponse {
    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("/root/.teton_environment")
    {
        for (key, value) in variables {
            _ = writeln!(file, "{}={}", key, value);
        }
        return SafeCommandResponse {
            id,
            command: SafeCommandRx::UpdateVariables,
            status: 0,
        };
    }
    SafeCommandResponse {
        id,
        command: SafeCommandRx::UpdateVariables,
        status: -1,
    }
}
