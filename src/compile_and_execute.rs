use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    compile::{compile, CompileRequest},
    error::AppError,
    execute::{execute, ExecuteOptions, ExecuteRequest, ExecuteResponse},
    run_command::CommandOutput, AppState,
};

/// Payload for POST /compile-and-execute
///
/// Called when the user wants to compile and execute the given code with a single lambda call.
/// Used by the USACO Guide IDE's "execute code" functionality.
#[derive(Deserialize)]
pub struct CompileAndExecuteRequest {
    pub compile: CompileRequest,
    pub execute: ExecuteOptions,
}

/// Response for POST /compile-and-execute
#[derive(Serialize)]
pub struct CompileAndExecuteResponse {
    pub compile: CommandOutput,
    /// None if the program failed to compile.
    pub execute: Option<ExecuteResponse>,
}

pub async fn compile_and_execute_handler(
    State(state): State<AppState>,
    Json(payload): Json<CompileAndExecuteRequest>,
) -> Result<Json<CompileAndExecuteResponse>, AppError> {
    let compile_output = compile(payload.compile)?;
    let execute_output = if let Some(executable) = compile_output.executable {
        Some(execute(ExecuteRequest {
            executable,
            options: payload.execute,
        }, state.s3_client).await?)
    } else {
        None
    };
    Ok(Json(CompileAndExecuteResponse {
        compile: compile_output.compile_output,
        execute: execute_output,
    }))
}
