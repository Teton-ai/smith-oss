use crate::config::Config;
use async_trait::async_trait;
use russh::{ChannelMsg, Disconnect, client, keys::key};
use std::{env, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

struct Client {}

#[async_trait]
impl client::Handler for Client {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

/// This struct is a convenience wrapper
/// around a russh client
/// that handles the input/output event loop
pub struct Session {
    session: client::Handle<Client>,
}

impl Session {
    pub async fn connect(
        username: String,
        password: String,
        port: u16,
        config: &Config,
    ) -> anyhow::Result<Self> {
        let sh = Client {};

        let client_config = Arc::new(client::Config::default());

        let domain = config.tunnel_server();

        let mut session = client::connect(client_config, (domain, port), sh).await?;

        let auth = session.authenticate_password(username, password).await?;

        if !auth {
            return Err(anyhow::anyhow!("Failed to authenticate"));
        }

        Ok(Self { session })
    }

    pub async fn call(&mut self) -> anyhow::Result<u32> {
        let mut channel = self.session.channel_open_session().await?;

        // This example doesn't terminal resizing after the connection is established
        let (w, h) = termion::terminal_size()?;

        // Request an interactive PTY from the server
        channel
            .request_pty(
                false,
                &env::var("TERM").unwrap_or("xterm".into()),
                w as u32,
                h as u32,
                0,
                0,
                &[], // ideally you want to pass the actual terminal modes here
            )
            .await?;

        channel.request_shell(true).await?;

        let mut stdin = tokio_fd::AsyncFd::try_from(0)?;
        let mut stdout = tokio_fd::AsyncFd::try_from(1)?;
        let mut buf = vec![0; 1024];
        let mut stdin_closed = false;

        loop {
            // Handle one of the possible events:
            tokio::select! {
                // There's terminal input available from the user
                r = stdin.read(&mut buf), if !stdin_closed => {
                    match r {
                        Ok(0) => {
                            stdin_closed = true;
                            channel.eof().await?;
                        },
                        // Send it to the server
                        Ok(n) => channel.data(&buf[..n]).await?,
                        Err(e) => return Err(e.into()),
                    };
                },
                // There's an event available on the session channel
                Some(msg) = channel.wait() => {
                    match msg {
                        // Write data to the terminal
                        ChannelMsg::Data { ref data } => {
                            stdout.write_all(data).await?;
                            stdout.flush().await?;
                        }
                        ChannelMsg::Eof => {
                            break;
                        }
                        _ => {}
                    }
                },
            }
        }
        Ok(0)
    }

    pub async fn close(&mut self) -> anyhow::Result<()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

pub async fn connect_local_port_to_remote_port(
    config: &Config,
    username: String,
    password: String,
    port: u16,
) -> anyhow::Result<()> {
    let tunnel_server = config.tunnel_server();
    let domain = format!("{username}@{tunnel_server}");

    let child = Command::new("sshpass")
        .arg("-p")
        .arg(password)
        .arg("ssh")
        .arg("-o")
        .arg("StrictHostKeyChecking=no")
        .arg("-L")
        .arg("9222:localhost:9222")
        .arg(&domain)
        .arg("-p")
        .arg(format!("{}", port))
        .kill_on_drop(true)
        .spawn()?;

    let output = child.wait_with_output().await?;

    println!("output: {:?}", output);

    Ok(())
}
