use axum::{
    routing::{get, post},
    Router,
};
use lambda_http::{run, tracing, Error};

mod compile;
mod compile_and_execute;
mod execute;
mod types;
mod error;
mod process;

use compile::compile_handler;
use execute::execute_handler;
use compile_and_execute::compile_and_execute;

async fn index_page() -> &'static str {
    "Serverless Online Judge (Rust)"
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let app = Router::new()
        .route("/", get(index_page))
        .route("/compile", post(compile_handler))
        .route("/execute", post(execute_handler))
        .route("/compile-and-execute", post(compile_and_execute));

    run(app).await
}
