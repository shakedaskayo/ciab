use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use ciab_core::types::image::ImageBuildRequest;
use tracing::info;

use crate::state::AppState;

pub async fn build_image(
    State(state): State<AppState>,
    Json(request): Json<ImageBuildRequest>,
) -> Result<impl IntoResponse, ciab_core::error::CiabError> {
    let builder = state.image_builder.as_ref().ok_or_else(|| {
        ciab_core::error::CiabError::ImageBuildError(
            "No image builder configured. Set [packer] in config.toml.".to_string(),
        )
    })?;
    info!("Starting image build");
    let result = builder.build_image(&request).await?;
    Ok((StatusCode::ACCEPTED, Json(result)))
}

pub async fn list_images(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ciab_core::error::CiabError> {
    let builder = state.image_builder.as_ref().ok_or_else(|| {
        ciab_core::error::CiabError::ImageBuildError("No image builder configured.".to_string())
    })?;
    let images = builder.list_images().await?;
    Ok(Json(images))
}

pub async fn get_build_status(
    State(state): State<AppState>,
    axum::extract::Path(build_id): axum::extract::Path<uuid::Uuid>,
) -> Result<impl IntoResponse, ciab_core::error::CiabError> {
    let builder = state.image_builder.as_ref().ok_or_else(|| {
        ciab_core::error::CiabError::ImageBuildError("No image builder configured.".to_string())
    })?;
    let status = builder.build_status(&build_id).await?;
    Ok(Json(status))
}

pub async fn delete_image(
    State(state): State<AppState>,
    axum::extract::Path(image_id): axum::extract::Path<String>,
) -> Result<impl IntoResponse, ciab_core::error::CiabError> {
    let builder = state.image_builder.as_ref().ok_or_else(|| {
        ciab_core::error::CiabError::ImageBuildError("No image builder configured.".to_string())
    })?;
    builder.delete_image(&image_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
