use async_trait::async_trait;
use uuid::Uuid;

use crate::error::CiabResult;
use crate::types::image::{BuiltImage, ImageBuildRequest, ImageBuildResult, ImageBuildStatus};

/// Trait for building machine images (e.g., AMIs via Packer).
#[async_trait]
pub trait ImageBuilder: Send + Sync {
    async fn build_image(&self, request: &ImageBuildRequest) -> CiabResult<ImageBuildResult>;
    async fn list_images(&self) -> CiabResult<Vec<BuiltImage>>;
    async fn delete_image(&self, image_id: &str) -> CiabResult<()>;
    async fn build_status(&self, build_id: &Uuid) -> CiabResult<ImageBuildStatus>;
}
