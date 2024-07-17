//! Provides a function to run a command and return the output.

use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::{os::unix::process::ExitStatusExt, process::Command, str};

use anyhow::Result;
use anyhow::{anyhow, Context};
use nix::sys::signal::Signal;
use regex::Regex;
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
    pub wall_time: String, // time format is 0:00.00
    pub memory_usage: String,

    /// The underlying raw wait status. Note that this is different from an exit status.
    pub exit_code: i32,
    pub exit_signal: Option<String>,
}

struct TimingOutput {
    stderr: String,
    wall_time: String,
    memory_usage: String,
}

fn parse_timing_stderr(stderr: &str) -> Result<TimingOutput> {
    let start_index = stderr.rfind("\tCommand being timed:");
    if start_index.is_none() {
        return Err(anyhow!(
            "Failed to parse timing output: Couldn't find \"Command being timed\""
        ));
    }
    let start_index = start_index.unwrap();
    let rest_of_string = stderr[..start_index].to_string();
    let time_output = &stderr[start_index..];

    let wall_time_re =
        Regex::new(r"\tElapsed \(wall clock\) time \(h:mm:ss or m:ss\): (.+)").unwrap();
    let memory_usage_re = Regex::new(r"\tMaximum resident set size \(kbytes\): (.+)").unwrap();

    let wall_time = wall_time_re
        .captures(time_output)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow!("Failed to parse wall time: Couldn't match regex"))?;

    let memory_usage = memory_usage_re
        .captures(time_output)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow!(
            "Failed to parse memory usage: Couldn't match regex"
        ))?;

    let wall_time = if wall_time.len() > 3 {
        wall_time[3..].to_string()
    } else {
        return Err(anyhow!(
            "Failed to parse wall time: matched regex length less than expected"
        ));
    };

    Ok(TimingOutput {
        stderr: rest_of_string,
        wall_time,
        memory_usage,
    })
}

pub fn run_command(
    command: &str,
    working_dir: &Path,
    options: CommandOptions,
) -> Result<CommandOutput> {
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
        stdin_pipe
            .write_all(options.stdin.as_bytes())
            .expect("Failed to write to stdin");
    });

    let process = process.wait_with_output()?;

    let timing_output = parse_timing_stderr(str::from_utf8(&process.stderr)?)?;

    Ok(CommandOutput {
        exit_code: process.status.into_raw(),
        exit_signal: process.status.signal().map(|signal| {
            Signal::try_from(signal).map_or(format!("Unknown signal {signal}"), |signal| {
                signal.to_string()
            })
        }),
        stdout: str::from_utf8(&process.stdout)?.to_string(),
        stderr: timing_output.stderr,
        wall_time: timing_output.wall_time,
        memory_usage: timing_output.memory_usage,
    })
}
