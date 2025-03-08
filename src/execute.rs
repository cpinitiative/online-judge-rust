use std::{
    cmp::{max, min},
    fs::{self, File},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
};

use anyhow::{anyhow, Result};
use axum::Json;
use serde::{Deserialize, Serialize};
use tempfile::{tempdir, NamedTempFile};

use crate::{
    error::AppError,
    run_command::{run_command, CommandOptions},
    types::Executable,
};
use base64::{prelude::BASE64_STANDARD, Engine};

#[derive(Deserialize)]
pub struct ExecuteRequest {
    pub executable: Executable,
    pub options: ExecuteOptions,
}

#[derive(Deserialize)]
pub struct ExecuteOptions {
    pub stdin: String,
    pub timeout_ms: u32,

    /// Alphanumeric string if you want file I/O to be supported, such as "cowdating".
    ///
    /// Will create the files `file_io_name`.in and read `file_io_name`.out.
    pub file_io_name: Option<String>,
}

#[derive(Serialize)]
pub enum Verdict {
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "wrong_answer")]
    #[allow(dead_code)]
    WrongAnswer,
    #[serde(rename = "time_limit_exceeded")]
    TimeLimitExceeded,
    #[serde(rename = "runtime_error")]
    RuntimeError,
}

#[derive(Serialize)]
pub struct ExecuteResponse {
    pub stdout: String,

    /// Only if `file_io_name`.out exists.
    pub file_output: Option<String>,

    pub stderr: String,
    pub wall_time: String, // time format is 0:00.00
    pub memory_usage: String,

    /// The underlying raw wait status. Note that this is different from an exit status.
    pub exit_code: i32,
    pub exit_signal: Option<String>,

    pub verdict: Verdict,
}

fn extract_zip(dir: &Path, base64_zip: &str) -> Result<()> {
    let mut tmp_file = NamedTempFile::new()?;
    tmp_file.write_all(&BASE64_STANDARD.decode(base64_zip)?)?;
    if !Command::new("tar")
        .arg("xf")
        .arg(tmp_file.path().to_str().unwrap())
        .arg("-C")
        .arg(dir.to_str().unwrap())
        .status()?
        .success()
    {
        Err(anyhow!("Failed to extract base64 .tar.gz file"))
    } else {
        Ok(())
    }
}

fn truncate_if_needed(mut str: String, max_len: usize) -> String {
    if str.len() > max_len {
        // Note: This could panic if truncating multi-byte characters!
        str.truncate(max_len);
        str += "\n[Truncated]";
    }
    str
}

fn truncate_response(mut response: ExecuteResponse) -> ExecuteResponse {
    let mut remaining_len: usize = 5_800_000;

    response.file_output = response.file_output.map(|str| {
        truncate_if_needed(
            str,
            max(
                remaining_len / 3,
                remaining_len - min(response.stdout.len() + response.stderr.len(), remaining_len),
            ),
        )
    });
    remaining_len -= response.file_output.as_ref().map(|x| x.len()).unwrap_or(0);

    response.stderr = truncate_if_needed(
        response.stderr,
        max(
            remaining_len / 2,
            remaining_len - min(response.stdout.len(), remaining_len),
        ),
    );
    remaining_len -= response.stderr.len();

    response.stdout = truncate_if_needed(response.stdout, remaining_len);

    response
}

pub fn execute(payload: ExecuteRequest) -> Result<ExecuteResponse> {
    let tmp_dir = tempdir()?;

    extract_zip(tmp_dir.path(), &payload.executable.files)?;

    if let Some(ref name) = payload.options.file_io_name {
        if !name.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(anyhow!(
                "Invalid file I/O name. It must be alphanumeric, like \"cowdating\"."
            ));
        }
        let mut stdin_file = File::create(tmp_dir.path().join(name).with_extension("in"))?;
        stdin_file.write_all(payload.options.stdin.as_ref())?;
    }

    let command_options = CommandOptions {
        stdin: payload.options.stdin,
        timeout_ms: payload.options.timeout_ms,
    };

    // Run the command in a file to get messages like
    // ./run: line 1:   308 Segmentation fault      ./prog
    // I don't know why we don't get these messages normally.
    let mut run_file = File::create(tmp_dir.path().join("run"))?;
    run_file.write_all(payload.executable.run_command.as_bytes())?;
    let mut run_file_permissions = run_file.metadata()?.permissions();
    run_file_permissions.set_mode(0o755);
    run_file.set_permissions(run_file_permissions)?;
    drop(run_file);

    let command_output = run_command("./run", tmp_dir.path(), command_options)?;

    let verdict = match command_output.exit_code {
        124 => Verdict::TimeLimitExceeded,
        0 => Verdict::Accepted,
        _ => Verdict::RuntimeError,
    };

    let file_output = if let Some(name) = payload.options.file_io_name {
        let output_file_path = tmp_dir.path().join(name).with_extension("out");
        if Path::exists(&output_file_path) {
            Some(String::from_utf8_lossy(&fs::read(output_file_path)?).into_owned())
        } else {
            None
        }
    } else {
        None
    };

    Ok(truncate_response(ExecuteResponse {
        stdout: command_output.stdout,
        file_output,
        stderr: command_output.stderr,
        wall_time: command_output.wall_time,
        memory_usage: command_output.memory_usage,
        exit_code: command_output.exit_code,
        exit_signal: command_output.exit_signal,
        verdict,
    }))
}

pub async fn execute_handler(
    Json(payload): Json<ExecuteRequest>,
) -> Result<Json<ExecuteResponse>, AppError> {
    Ok(Json(execute(payload)?))
}
