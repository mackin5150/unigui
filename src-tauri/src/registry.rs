use crate::spec::{ArgKind, ArgSpec, CommandSpec, IOSpec};
use directories::ProjectDirs;
use std::{fs, path::{Path, PathBuf}};

fn project_dirs() -> ProjectDirs {
    // org, app — pick stable names
    ProjectDirs::from("co", "prestine", "universal-gui").expect("ProjectDirs unavailable")
}

fn specs_dir() -> PathBuf {
    let pd = project_dirs();
    let dir = pd.data_dir().join("specs");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn load_user_specs() -> Vec<CommandSpec> {
    let dir = specs_dir();
    let mut out = vec![];

    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            if let Ok(txt) = fs::read_to_string(&p) {
                if let Ok(spec) = serde_json::from_str::<CommandSpec>(&txt) {
                    out.push(spec);
                }
            }
        }
    }

    out
}

pub fn save_user_spec(spec: &CommandSpec) -> anyhow::Result<()> {
    let dir = specs_dir();
    let safe = spec.id.replace('/', "_").replace('\\', "_").replace(':', "_");
    let path = dir.join(format!("{safe}.json"));
    let txt = serde_json::to_string_pretty(spec)?;
    fs::write(path, txt)?;
    Ok(())
}

pub fn builtin_specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            id: "core.echo".to_string(),
            title: "Echo".to_string(),
            description: "Demo node: prints text".to_string(),
            program: "echo".to_string(),
            args: vec![
                ArgSpec {
                    name: "text".to_string(),
                    kind: ArgKind::String,
                    required: true,
                    default: None,
                    help: Some("Text to print".to_string()),
                },
            ],
            io: IOSpec { stdin: false, stdout: true, stderr: true, outputs_files: false },
        },
        CommandSpec {
            id: "core.cat".to_string(),
            title: "Cat (stdin)".to_string(),
            description: "Demo node: echoes stdin (type into input)".to_string(),
            program: "cat".to_string(),
            args: vec![],
            io: IOSpec { stdin: true, stdout: true, stderr: true, outputs_files: false },
        },
    ]
}

pub fn all_specs() -> Vec<CommandSpec> {
    let mut specs = builtin_specs();
    let mut user = load_user_specs();

    // Ensure no collisions: user specs override builtin by id
    specs.retain(|b| !user.iter().any(|u| u.id == b.id));
    specs.append(&mut user);

    specs
}
