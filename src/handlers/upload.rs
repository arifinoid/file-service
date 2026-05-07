use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tracing::{error};
use uuid::Uuid;

use crate::models::UploadResponse;
use crate::services::image_service::ImageService;
use crate::AppState;

pub async fn upload_avatar(
    State(state): State<Arc<AppState>>,
    multipart: Multipart,
) -> impl IntoResponse {
    process_upload(state, multipart, 512).await
}

pub async fn upload_activity(
    State(state): State<Arc<AppState>>,
    multipart: Multipart,
) -> impl IntoResponse {
    process_upload(state, multipart, 1080).await
}

async fn process_upload(
    state: Arc<AppState>,
    mut multipart: Multipart,
    max_size: u32,
) -> impl IntoResponse {
    let field = match multipart.next_field().await {
        Ok(Some(field)) => field,
        _ => return (StatusCode::BAD_REQUEST, "No file provided").into_response(),
    };

    let data = match field.bytes().await {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to read bytes from multipart field: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read bytes").into_response();
        }
    };

    let compressed_data = match ImageService::compress(&data, max_size) {
        Ok(data) => data,
        Err(e) => {
            error!("Compression error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to compress image").into_response()
        }
    };

    let file_name = format!("{}.webp", Uuid::new_v4());
    
    match state.storage.upload(&file_name, compressed_data, "image/webp").await {
        Ok(url) => (StatusCode::OK, Json(UploadResponse { url })).into_response(),
        Err(e) => {
            error!("Storage error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to upload to storage").into_response()
        }
    }
}
