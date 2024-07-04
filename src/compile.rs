use std::{
    fs::{self, File},
    io::Write,
    os::unix::process::ExitStatusExt,
    process::Command,
    str,
};

use anyhow::{anyhow, Context};
use axum::Json;
use base64::{prelude::BASE64_STANDARD, Engine};
use nix::sys::signal::Signal;
use tempdir::TempDir;

use crate::{
    error::AppError,
    types::{CompileRequest, CompileResponse, Executable, Language, ProcessOutput},
};

pub async fn compile(
    Json(payload): Json<CompileRequest>,
) -> Result<Json<CompileResponse>, AppError> {
    let tmp_dir = TempDir::new("compile")?;
    let tmp_out_dir = TempDir::new("compile-out")?;

    let mut source_file = File::create(tmp_dir.path().join("program.cpp"))?;
    source_file.write_all(payload.source_code.as_bytes())?;

    let output_file_path = tmp_out_dir
        .path()
        .join("program")
        .into_os_string()
        .into_string()
        .map_err(|_| anyhow!("failed to convert output_file_path into string"))?;

    let mut compile_args = shell_words::split(&payload.compiler_options)?;
    compile_args.extend(["-o".to_string(), output_file_path.clone(), "program.cpp".to_string()]);
    let process = Command::new("g++")
        .args(compile_args)
        .current_dir(tmp_dir.as_ref())
        .output()
        .with_context(|| "Failed to start compilation process")?;

    let executable = if process.status.success() {
        let encoded_binary = BASE64_STANDARD.encode(fs::read(output_file_path)?);
        Some(match payload.language {
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
        process_output: ProcessOutput {
            exit_code: process.status.into_raw(),
            exit_signal: process.status.signal().map(|signal| {
                Signal::try_from(signal).map_or(format!("Unknown signal {signal}"), |signal| {
                    signal.to_string()
                })
            }),
            stdout: str::from_utf8(&process.stdout)?.to_string(),
            stderr: str::from_utf8(&process.stderr)?.to_string(),
        },
    };

    drop(source_file);
    tmp_dir.close()?;

    Ok(Json(response))
}
