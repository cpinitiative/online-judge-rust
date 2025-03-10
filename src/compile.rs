use std::{
    fs::{self, File},
    io::Write,
    os::unix::process::ExitStatusExt,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

use anyhow::{anyhow, Context, Result};
use axum::Json;
use base64::{prelude::BASE64_STANDARD, Engine};
use bytes::Bytes;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

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
/// we only precompile bits/stdc++.h for the default C++ compiler option.
///
/// I believe flags like -Wall are ignored, but flags like -std, -O2, and -fsanitize=address must
/// match the flags used to precompile the header.
///
/// We don't do this precompilation in the dockerfile because lambda disk read speeds are abysmally
/// slow (~6 MB/s empirically), and the precompiled headers are quite large.
fn precompile_headers(compile_request: &CompileRequest) -> Result<()> {
    let cpp_version = "23";

    if compile_request.language != Language::Cpp
        || !compile_request.compiler_options.contains("-O2")
        || !compile_request
            .compiler_options
            .contains(&format!("-std=c++{cpp_version}"))
        || !compile_request
            .source_code
            .contains("#include <bits/stdc++.h>")
    {
        return Ok(());
    }

    let precompiled_header_path =
        format!("/tmp/precompiled-headers/bits/stdc++.h.gch/{cpp_version}");

    if Path::new(&precompiled_header_path).exists() {
        return Ok(());
    }

    // todo: disable in local development
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
    let tmp_dir = tempdir()?;
    let tmp_out_dir = tempdir()?;

    let program_filename: PathBuf = match compile_request.language {
        Language::Cpp => "program.cpp".into(),
        Language::Java21 => {
            let re = Regex::new(r"public\s+class\s+(\w+)").unwrap();
            if let Some(captures) = re.captures(&compile_request.source_code) {
                format!("{}.java", &captures[1]).into()
            } else {
                "Main.java".into() // fallback, something went wrong
            }
        }
        Language::Py12 => "program.py".into(),
    };

    let mut source_file = File::create(tmp_dir.path().join(&program_filename))?;
    source_file.write_all(compile_request.source_code.as_bytes())?;
    drop(source_file);

    if let Err(err) = precompile_headers(&compile_request) {
        println!("Warning: Failed to precompile headers: {err}");
    }

    let command = match compile_request.language {
        Language::Cpp => format!(
            "g++ -I/tmp/precompiled-headers -o {} {} {}",
            tmp_out_dir
                .path()
                .join(program_filename.clone().with_extension(""))
                .as_os_str()
                .to_str()
                .unwrap(),
            compile_request.compiler_options,
            program_filename.as_os_str().to_str().unwrap(),
        ),
        Language::Java21 => format!(
            "javac -d {} {} {}",
            tmp_out_dir.path().as_os_str().to_str().unwrap(),
            compile_request.compiler_options,
            program_filename.as_os_str().to_str().unwrap(),
        ),
        Language::Py12 => format!(
            "cp {} {}",
            tmp_dir
                .path()
                .join(&program_filename)
                .as_os_str()
                .to_str()
                .unwrap(),
            tmp_out_dir
                .path()
                .join(&program_filename)
                .as_os_str()
                .to_str()
                .unwrap(),
        ),
    };
    let compile_output = run_command(
        &command,
        tmp_dir.path(),
        CommandOptions {
            stdin: Bytes::new(),
            timeout_ms: 20000,
        },
    )?;

    let run_command = match compile_request.language {
        Language::Cpp => "./program".to_owned(),
        Language::Java21 => format!(
            "java {}",
            program_filename.file_stem().unwrap().to_str().unwrap()
        ),
        Language::Py12 => "python3.12 program.py".to_owned(),
    };

    let base64_files = if ExitStatus::from_raw(compile_output.exit_code).success() {
        if !Command::new("sh")
            .arg("-c")
            .arg("tar czf executable.tar.gz *")
            .current_dir(tmp_out_dir.path())
            .status()?
            .success()
        {
            return Err(anyhow!("Failed to tar executable file"));
        }
        Some(BASE64_STANDARD.encode(fs::read(tmp_out_dir.path().join("executable.tar.gz"))?))
    } else {
        None
    };

    let response = CompileResponse {
        executable: base64_files.map(|files| Executable { files, run_command }),
        compile_output,
    };

    tmp_dir.close()?;

    Ok(response)
}

pub async fn compile_handler(
    Json(payload): Json<CompileRequest>,
) -> Result<Json<CompileResponse>, AppError> {
    Ok(Json(compile(payload)?))
}
