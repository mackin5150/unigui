use crate::{help_parser::{parse_help_to_spec, ToolIntrospect}, registry, spec::CommandSpec};
use std::process::Command;

fn run_capture(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    let s = String::from_utf8_lossy(&out.stdout).to_string();
    if s.trim().is_empty() {
        // Some tools print help/version to stderr
        let e = String::from_utf8_lossy(&out.stderr).to_string();
        if e.trim().is_empty() { None } else { Some(e) }
    } else {
        Some(s)
    }
}

pub fn introspect(exe_path: &str) -> anyhow::Result<ToolIntrospect> {
    let help = run_capture(exe_path, &["--help"])
        .or_else(|| run_capture(exe_path, &["-h"]))
        .unwrap_or_else(|| "Help not available (no --help)".to_string());

    let version = run_capture(exe_path, &["--version"])
        .or_else(|| run_capture(exe_path, &["-V"]))
        .map(|s| s.lines().next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty());

    Ok(ToolIntrospect {
        program: exe_path.to_string(),
        version,
        help,
    })
}

pub fn import_tool(exe_path: String, tool_id: String, title: String) -> anyhow::Result<CommandSpec> {
    let ti = introspect(&exe_path)?;
    let spec = parse_help_to_spec(tool_id, title, ti);
    registry::save_user_spec(&spec)?;
    Ok(spec)
}
