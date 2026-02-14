use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpec {
    pub id: String,              // e.g. "core.echo"
    pub title: String,           // e.g. "Echo"
    pub description: String,
    pub program: String,         // e.g. "echo"
    pub args: Vec<ArgSpec>,      // flags/positionals
    pub io: IOSpec,              // stdin/stdout/files
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgSpec {
    pub name: String,            // e.g. "--count" or "text"
    pub kind: ArgKind,
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ArgKind {
    FlagBool,                    // --verbose
    Int,                         // --count 3
    Float,
    String,
    PathFile,
    PathDir,
    Enum(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IOSpec {
    pub stdin: bool,
    pub stdout: bool,
    pub stderr: bool,
    pub outputs_files: bool,     // runner will track created files later (Phase 2+)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInvocation {
    pub spec_id: String, // points to CommandSpec.id
    pub values: serde_json::Map<String, serde_json::Value>,
    pub stdin_text: Option<String>,
    pub workdir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}
