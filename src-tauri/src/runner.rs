use crate::spec::{CommandSpec, NodeInvocation, RunResult};
use std::process::{Command, Stdio};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunError {
    #[error("unknown spec id: {0}")]
    UnknownSpec(String),

    #[error("failed to spawn process: {0}")]
    Spawn(String),

    #[error("process IO error: {0}")]
    Io(String),
}

pub fn build_argv(spec: &CommandSpec, inv: &NodeInvocation) -> Vec<String> {
    // Simple baseline: for each ArgSpec, if value exists -> push appropriately
    // Phase 2 will improve: flags, repeated args, positionals, type coercion, quoting
    let mut argv: Vec<String> = vec![];

    for a in &spec.args {
        if let Some(v) = inv.values.get(&a.name) {
            match v {
                serde_json::Value::Bool(b) => {
                    if *b {
                        argv.push(a.name.clone());
                    }
                }
                serde_json::Value::Number(n) => {
                    argv.push(a.name.clone());
                    argv.push(n.to_string());
                }
                serde_json::Value::String(s) => {
                    // If it's a positional like "text" (no leading dash), we still push it.
                    if a.name.starts_with("--") || a.name.starts_with("-") {
                        argv.push(a.name.clone());
                    }
                    argv.push(s.clone());
                }
                _ => {}
            }
        } else if let Some(def) = &a.default {
            // Apply default if present
            if let Some(s) = def.as_str() {
                if a.name.starts_with("--") || a.name.starts_with("-") {
                    argv.push(a.name.clone());
                }
                argv.push(s.to_string());
            }
        }
    }

    argv
}

pub fn run_one(specs: &[CommandSpec], inv: NodeInvocation) -> Result<RunResult, RunError> {
    let spec = specs.iter().find(|s| s.id == inv.spec_id).ok_or_else(|| RunError::UnknownSpec(inv.spec_id.clone()))?;

    let argv = build_argv(spec, &inv);

    let mut cmd = Command::new(&spec.program);
    cmd.args(argv);

    if let Some(wd) = inv.workdir {
        cmd.current_dir(wd);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    if inv.stdin_text.is_some() {
        cmd.stdin(Stdio::piped());
    }

    let mut child = cmd.spawn().map_err(|e| RunError::Spawn(e.to_string()))?;

    if let Some(input) = inv.stdin_text {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(input.as_bytes()).map_err(|e| RunError::Io(e.to_string()))?;
        }
    }

    let out = child.wait_with_output().map_err(|e| RunError::Io(e.to_string()))?;

    Ok(RunResult {
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        exit_code: out.status.code().unwrap_or(-1),
    })
}
