pub mod engine;

pub use engine::{CiabEngine, CiabEngineBuilder};

// Re-export core types
pub use ciab_core::error::{CiabError, CiabResult};
pub use ciab_core::traits::agent::AgentProvider;
pub use ciab_core::traits::image_builder::ImageBuilder;
pub use ciab_core::traits::runtime::SandboxRuntime;
pub use ciab_core::types::config::AppConfig;
pub use ciab_core::types::image::*;
pub use ciab_core::types::sandbox::*;
pub use ciab_core::types::stream::StreamEvent;

// Re-export runtime constructors (feature-gated)
#[cfg(feature = "local")]
pub use ciab_sandbox::LocalProcessRuntime;

#[cfg(feature = "ec2")]
pub use ciab_sandbox_ec2::Ec2Runtime;

#[cfg(feature = "kubernetes")]
pub use ciab_sandbox_k8s::KubernetesRuntime;

#[cfg(feature = "packer")]
pub use ciab_packer::PackerImageBuilder;

// Re-export agent providers
pub use ciab_agent_claude::ClaudeCodeProvider;
pub use ciab_agent_codex::CodexProvider;
pub use ciab_agent_cursor::CursorProvider;
pub use ciab_agent_gemini::GeminiProvider;
