mod handlers;
mod infrastructure;
mod models;
mod services;

use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::Client;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;
use axum::{routing::post, Router, extract::DefaultBodyLimit};
use tower_http::trace::TraceLayer;

use crate::infrastructure::storage::S3Storage;
use crate::handlers::upload::{upload_avatar, upload_activity};

pub struct AppState {
    pub storage: S3Storage,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let account_id = env::var("R2_ACCOUNT_ID").expect("R2_ACCOUNT_ID must be set");
    let access_key_id = env::var("R2_ACCESS_KEY_ID").expect("R2_ACCESS_KEY_ID must be set");
    let secret_access_key = env::var("R2_SECRET_ACCESS_KEY").expect("R2_SECRET_ACCESS_KEY must be set");
    let bucket_name = env::var("R2_BUCKET_NAME").expect("R2_BUCKET_NAME must be set");
    let r2_public_url = env::var("R2_PUBLIC_URL").expect("R2_PUBLIC_URL must be set");

    let endpoint = format!("https://{}.r2.cloudflarestorage.com", account_id);
    let credentials = Credentials::new(access_key_id, secret_access_key, None, None, "static");
    
    let s3_config = aws_sdk_s3::config::Builder::new()
        .endpoint_url(endpoint)
        .region(aws_sdk_s3::config::Region::new("auto"))
        .credentials_provider(credentials)
        .behavior_version_latest()
        .build();

    let s3_client = Client::from_conf(s3_config);
    let storage = S3Storage::new(s3_client, bucket_name, r2_public_url);

    let state = Arc::new(AppState { storage });

    let app = Router::new()
        .route("/api/upload/avatar", post(upload_avatar))
        .route("/api/upload/activity", post(upload_activity))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB limit
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();
    info!("File service running on http://0.0.0.0:8081");
    axum::serve(listener, app).await.unwrap();
}
