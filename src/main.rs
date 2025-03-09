use std::fs;

use aws_sdk_s3::presigning::PresigningConfig;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use error::AppError;
use lambda_http::{run, tracing, Error};

mod compile;
mod compile_and_execute;
mod error;
mod execute;
mod run_command;
mod types;

use compile::compile_handler;
use compile_and_execute::compile_and_execute_handler;
use execute::execute_handler;
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    s3_client: aws_sdk_s3::Client,
}

async fn index_page() -> &'static str {
    "Serverless Online Judge (Rust)"
}

#[derive(Serialize)]
struct LargeInputResponse {
    presigned_url: String,
    input_id: String,
}

async fn large_input_handler(
    State(state): State<AppState>,
) -> Result<Json<LargeInputResponse>, AppError> {
    let id = Uuid::new_v4();

    let presigned_url = state
        .s3_client
        .put_object()
        .bucket("online-judge-rust-data")
        .key(format!("inputs/{id}.txt"))
        .presigned(PresigningConfig::expires_in(
            std::time::Duration::from_secs(60 * 5),
        )?)
        .await?;
    Ok(Json(LargeInputResponse {
        presigned_url: presigned_url.uri().to_string(),
        input_id: id.to_string(),
    }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&config);

    fs::create_dir_all("/tmp/precompiled-headers/bits/stdc++.h.gch")?;

    let state = AppState { s3_client };

    let app = Router::new()
        .route("/", get(index_page))
        .route("/compile", post(compile_handler))
        .route("/execute", post(execute_handler))
        .route("/compile-and-execute", post(compile_and_execute_handler))
        .route("/large-input", post(large_input_handler))
        .with_state(state);

    run(app).await
}
