use std::{
    fs::{self, File},
    io::Write,
    os::unix::process::ExitStatusExt,
    process::ExitStatus,
};

use anyhow::{anyhow, Result};
use axum::Json;
use base64::{prelude::BASE64_STANDARD, Engine};
use serde::{Deserialize, Serialize};
use tempdir::TempDir;

use crate::{
    error::AppError, run_command::{run_command, CommandOptions, CommandOutput}, types::{Executable, Language}
};

#[derive(Deserialize)]
pub struct CompileRequest {
    pub source_code: String,
    pub compiler_options: String,
    pub language: Language,
}

#[derive(Serialize)]
pub struct CompileResponse {
    /// None if the compilation did not succeed.
    pub executable: Option<Executable>,

    /// Process output of the compilation command.
    pub compile_output: CommandOutput,
}

pub fn compile(compile_request: CompileRequest) -> Result<CompileResponse> {
    let tmp_dir = TempDir::new("compile")?;
    let tmp_out_dir = TempDir::new("compile-out")?;

    let mut source_file = File::create(tmp_dir.path().join("program.cpp"))?;
    source_file.write_all(compile_request.source_code.as_bytes())?;

    let output_file_path = tmp_out_dir
        .path()
        .join("program")
        .into_os_string()
        .into_string()
        .map_err(|_| anyhow!("failed to convert output_file_path into string"))?;

    let command = format!(
        "g++ -o {} {} program.cpp",
        output_file_path, compile_request.compiler_options
    );
    let compile_output = run_command(
        &command,
        tmp_dir.path(),
        CommandOptions {
            stdin: String::new(),
            timeout_ms: 5000,
        },
    )?;

    let executable = if ExitStatus::from_raw(compile_output.exit_code).success() {
        let encoded_binary = BASE64_STANDARD.encode(fs::read(output_file_path)?);
        Some(match compile_request.language {
            Language::Cpp => Executable::Binary {
                value: encoded_binary,
            },
            Language::Java21 => Executable::JavaClass {
                class_name: "Main".to_string(),
                value: encoded_binary,
            },
            Language::Py11 => unreachable!(),
        })
    } else {
        Option::None
    };

    let response = CompileResponse {
        executable,
        compile_output,
    };

    drop(source_file);
    tmp_dir.close()?;

    Ok(response)
}

pub async fn compile_handler(
    Json(payload): Json<CompileRequest>,
) -> Result<Json<CompileResponse>, AppError> {
    Ok(Json(compile(payload)?))
}
