use std::{
    fs::{File, Permissions},
    io::Write,
    os::unix::fs::PermissionsExt,
};

use anyhow::Result;
use axum::Json;
use serde::Deserialize;
use tempdir::TempDir;

use crate::{
    error::AppError,
    run_command::{run_command, CommandOptions, CommandOutput},
    types::Executable,
};
use base64::{prelude::BASE64_STANDARD, Engine};

#[derive(Deserialize)]
pub struct ExecuteRequest {
    pub executable: Executable,
    pub options: CommandOptions,
}

pub fn execute(payload: ExecuteRequest) -> Result<CommandOutput> {
    let tmp_dir = TempDir::new("execute")?;

    match payload.executable {
        Executable::Binary { value } => {
            let mut executable_file = File::create(tmp_dir.path().join("program"))?;
            executable_file.write_all(BASE64_STANDARD.decode(value)?.as_ref())?;
            executable_file.set_permissions(Permissions::from_mode(0o755))?;
            drop(executable_file);

            run_command("./program", tmp_dir.path(), payload.options)
        }
        Executable::JavaClass { class_name, value } => {
            let mut class_file = File::create(
                tmp_dir
                    .path()
                    .join(class_name.clone())
                    .with_extension("class"),
            )?;
            class_file.write_all(BASE64_STANDARD.decode(value)?.as_ref())?;
            drop(class_file);

            run_command(
                format!("java {}", class_name).as_ref(),
                tmp_dir.path(),
                payload.options,
            )
        }
        Executable::Script {
            language,
            source_code,
        } => unimplemented!(),
    }
}

pub async fn execute_handler(
    Json(payload): Json<ExecuteRequest>,
) -> Result<Json<CommandOutput>, AppError> {
    Ok(Json(execute(payload)?))
}
