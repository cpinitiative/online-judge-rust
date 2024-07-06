use axum::Json;
use serde::{Deserialize, Serialize};

use crate::{
    compile::{compile, CompileRequest},
    error::AppError,
    execute::{execute, ExecuteRequest},
    run_command::{CommandOptions, CommandOutput},
};

/// Payload for POST /compile-and-execute
///
/// Called when the user wants to compile and execute the given code with a single lambda call.
/// Used by the USACO Guide IDE's "execute code" functionality.
#[derive(Deserialize)]
pub struct CompileAndExecuteRequest {
    pub compile: CompileRequest,
    pub execute: CommandOptions,
}

/// Response for POST /compile-and-execute
#[derive(Serialize)]
pub struct CompileAndExecuteResponse {
    pub compile: CommandOutput,
    /// None if the program failed to compile.
    pub execute: Option<CommandOutput>,
}

pub async fn compile_and_execute_handler(
    Json(payload): Json<CompileAndExecuteRequest>,
) -> Result<Json<CompileAndExecuteResponse>, AppError> {
    let compile_output = compile(payload.compile)?;
    let execute_output = if let Some(executable) = compile_output.executable {
        Some(execute(ExecuteRequest {
            executable,
            options: payload.execute,
        })?)
    } else {
        None
    };
    Ok(Json(CompileAndExecuteResponse {
        compile: compile_output.compile_output,
        execute: execute_output,
    }))
}
