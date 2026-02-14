#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod spec;
mod runner;
mod registry;
mod help_parser;
mod import;

use spec::{CommandSpec, NodeInvocation, RunResult};

#[tauri::command]
fn list_specs() -> Vec<CommandSpec> {
    registry::all_specs()
}

#[tauri::command]
fn run_node(invocation: NodeInvocation) -> Result<RunResult, String> {
    let specs = registry::all_specs();
    runner::run_one(&specs, invocation).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_tool(exe_path: String, tool_id: Option<String>, title: Option<String>) -> Result<CommandSpec, String> {
    // Defaults: tool_id derived from basename, title = basename
    let base = exe_path
        .split(std::path::MAIN_SEPARATOR)
        .last()
        .unwrap_or("tool")
        .to_string();

    let tid = tool_id.unwrap_or_else(|| format!("cli.{}", base.replace(' ', "_").to_lowercase()));
    let ttitle = title.unwrap_or_else(|| base);

    import::import_tool(exe_path, tid, ttitle).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![list_specs, run_node, import_tool])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
