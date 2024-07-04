use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::{os::unix::process::ExitStatusExt, process::Command, str};

use anyhow::Context;
use anyhow::Result;
use nix::sys::signal::Signal;

use crate::types::{ExecuteOptions, ProcessOutput};

pub fn run_process(command: &str, working_dir: &Path, options: ExecuteOptions) -> Result<ProcessOutput> {
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

    Ok(ProcessOutput {
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
