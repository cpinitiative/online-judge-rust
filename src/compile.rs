use std::{
    fs::{self, File},
    io::Write,
    os::unix::process::ExitStatusExt,
    path::Path,
    process::{Command, ExitStatus},
};

use anyhow::{anyhow, Context, Result};
use axum::Json;
use base64::{prelude::BASE64_STANDARD, Engine};
use serde::{Deserialize, Serialize};
use tempdir::TempDir;

use crate::{
    error::AppError,
    run_command::{run_command, CommandOptions, CommandOutput},
    types::{Executable, Language},
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

/// Precompile bits/stdc++.h.
///
/// Building bits/stdc++.h can be very slow. We can substantially speed this up by precompiling
/// headers: https://gcc.gnu.org/onlinedocs/gcc/Precompiled-Headers.html
///
/// However, precompiling headers is slow (~6s for C++23), and /tmp storage space is expensive, so
/// we only precompile bits/stdc++.h for some compiler options.
///
/// I believe flags like -Wall are ignored, but flags like -std, -O2, and -fsanitize=address must
/// match the flags used to precompile the header.
///
/// We don't do this precompilation in the dockerfile because lambda disk read speeds are abysmally
/// slow (~6 MB/s empirically), and the precompiled headers are quite large.
///
/// We precompile headers even if the request doesn't need it. Otherwise if nobody uses C++23 for
/// example, one poor user may end up with long compile times for every lambda instance. By
/// precompiling headers for the first two requests, we reduce the chance that one user repeatedly
/// gets a slow experience.
fn precompile_headers() -> Result<()> {
    const PRECOMPILE_VERSIONS: &'static [&'static str] = &["17", "23"];
    static mut VERSION_IDX: usize = 0;

    // Note: this must be single-threaded due to the use of static mut
    if unsafe { VERSION_IDX } >= PRECOMPILE_VERSIONS.len() {
        return Ok(());
    }
    let cpp_version = PRECOMPILE_VERSIONS[unsafe { VERSION_IDX }];
    unsafe { VERSION_IDX += 1 };

    let precompiled_header_path =
        format!("/tmp/precompiled-headers/bits/stdc++.h.gch/{cpp_version}");

    if Path::new(&precompiled_header_path).exists() {
        return Ok(());
    }

    if !Command::new("g++")
        .arg("-o")
        .arg(precompiled_header_path)
        .arg(format!("-std=c++{cpp_version}"))
        .arg("-O2")
        .arg("/usr/include/c++/11/x86_64-amazon-linux/bits/stdc++.h")
        .status()
        .with_context(|| format!("Failed to precompile header"))?
        .success()
    {
        return Err(anyhow!(
            "Command to precompile header exited with nonzero exit code"
        ));
    }

    Ok(())
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

    if let Err(err) = precompile_headers() {
        println!("Warning: Failed to precompile headers: {err}");
    }

    let command = format!(
        "g++ -I/tmp/precompiled-headers -o {} {} program.cpp",
        output_file_path, compile_request.compiler_options
    );
    let compile_output = run_command(
        &command,
        tmp_dir.path(),
        CommandOptions {
            stdin: String::new(),
            timeout_ms: 10000,
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
