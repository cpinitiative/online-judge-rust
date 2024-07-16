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
