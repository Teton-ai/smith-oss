// TODO: Move this file to schema module
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SafeCommandRequest {
    pub id: i32,
    pub command: SafeCommandTx,
    pub continue_on_error: bool,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub enum SafeCommandTx {
    #[default]
    Ping,
    OpenTunnel {
        port: Option<u16>,
    },
    CloseTunnel,
}
