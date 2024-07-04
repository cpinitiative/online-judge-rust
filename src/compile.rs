use std::{fs::{self, File}, io::{Cursor, Read, Write}, os::unix::process::ExitStatusExt, process::Command, str};

use anyhow::{anyhow, Context};
use axum::Json;
use base64::{prelude::BASE64_STANDARD, Engine};
use nix::sys::signal::Signal;
use tempdir::TempDir;
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{
    error::AppError,
    types::{CompileRequest, CompileResponse, ProcessOutput},
};

pub async fn compile(
    Json(payload): Json<CompileRequest>,
) -> Result<Json<CompileResponse>, AppError> {
    let tmp_dir = TempDir::new("compile")?;
    let tmp_out_dir = TempDir::new("compile-out")?;

    let mut source_file = File::create(tmp_dir.path().join(payload.filename.clone()))?;
    source_file.write_all(payload.source_code.as_bytes())?;

    let output_file_path = tmp_out_dir.path().join("program")
        .into_os_string()
        .into_string()
        .map_err(|_| anyhow!("failed to convert output_file_path into string"))?;

    let mut compile_args = shell_words::split(&payload.compiler_options)?;
    compile_args.extend(["-o".to_string(), output_file_path.clone(), payload.filename]);
    let process = Command::new("g++")
        .args(compile_args)
        .current_dir(tmp_dir.as_ref())
        .output()
        .with_context(|| "Failed to start compilation process")?;

    let zip_output = if process.status.success() {
        let mut zip_buf = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(&mut zip_buf);
        let options = SimpleFileOptions::default();

        zip.start_file("run.sh", options)?;
        zip.write_all(b"./program")?;

        zip.start_file("program", options)?;
        zip.write_all(fs::read(output_file_path)?.as_ref())?;

        zip.finish()?;

        Some(BASE64_STANDARD.encode(zip_buf.into_inner()))
    } else {
        Option::None
    };

    let response = CompileResponse {
        output: zip_output,
        process_output: Some(ProcessOutput {
            exit_code: process.status.into_raw(),
            exit_signal: process.status.signal().map(|signal| {
                Signal::try_from(signal).map_or(format!("Unknown signal {signal}"), |signal| {
                    signal.to_string()
                })
            }),
            stdout: str::from_utf8(&process.stdout)?.to_string(),
            stderr: str::from_utf8(&process.stderr)?.to_string(),
        }),
    };

    drop(source_file);
    tmp_dir.close()?;

    Ok(Json(response))
}
