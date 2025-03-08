use std::fs;

use axum::{
    routing::{get, post}, Router
};
use lambda_http::{run, tracing, Error};

mod compile;
mod execute;
mod compile_and_execute;
mod types;
mod error;
mod run_command;

use compile::compile_handler;
use execute::execute_handler;
use compile_and_execute::compile_and_execute_handler;

async fn index_page() -> &'static str {
    "Serverless Online Judge (Rust)"
}

#[derive(Clone)]
struct AppState {
    s3_client: aws_sdk_s3::Client,
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
        .with_state(state);

    run(app).await
}
