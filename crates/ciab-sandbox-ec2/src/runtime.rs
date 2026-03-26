use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use aws_sdk_ec2::config::Region;
use aws_sdk_ec2::types::{
    BlockDeviceMapping, EbsBlockDevice, IamInstanceProfileSpecification, InstanceType,
    ResourceType, Tag, TagSpecification, VolumeType,
};
use aws_sdk_ec2::Client as Ec2Client;
use chrono::Utc;
use dashmap::DashMap;
use ssh_key::PrivateKey;
use tokio::sync::mpsc;
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::runtime::SandboxRuntime;
use ciab_core::types::config::Ec2Config;
use ciab_core::types::sandbox::{
    ExecRequest, ExecResult, FileInfo, LogOptions, ResourceStats, SandboxInfo, SandboxSpec,
    SandboxState,
};

use crate::ssh::{self, GeneratedKeypair, SshSession};

/// Per-sandbox state tracked in memory.
struct InstanceState {
    instance_id: String,
    public_ip: Option<String>,
    keypair: GeneratedKeypair,
    info: SandboxInfo,
}

/// AWS EC2 runtime backend for CIAB.
pub struct Ec2Runtime {
    ec2_client: Ec2Client,
    config: Ec2Config,
    sandboxes: DashMap<Uuid, InstanceState>,
}

impl Ec2Runtime {
    /// Create a new EC2 runtime from the given config.
    pub async fn new(config: Ec2Config) -> CiabResult<Self> {
        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .load()
            .await;

        let ec2_client = Ec2Client::new(&aws_config);

        Ok(Self {
            ec2_client,
            config,
            sandboxes: DashMap::new(),
        })
    }

    /// Build cloud-init user-data script that injects the SSH public key.
    fn build_user_data(&self, public_key_openssh: &str) -> String {
        format!(
            r#"#!/bin/bash
set -e
mkdir -p /home/{user}/.ssh
echo '{pubkey}' >> /home/{user}/.ssh/authorized_keys
chmod 700 /home/{user}/.ssh
chmod 600 /home/{user}/.ssh/authorized_keys
chown -R {user}:{user} /home/{user}/.ssh
"#,
            user = self.config.ssh_user,
            pubkey = public_key_openssh,
        )
    }

    /// Build EC2 tags for a sandbox.
    fn build_tags(&self, sandbox_id: &Uuid, name: &str) -> Vec<Tag> {
        let mut tags = vec![
            Tag::builder()
                .key("ciab-sandbox-id")
                .value(sandbox_id.to_string())
                .build(),
            Tag::builder()
                .key("ciab-managed")
                .value("true")
                .build(),
            Tag::builder()
                .key("Name")
                .value(format!("ciab-{name}"))
                .build(),
        ];
        for (k, v) in &self.config.tags {
            tags.push(Tag::builder().key(k).value(v).build());
        }
        tags
    }

    /// Wait for an instance to reach the "running" state and obtain its public IP.
    async fn wait_for_running(&self, instance_id: &str) -> CiabResult<String> {
        let timeout = std::time::Duration::from_secs(self.config.instance_ready_timeout_secs);
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(CiabError::SandboxTimeout(format!(
                    "instance {instance_id} did not become running within {}s",
                    timeout.as_secs()
                )));
            }

            let resp = self
                .ec2_client
                .describe_instances()
                .instance_ids(instance_id)
                .send()
                .await
                .map_err(|e| CiabError::Ec2Error(format!("describe_instances: {e}")))?;

