//! Provides a function to run a command and return the output.

use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::{os::unix::process::ExitStatusExt, process::Command, str};

use anyhow::Context;
use anyhow::Result;
use nix::sys::signal::Signal;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CommandOptions {
    pub stdin: String,
    pub timeout_ms: u32,
}

#[derive(Serialize)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,

    /// The underlying raw wait status. Note that this is different from an exit status.
    pub exit_code: i32,

    pub exit_signal: Option<String>,

    // /**
    //  * When executing, if `fileIOName` is given, this is
    //  * set to whatever is written in `[fileIOName].out`
    //  * or null if there's no such file.
    //  */
    // pub file_output: Option<String>,
}

pub fn run_command(command: &str, working_dir: &Path, options: CommandOptions) -> Result<CommandOutput> {
    let mut process = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "{} {}s {command}",
            if cfg!(target_os = "macos") {
                // ulimit -s unlimited does not work on mac os
                // use `brew install gtime` to install linux time on mac os
                // use `brew install timeout` to install linux timeout on mac os
                "gtime -v timeout"
            } else {
                "ulimit -c 0 && ulimit -s unlimited && /usr/bin/time -v /usr/bin/timeout"
            },
            options.timeout_ms / 1000
        ))
        .current_dir(working_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn command {command}"))?;

    let mut stdin_pipe = process.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        // Note: This may be due to a broken pipe if the program closes their stdin pipe.
        // This thread panicing does not crash the main thread.
        stdin_pipe.write_all(options.stdin.as_bytes()).expect("Failed to write to stdin");
    });

    let process = process.wait_with_output()?;

    Ok(CommandOutput {
        exit_code: process.status.into_raw(),
        exit_signal: process.status.signal().map(|signal| {
            Signal::try_from(signal).map_or(format!("Unknown signal {signal}"), |signal| {
                signal.to_string()
            })
        }),
        stdout: str::from_utf8(&process.stdout)?.to_string(),
        stderr: str::from_utf8(&process.stderr)?.to_string(),
    })
}
