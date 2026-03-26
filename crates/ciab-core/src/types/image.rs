use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Where to load a Packer template from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TemplateSource {
    /// Raw HCL content inline.
    Inline { content: String },
    /// Local filesystem path.
    FilePath { path: PathBuf },
    /// HTTP(S) URL.
    Url { url: String },
    /// Git repository with optional subpath and ref.
    Git {
        url: String,
        path: String,
        #[serde(rename = "ref")]
        ref_: Option<String>,
    },
    /// Built-in template shipped with CIAB.
    Builtin { name: String },
}

/// Request to build a machine image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBuildRequest {
    pub template: Option<TemplateSource>,
    #[serde(default)]
    pub variables: HashMap<String, String>,
    pub agent_provider: Option<String>,
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

/// Result of an image build.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBuildResult {
    pub build_id: Uuid,
    pub status: ImageBuildStatus,
    pub image_id: Option<String>,
    #[serde(default)]
    pub logs: Vec<String>,
}

/// Status of an image build.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImageBuildStatus {
    Queued,
    Running,
    Succeeded,
    Failed(String),
}

/// A previously built image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltImage {
    pub image_id: String,
    pub provider: String,
    pub region: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: HashMap<String, String>,
}
