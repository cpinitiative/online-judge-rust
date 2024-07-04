use axum::{
    http::StatusCode,
    Json,
};
use crate::types::{CompileAndExecuteRequest, CompileAndExecuteResponse};

pub async fn compile_and_execute(
    Json(_payload): Json<CompileAndExecuteRequest>,
) -> (StatusCode, Json<CompileAndExecuteResponse>) {
    let response = CompileAndExecuteResponse {
        compile_result: "OK".to_string(),
    };

    (StatusCode::OK, Json(response))
}