            if let Some(reservation) = resp.reservations().first() {
                if let Some(instance) = reservation.instances().first() {
                    let state_name = instance
                        .state()
                        .and_then(|s| s.name())
                        .map(|n| n.as_str().to_string())
                        .unwrap_or_default();

                    if state_name == "running" {
                        if let Some(ip) = instance.public_ip_address() {
                            return Ok(ip.to_string());
                        }
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }

    /// Wait until SSH is reachable on the instance.
    async fn wait_for_ssh(
        &self,
        host: &str,
        key: Arc<PrivateKey>,
    ) -> CiabResult<()> {
        let timeout = std::time::Duration::from_secs(self.config.instance_ready_timeout_secs);
        let start = Instant::now();
        let port = self.config.ssh_port;
        let user = self.config.ssh_user.clone();

        loop {
            if start.elapsed() > timeout {
                return Err(CiabError::SandboxTimeout(format!(
                    "SSH not reachable on {host}:{port} within {}s",
                    timeout.as_secs()
                )));
            }

            match SshSession::connect(host, port, &user, key.clone()).await {
                Ok(session) => {
                    let _ = session.close().await;
                    return Ok(());
                }
                Err(_) => {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// Connect SSH session for a sandbox by ID.
    async fn ssh_session(&self, id: &Uuid) -> CiabResult<SshSession> {
        let state = self
            .sandboxes
            .get(id)
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;

        let host = state
            .public_ip
            .as_deref()
            .ok_or_else(|| CiabError::Ec2Error("instance has no public IP".to_string()))?;

        SshSession::connect(
            host,
            self.config.ssh_port,
            &self.config.ssh_user,
            state.keypair.private_key.clone(),
        )
        .await
    }

    /// Build the shell command string from an ExecRequest.
    fn build_command(request: &ExecRequest) -> String {
        let mut parts = Vec::new();

        // Set environment variables
        for (k, v) in &request.env {
            parts.push(format!("export {}={}", k, shell_escape(v)));
        }

        // Change directory if specified
        if let Some(ref wd) = request.workdir {
            parts.push(format!("cd {}", shell_escape(wd)));
        }

        // The command itself
        let cmd = request.command.join(" ");
        parts.push(cmd);

        parts.join(" && ")
    }
}

/// Simple shell escaping for values.
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[async_trait]
impl SandboxRuntime for Ec2Runtime {
    async fn create_sandbox(&self, spec: &SandboxSpec) -> CiabResult<SandboxInfo> {
        let sandbox_id = Uuid::new_v4();
        let name = spec
            .name
            .clone()
            .unwrap_or_else(|| format!("ec2-{}", &sandbox_id.to_string()[..8]));

        // Generate ephemeral SSH keypair
        let keypair = ssh::generate_keypair()?;

        // Build cloud-init user data
        let user_data = self.build_user_data(&keypair.public_key_openssh);
        let user_data_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &user_data);

        // Build tags
        let tags = self.build_tags(&sandbox_id, &name);
        let tag_spec = TagSpecification::builder()
            .resource_type(ResourceType::Instance)
            .set_tags(Some(tags))
            .build();

        // AMI
        let ami = self
            .config
            .default_ami
            .clone()
            .ok_or_else(|| CiabError::Ec2Error("no AMI configured (ec2.default_ami)".into()))?;

        // Instance type
        let instance_type = InstanceType::from(self.config.instance_type.as_str());

        // Build RunInstances request
        let mut run_req = self
            .ec2_client
            .run_instances()
            .image_id(&ami)
            .instance_type(instance_type)
            .min_count(1)
            .max_count(1)
            .user_data(&user_data_b64)
            .tag_specifications(tag_spec)
            .block_device_mappings(
                BlockDeviceMapping::builder()
                    .device_name("/dev/sda1")
                    .ebs(
                        EbsBlockDevice::builder()
                            .volume_size(self.config.root_volume_size_gb as i32)
                            .volume_type(VolumeType::Gp3)
                            .delete_on_termination(true)
                            .build(),
                    )
                    .build(),
            );

        if let Some(ref subnet) = self.config.subnet_id {
            run_req = run_req.subnet_id(subnet);
        }

        for sg in &self.config.security_group_ids {
            run_req = run_req.security_group_ids(sg);
        }

        if let Some(ref profile) = self.config.iam_instance_profile {
            run_req = run_req.iam_instance_profile(
                IamInstanceProfileSpecification::builder()
                    .name(profile)
                    .build(),
            );
        }

        let run_resp = run_req
            .send()
            .await
            .map_err(|e| CiabError::Ec2Error(format!("RunInstances failed: {e}")))?;

        let instance = run_resp
            .instances()
            .first()
            .ok_or_else(|| CiabError::Ec2Error("no instance returned".into()))?;

        let instance_id = instance
            .instance_id()
            .ok_or_else(|| CiabError::Ec2Error("no instance ID".into()))?
            .to_string();

        tracing::info!(sandbox_id = %sandbox_id, instance_id = %instance_id, "EC2 instance launched");

        // Wait for running + public IP
        let public_ip = self.wait_for_running(&instance_id).await?;
        tracing::info!(sandbox_id = %sandbox_id, ip = %public_ip, "instance running");

        // Wait for SSH
        self.wait_for_ssh(&public_ip, keypair.private_key.clone())
            .await?;
        tracing::info!(sandbox_id = %sandbox_id, "SSH reachable");

        let now = Utc::now();
        let info = SandboxInfo {
            id: sandbox_id,
            name: Some(name),
            state: SandboxState::Running,
            persistence: spec.persistence.clone(),
            agent_provider: spec.agent_provider.clone(),
            endpoint_url: Some(format!("ssh://{}@{}:{}", self.config.ssh_user, public_ip, self.config.ssh_port)),
            resource_stats: None,
            labels: {
                let mut labels = spec.labels.clone();
                labels.insert("ec2-instance-id".to_string(), instance_id.clone());
                labels
            },
            created_at: now,
            updated_at: now,
            spec: spec.clone(),
        };

        self.sandboxes.insert(
            sandbox_id,
            InstanceState {
                instance_id,
                public_ip: Some(public_ip),
                keypair,
                info: info.clone(),
            },
        );

        Ok(info)
    }

    async fn get_sandbox(&self, id: &Uuid) -> CiabResult<SandboxInfo> {
        let state = self
            .sandboxes
            .get(id)
            .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;
        Ok(state.info.clone())
    }

    async fn list_sandboxes(
        &self,
        state_filter: Option<SandboxState>,
        provider: Option<&str>,
        labels: &HashMap<String, String>,
    ) -> CiabResult<Vec<SandboxInfo>> {
        let mut result = Vec::new();
        for entry in self.sandboxes.iter() {
            let info = &entry.info;
            if let Some(ref sf) = state_filter {
                if &info.state != sf {
                    continue;
                }
            }
            if let Some(prov) = provider {
                if info.agent_provider != prov {
                    continue;
                }
            }
            let mut match_labels = true;
            for (k, v) in labels {
                if info.labels.get(k) != Some(v) {
                    match_labels = false;
                    break;
                }
            }
            if match_labels {
                result.push(info.clone());
            }
        }
        Ok(result)
    }

    async fn start_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let instance_id = {
            let state = self
                .sandboxes
                .get(id)
                .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;
            state.instance_id.clone()
        };

        self.ec2_client
            .start_instances()
            .instance_ids(&instance_id)
            .send()
            .await
            .map_err(|e| CiabError::Ec2Error(format!("StartInstances: {e}")))?;

        // Wait for running + update IP
        let public_ip = self.wait_for_running(&instance_id).await?;

        if let Some(mut state) = self.sandboxes.get_mut(id) {
            state.public_ip = Some(public_ip.clone());
            state.info.state = SandboxState::Running;
            state.info.endpoint_url = Some(format!(
                "ssh://{}@{}:{}",
                self.config.ssh_user, public_ip, self.config.ssh_port
            ));
            state.info.updated_at = Utc::now();
        }

        Ok(())
    }

    async fn stop_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let instance_id = {
            let state = self
                .sandboxes
                .get(id)
                .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;
            state.instance_id.clone()
        };

        self.ec2_client
            .stop_instances()
            .instance_ids(&instance_id)
            .send()
            .await
            .map_err(|e| CiabError::Ec2Error(format!("StopInstances: {e}")))?;

        if let Some(mut state) = self.sandboxes.get_mut(id) {
            state.info.state = SandboxState::Stopped;
            state.info.endpoint_url = None;
            state.info.updated_at = Utc::now();
        }

        Ok(())
    }

    async fn pause_sandbox(&self, _id: &Uuid) -> CiabResult<()> {
        Err(CiabError::Unsupported(
            "EC2 instances cannot be paused; use stop instead".to_string(),
        ))
    }

    async fn resume_sandbox(&self, _id: &Uuid) -> CiabResult<()> {
        Err(CiabError::Unsupported(
            "EC2 instances cannot be resumed; use start instead".to_string(),
        ))
    }

    async fn terminate_sandbox(&self, id: &Uuid) -> CiabResult<()> {
        let instance_id = {
            let state = self
                .sandboxes
                .get(id)
                .ok_or_else(|| CiabError::SandboxNotFound(id.to_string()))?;
            state.instance_id.clone()
        };

        self.ec2_client
            .terminate_instances()
            .instance_ids(&instance_id)
            .send()
            .await
            .map_err(|e| CiabError::Ec2Error(format!("TerminateInstances: {e}")))?;

        self.sandboxes.remove(id);

        tracing::info!(sandbox_id = %id, "EC2 instance terminated");
        Ok(())
    }

    async fn exec(&self, id: &Uuid, request: &ExecRequest) -> CiabResult<ExecResult> {
        let start = Instant::now();
        let session = self.ssh_session(id).await?;
        let command = Self::build_command(request);

        let (stdout, stderr, exit_code) = session.exec(&command).await?;
        let _ = session.close().await;

        Ok(ExecResult {
            exit_code,
            stdout,
            stderr,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn exec_streaming(
        &self,
        id: &Uuid,
        request: &ExecRequest,
    ) -> CiabResult<(
        mpsc::Receiver<String>,
        tokio::task::JoinHandle<CiabResult<ExecResult>>,
    )> {
        let start = Instant::now();
        let session = self.ssh_session(id).await?;
        let command = Self::build_command(request);

        // For streaming, we run exec and split stdout lines
        let (tx, rx) = mpsc::channel::<String>(256);

        let handle = tokio::spawn(async move {
            let (stdout, stderr, exit_code) = session.exec(&command).await?;
            let _ = session.close().await;

            for line in stdout.lines() {
                if tx.send(line.to_string()).await.is_err() {
                    break;
                }
            }

            Ok(ExecResult {
                exit_code,
                stdout,
                stderr,
                duration_ms: start.elapsed().as_millis() as u64,
            })
        });

        Ok((rx, handle))
    }

    async fn read_file(&self, id: &Uuid, path: &str) -> CiabResult<Vec<u8>> {
        let session = self.ssh_session(id).await?;
        let data = session.read_file(path).await?;
        let _ = session.close().await;
        Ok(data)
    }

    async fn write_file(&self, id: &Uuid, path: &str, content: &[u8]) -> CiabResult<()> {
        let session = self.ssh_session(id).await?;
        session.write_file(path, content).await?;
        let _ = session.close().await;
        Ok(())
    }

    async fn list_files(&self, id: &Uuid, path: &str) -> CiabResult<Vec<FileInfo>> {
        let session = self.ssh_session(id).await?;
        let output = session.list_files(path).await?;
        let _ = session.close().await;

        parse_ls_output(&output)
    }

    async fn get_stats(&self, id: &Uuid) -> CiabResult<ResourceStats> {
        let session = self.ssh_session(id).await?;

        // Get CPU usage from /proc/stat (simplified: use top -bn1)
        let (cpu_out, _, _) = session
            .exec("top -bn1 | head -3 | tail -1 | awk '{print $2+$4}'")
            .await?;
        let cpu_usage: f32 = cpu_out.trim().parse().unwrap_or(0.0);

        // Get memory info
        let (mem_out, _, _) = session
            .exec("free -m | awk '/Mem:/{printf \"%s %s\", $3, $2}'")
            .await?;
        let mem_parts: Vec<&str> = mem_out.split_whitespace().collect();
        let mem_used: u32 = mem_parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let mem_total: u32 = mem_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

        // Get disk info
        let (disk_out, _, _) = session
            .exec("df -m / | awk 'NR==2{printf \"%s %s\", $3, $2}'")
            .await?;
        let disk_parts: Vec<&str> = disk_out.split_whitespace().collect();
        let disk_used: u32 = disk_parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let disk_total: u32 = disk_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

        let _ = session.close().await;

        Ok(ResourceStats {
            cpu_usage_percent: cpu_usage,
            memory_used_mb: mem_used,
            memory_limit_mb: mem_total,
            disk_used_mb: disk_used,
            disk_limit_mb: disk_total,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
        })
    }

    async fn stream_logs(
        &self,
        id: &Uuid,
        options: &LogOptions,
    ) -> CiabResult<mpsc::Receiver<String>> {
        let session = self.ssh_session(id).await?;

        let mut cmd = String::from("tail");
        if options.follow {
            cmd.push_str(" -f");
        }
        if let Some(n) = options.tail {
            cmd.push_str(&format!(" -n {n}"));
        }
        cmd.push_str(" /var/log/syslog /var/log/cloud-init-output.log 2>/dev/null");

        let rx = session.exec_streaming(&cmd).await?;
        // Session is moved into the spawned task inside exec_streaming,
        // so we don't close it here.
        Ok(rx)
    }

    async fn kill_exec(&self, id: &Uuid) -> CiabResult<()> {
        let session = self.ssh_session(id).await?;
        let (_, stderr, code) = session
            .exec("pkill -f 'ciab-exec' 2>/dev/null || true")
            .await?;
        let _ = session.close().await;

        if code != 0 {
            tracing::warn!(sandbox_id = %id, stderr = %stderr, "kill_exec returned non-zero");
        }

        Ok(())
    }
}

/// Parse `ls -la` output into FileInfo entries.
fn parse_ls_output(output: &str) -> CiabResult<Vec<FileInfo>> {
    let mut files = Vec::new();
    for line in output.lines().skip(1) {
        // skip "total N" line
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        let mode_str = parts[0];
        let is_dir = mode_str.starts_with('d');
        let size: u64 = parts[4].parse().unwrap_or(0);
        let name = parts[8..].join(" ");

        // Skip . and ..
        if name == "." || name == ".." {
            continue;
        }

        files.push(FileInfo {
            path: name,
            size,
            is_dir,
            mode: parse_mode_string(mode_str),
            modified_at: None,
        });
    }
    Ok(files)
}

/// Convert ls mode string (e.g. "drwxr-xr-x") to octal mode u32.
fn parse_mode_string(s: &str) -> u32 {
    if s.len() < 10 {
        return 0;
    }
    let chars: Vec<char> = s.chars().collect();
    let mut mode: u32 = 0;

    // Owner
    if chars[1] == 'r' { mode |= 0o400; }
    if chars[2] == 'w' { mode |= 0o200; }
    if chars[3] == 'x' || chars[3] == 's' { mode |= 0o100; }

    // Group
    if chars[4] == 'r' { mode |= 0o040; }
    if chars[5] == 'w' { mode |= 0o020; }
    if chars[6] == 'x' || chars[6] == 's' { mode |= 0o010; }

    // Other
    if chars[7] == 'r' { mode |= 0o004; }
    if chars[8] == 'w' { mode |= 0o002; }
    if chars[9] == 'x' || chars[9] == 't' { mode |= 0o001; }

    mode
}
