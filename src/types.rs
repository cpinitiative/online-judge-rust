use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Language {
    #[serde(rename = "cpp")]
    Cpp,
    #[serde(rename = "java21")]
    Java21,
    #[serde(rename = "py12")]
    Py12,
}

// for the future, when implementing grader / scorer support, we probably want to add an "additional_files" field to executable.
// grader: https://probgate.org/viewsolution.php?grader_id=557
// needs 3 files, input, output, answer
// scorer: https://probgate.org/viewsolution.php?scorer_id=4
// needs N files, one for each test case

#[derive(Serialize, Deserialize)]
pub struct Executable {
    /// base64 .tar.gz file
    pub files: String,
    pub run_command: String,
}
