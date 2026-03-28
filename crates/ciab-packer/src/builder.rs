use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tracing::info;
use uuid::Uuid;

use ciab_core::error::{CiabError, CiabResult};
use ciab_core::traits::image_builder::ImageBuilder;
use ciab_core::types::config::PackerConfig;
use ciab_core::types::image::{BuiltImage, ImageBuildRequest, ImageBuildResult, ImageBuildStatus};

use crate::template;

struct BuildState {
    status: ImageBuildStatus,
    image_id: Option<String>,
    logs: Vec<String>,
}

pub struct PackerImageBuilder {
    config: PackerConfig,
    builds: Arc<DashMap<Uuid, BuildState>>,
    images: Arc<DashMap<String, BuiltImage>>,
}

impl PackerImageBuilder {
    pub fn new(config: PackerConfig) -> Self {
        Self {
            config,
            builds: Arc::new(DashMap::new()),
            images: Arc::new(DashMap::new()),
        }
    }

    async fn packer_binary(&self) -> CiabResult<String> {
        let check = Command::new("which")
            .arg(&self.config.binary)
            .output()
            .await;

        if let Ok(output) = check {
            if output.status.success() {
                return Ok(self.config.binary.clone());
            }
        }

        if self.config.auto_install {
            info!("Packer not found, attempting auto-install");
            self.install_packer().await?;
            Ok(self.config.binary.clone())
        } else {
            Err(CiabError::PackerError(format!(
                "Packer binary '{}' not found. Set auto_install = true to install automatically.",
                self.config.binary
            )))
        }
    }

    async fn install_packer(&self) -> CiabResult<()> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(
                "curl -fsSL https://releases.hashicorp.com/packer/1.11.2/packer_1.11.2_linux_amd64.zip -o /tmp/packer.zip \
                 && unzip -o /tmp/packer.zip -d /usr/local/bin/ \
                 && rm /tmp/packer.zip",
            )
            .output()
            .await
            .map_err(|e| CiabError::PackerError(format!("Failed to install packer: {}", e)))?;

