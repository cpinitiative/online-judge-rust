use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "cpp")]
    Cpp,
    #[serde(rename = "java21")]
    Java21,
    #[serde(rename = "py11")]
    Py11,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Executable {
    #[serde(rename = "binary")]
    Binary { value: String },

    #[serde(rename = "java_class")]
    JavaClass { class_name: String, value: String },

    #[serde(rename = "script")]
    Script {
        language: Language,
        source_code: String,
    },
}

#[derive(Serialize)]
pub struct ProcessOutput {
    pub stdout: String,
    pub stderr: String,

    /// Returns the underlying raw wait status.
    /// Note that this is a *wait status*, not an *exit status*.
    pub exit_code: i32,

    pub exit_signal: Option<String>,
    // todo: I'm pretty sure this only happens if the user passes wrong compilation flags or something
    // pub process_error: String,

    // /**
    //  * When executing, if `fileIOName` is given, this is
    //  * set to whatever is written in `[fileIOName].out`
    //  * or null if there's no such file.
    //  */
    // pub file_output: Option<String>,
}

/// Payload for POST /compile
///
/// Compiles the given code and returns it as a base64-encoded ZIP file. Running `run.sh` after
/// extracting the ZIP file will run the compiled binary.
#[derive(Deserialize)]
pub struct CompileRequest {
    pub source_code: String,
    pub compiler_options: String,
    pub language: Language,
}

/// Response for POST /compile
#[derive(Serialize)]
pub struct CompileResponse {
    /// None if the compilation did not succeed.
    pub executable: Option<Executable>,

    /// Process output of the compilation command.
    pub process_output: ProcessOutput,
}

/// Payload for POST /compile-and-execute
///
/// Called when the user wants to compile and execute the given code with a single lambda call.
/// Used by the USACO Guide IDE's "execute code" functionality.
#[derive(Deserialize)]
pub struct CompileAndExecuteRequest {}

/// Response for POST /compile-and-execute
#[derive(Serialize)]
pub struct CompileAndExecuteResponse {
    pub compile_result: String,
}
