use std::{
    fs::{File, Permissions},
    io::Write,
    os::unix::fs::PermissionsExt,
};

use axum::Json;
use tempdir::TempDir;

use crate::{
    error::AppError,
    process::run_process,
    types::{Executable, ExecuteRequest, ProcessOutput},
};
use base64::{prelude::BASE64_STANDARD, Engine};

pub fn execute(payload: ExecuteRequest) -> anyhow::Result<ProcessOutput> {
    let tmp_dir = TempDir::new("execute")?;

    match payload.executable {
        Executable::Binary { value } => {
            let mut executable_file = File::create(tmp_dir.path().join("program"))?;
            executable_file.write_all(BASE64_STANDARD.decode(value)?.as_ref())?;
            executable_file.set_permissions(Permissions::from_mode(0o755))?;
            drop(executable_file);

            run_process(
                "./program",
                tmp_dir.path(),
                payload.options,
            )
        }
        Executable::JavaClass { class_name, value } => unimplemented!(),
        Executable::Script {
            language,
            source_code,
        } => unimplemented!(),
    }
}


pub async fn execute_handler(Json(payload): Json<ExecuteRequest>) -> Result<Json<ProcessOutput>, AppError> {
    Ok(Json(execute(payload)?))
}