        if !output.status.success() {
            return Err(CiabError::PackerError(format!(
                "Packer install failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        info!("Packer installed successfully");
        Ok(())
    }

    fn merge_variables(&self, request: &ImageBuildRequest) -> HashMap<String, String> {
        let mut vars = self.config.variables.clone();
        vars.extend(request.variables.clone());
        vars
    }

    fn build_command_args(
        &self,
        binary: &str,
        template_path: &std::path::Path,
        variables: &HashMap<String, String>,
    ) -> Command {
        let mut cmd = Command::new(binary);
        cmd.arg("build");
        cmd.arg("-machine-readable");

        for (key, value) in variables {
            cmd.arg("-var");
            cmd.arg(format!("{}={}", key, value));
        }

        cmd.arg(template_path);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd
    }

    fn parse_artifact_id(line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 5 && parts[2] == "artifact" && parts[4] == "id" {
            let id_part = parts.get(5).unwrap_or(&"");
            if let Some((_region, ami)) = id_part.split_once(':') {
                return Some(ami.to_string());
            }
            return Some(id_part.to_string());
        }
        None
    }
}

#[async_trait]
impl ImageBuilder for PackerImageBuilder {
    async fn build_image(&self, request: &ImageBuildRequest) -> CiabResult<ImageBuildResult> {
        let build_id = Uuid::new_v4();
        info!(build_id = %build_id, "Starting Packer image build");

        self.builds.insert(
            build_id,
            BuildState {
                status: ImageBuildStatus::Running,
                image_id: None,
                logs: Vec::new(),
            },
        );

        let template_content = template::resolve_template(&request.template, &self.config).await?;
        let template_path = template::write_temp_template(&template_content).await?;

        let binary = self.packer_binary().await?;
        let variables = self.merge_variables(request);

        let mut cmd = self.build_command_args(&binary, &template_path, &variables);
        let mut child = cmd
            .spawn()
            .map_err(|e| CiabError::PackerError(format!("Failed to spawn packer: {}", e)))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| CiabError::PackerError("Failed to capture packer stdout".to_string()))?;

        let builds = self.builds.clone();
        let images = self.images.clone();
        let build_id_clone = build_id;
        let tags = request.tags.clone();

        tokio::spawn(async move {
            let reader = tokio::io::BufReader::new(stdout);
            let mut lines = reader.lines();
            let mut artifact_id: Option<String> = None;

            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(id) = PackerImageBuilder::parse_artifact_id(&line) {
                    artifact_id = Some(id);
                }
                if let Some(mut build) = builds.get_mut(&build_id_clone) {
                    build.logs.push(line);
                }
            }

            let status = child.wait().await;
            let success = status.map(|s| s.success()).unwrap_or(false);

            if let Some(mut build) = builds.get_mut(&build_id_clone) {
                if success {
                    build.status = ImageBuildStatus::Succeeded;
                    build.image_id = artifact_id.clone();
                    if let Some(ref image_id) = artifact_id {
                        images.insert(
                            image_id.clone(),
                            BuiltImage {
                                image_id: image_id.clone(),
                                provider: "amazon-ebs".to_string(),
                                region: None,
                                created_at: Utc::now(),
                                tags: tags.clone(),
                            },
                        );
                    }
                } else {
                    let err_msg = build
                        .logs
                        .last()
                        .cloned()
                        .unwrap_or_else(|| "Unknown error".to_string());
                    build.status = ImageBuildStatus::Failed(err_msg);
                }
            }
        });

        Ok(ImageBuildResult {
            build_id,
            status: ImageBuildStatus::Running,
            image_id: None,
            logs: Vec::new(),
        })
    }

    async fn list_images(&self) -> CiabResult<Vec<BuiltImage>> {
        Ok(self.images.iter().map(|r| r.value().clone()).collect())
    }

    async fn delete_image(&self, image_id: &str) -> CiabResult<()> {
        self.images.remove(image_id);
        info!(image_id = image_id, "Removed image from local registry");
        Ok(())
    }

    async fn build_status(&self, build_id: &Uuid) -> CiabResult<ImageBuildStatus> {
        self.builds
            .get(build_id)
            .map(|b| b.status.clone())
            .ok_or_else(|| CiabError::ImageBuildError(format!("Build {} not found", build_id)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_artifact_id_valid() {
        let line = "1234567890,amazon-ebs.agent,artifact,0,id,us-east-1:ami-0123456789abcdef0";
        let result = PackerImageBuilder::parse_artifact_id(line);
        assert_eq!(result, Some("ami-0123456789abcdef0".to_string()));
    }

    #[test]
    fn test_parse_artifact_id_no_match() {
        let line = "1234567890,amazon-ebs.agent,ui,message,Building AMI...";
        let result = PackerImageBuilder::parse_artifact_id(line);
        assert_eq!(result, None);
    }

    #[test]
    fn test_merge_variables() {
        let config = PackerConfig {
            binary: "packer".to_string(),
            auto_install: false,
            template_cache_dir: "/tmp".to_string(),
            template_cache_ttl_secs: 3600,
            default_template: "builtin://default-ec2".to_string(),
            variables: HashMap::from([
                ("region".to_string(), "us-east-1".to_string()),
                ("instance_type".to_string(), "t3.small".to_string()),
            ]),
        };
        let builder = PackerImageBuilder::new(config);
        let request = ImageBuildRequest {
            template: None,
            variables: HashMap::from([
                ("instance_type".to_string(), "t3.large".to_string()),
                ("base_ami".to_string(), "ami-123".to_string()),
            ]),
            agent_provider: None,
            tags: HashMap::new(),
        };
        let merged = builder.merge_variables(&request);
        assert_eq!(merged.get("region"), Some(&"us-east-1".to_string()));
        assert_eq!(merged.get("instance_type"), Some(&"t3.large".to_string()));
        assert_eq!(merged.get("base_ami"), Some(&"ami-123".to_string()));
    }
}
