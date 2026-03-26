use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use ciab_core::error::{CiabError, CiabResult};
use ssh_key::{Algorithm, PrivateKey};
use tokio::sync::mpsc;

/// SSH client handler that accepts all host keys (suitable for ephemeral sandboxes).
struct SshHandler;

#[async_trait]
impl russh::client::Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // Accept all host keys for ephemeral sandbox instances
        Ok(true)
    }
}

/// Wrapper around a russh client session handle.
pub struct SshSession {
    handle: russh::client::Handle<SshHandler>,
}

impl SshSession {
    /// Connect to a remote host via SSH using public key authentication.
    pub async fn connect(
        host: &str,
        port: u16,
        user: &str,
        key: Arc<PrivateKey>,
    ) -> CiabResult<Self> {
        let config = russh::client::Config {
            inactivity_timeout: Some(Duration::from_secs(300)),
            ..Default::default()
        };
        let config = Arc::new(config);

        let addr = (host.to_string(), port);
        let mut session = russh::client::connect(config, addr, SshHandler)
            .await
            .map_err(|e| CiabError::SshError(format!("connection failed: {e}")))?;

        let auth_ok = session
            .authenticate_publickey(user, key)
            .await
            .map_err(|e| CiabError::SshError(format!("auth failed: {e}")))?;

        if !auth_ok {
            return Err(CiabError::SshError(
                "public key authentication rejected".to_string(),
            ));
        }

        Ok(Self { handle: session })
    }

    /// Execute a command and collect stdout, stderr, and exit code.
    pub async fn exec(&self, command: &str) -> CiabResult<(String, String, i32)> {
        let mut channel = self
            .handle
            .channel_open_session()
            .await
            .map_err(|e| CiabError::SshError(format!("open session: {e}")))?;

        channel
            .exec(true, command)
            .await
            .map_err(|e| CiabError::SshError(format!("exec: {e}")))?;

        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();
        let mut exit_code: Option<u32> = None;

        loop {
            let Some(msg) = channel.wait().await else {
                break;
            };
            match msg {
                russh::ChannelMsg::Data { ref data } => {
                    stdout_buf.extend_from_slice(data);
                }
                russh::ChannelMsg::ExtendedData { ref data, ext } => {
                    if ext == 1 {
                        // stderr
                        stderr_buf.extend_from_slice(data);
                    }
                }
                russh::ChannelMsg::ExitStatus { exit_status } => {
                    exit_code = Some(exit_status);
                }
                russh::ChannelMsg::Eof | russh::ChannelMsg::Close => {}
                _ => {}
            }
        }

        let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
        let stderr = String::from_utf8_lossy(&stderr_buf).to_string();
        let code = exit_code.unwrap_or(255) as i32;

        Ok((stdout, stderr, code))
    }

    /// Execute a command and stream stdout lines through an mpsc channel.
    pub async fn exec_streaming(
        &self,
        command: &str,
    ) -> CiabResult<mpsc::Receiver<String>> {
        let mut channel = self
            .handle
            .channel_open_session()
            .await
            .map_err(|e| CiabError::SshError(format!("open session: {e}")))?;

        channel
            .exec(true, command)
            .await
            .map_err(|e| CiabError::SshError(format!("exec: {e}")))?;

        let (tx, rx) = mpsc::channel::<String>(256);

        tokio::spawn(async move {
            let mut buf = Vec::new();
            loop {
                let Some(msg) = channel.wait().await else {
                    break;
                };
                match msg {
                    russh::ChannelMsg::Data { ref data } => {
                        buf.extend_from_slice(data);
                        // Emit complete lines
                        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                            let line = String::from_utf8_lossy(&buf[..pos]).to_string();
                            buf.drain(..=pos);
                            if tx.send(line).await.is_err() {
                                return;
                            }
                        }
                    }
                    russh::ChannelMsg::Eof | russh::ChannelMsg::Close => break,
                    _ => {}
                }
            }
            // Flush remaining partial line
            if !buf.is_empty() {
                let line = String::from_utf8_lossy(&buf).to_string();
                let _ = tx.send(line).await;
            }
        });

        Ok(rx)
    }

    /// Write a file on the remote host by base64-encoding content through a shell command.
    pub async fn write_file(&self, path: &str, content: &[u8]) -> CiabResult<()> {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(content);
        let cmd = format!(
            "mkdir -p \"$(dirname {path})\" && echo '{encoded}' | base64 -d > {path}"
        );
        let (_, stderr, code) = self.exec(&cmd).await?;
        if code != 0 {
            return Err(CiabError::SshError(format!(
                "write_file failed (exit {code}): {stderr}"
            )));
        }
        Ok(())
    }

    /// Read a file from the remote host, returning raw bytes.
    pub async fn read_file(&self, path: &str) -> CiabResult<Vec<u8>> {
        use base64::Engine;
        let cmd = format!("base64 < {path}");
        let (stdout, stderr, code) = self.exec(&cmd).await?;
        if code != 0 {
            return Err(CiabError::SshError(format!(
                "read_file failed (exit {code}): {stderr}"
            )));
        }
        let cleaned: String = stdout.chars().filter(|c| !c.is_whitespace()).collect();
        base64::engine::general_purpose::STANDARD
            .decode(&cleaned)
            .map_err(|e| CiabError::SshError(format!("base64 decode: {e}")))
    }

    /// List files in a directory on the remote host.
    pub async fn list_files(&self, path: &str) -> CiabResult<String> {
        let cmd = format!("ls -la {path}");
        let (stdout, stderr, code) = self.exec(&cmd).await?;
        if code != 0 {
            return Err(CiabError::SshError(format!(
                "list_files failed (exit {code}): {stderr}"
            )));
        }
        Ok(stdout)
    }

    /// Disconnect the SSH session.
    pub async fn close(self) -> CiabResult<()> {
        self.handle
            .disconnect(russh::Disconnect::ByApplication, "", "en")
            .await
            .map_err(|e| CiabError::SshError(format!("disconnect: {e}")))?;
        Ok(())
    }
}

/// Generated SSH keypair: the private key (for russh auth) and the OpenSSH-formatted public key string.
pub struct GeneratedKeypair {
    pub private_key: Arc<PrivateKey>,
    pub public_key_openssh: String,
}

/// Generate an Ed25519 SSH keypair suitable for injecting into cloud-init.
pub fn generate_keypair() -> CiabResult<GeneratedKeypair> {
    use ssh_key::rand_core::OsRng;

    let private_key = PrivateKey::random(&mut OsRng, Algorithm::Ed25519)
        .map_err(|e| CiabError::SshError(format!("keygen failed: {e}")))?;

    let public_key_openssh = private_key
        .public_key()
        .to_openssh()
        .map_err(|e| CiabError::SshError(format!("public key format: {e}")))?;

    Ok(GeneratedKeypair {
        private_key: Arc::new(private_key),
        public_key_openssh,
    })
}
