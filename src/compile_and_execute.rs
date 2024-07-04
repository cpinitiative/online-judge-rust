use crate::{
    compile::compile,
    error::AppError,
    execute::execute,
    types::{CompileAndExecuteRequest, CompileAndExecuteResponse, ExecuteRequest},
};
use axum::Json;

pub async fn compile_and_execute(
    Json(payload): Json<CompileAndExecuteRequest>,
) -> Result<Json<CompileAndExecuteResponse>, AppError> {
    let compile_output = compile(payload.compile)?;
    let execute_output = if let Some(executable) = compile_output.executable {
        Some(execute(ExecuteRequest {
            executable: executable,
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
