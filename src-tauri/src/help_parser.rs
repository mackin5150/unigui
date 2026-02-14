use crate::spec::{ArgKind, ArgSpec, CommandSpec, IOSpec};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct ToolIntrospect {
    pub program: String,
    pub version: Option<String>,
    pub help: String,
}

fn guess_kind(placeholder: &str) -> ArgKind {
    let p = placeholder.to_uppercase();
    if p.contains("FILE") || p.ends_with(".JSON") || p.ends_with(".TXT") {
        ArgKind::PathFile
    } else if p.contains("DIR") || p.contains("FOLDER") {
        ArgKind::PathDir
    } else if p == "N" || p.contains("NUM") || p.contains("INT") || p.contains("COUNT") {
        ArgKind::Int
    } else {
        ArgKind::String
    }
}

fn normalize_positional(name: &str) -> String {
    name.trim()
        .trim_matches(|c: char| c == '<' || c == '>' || c == '[' || c == ']')
        .to_lowercase()
}

pub fn parse_help_to_spec(tool_id: String, title: String, ti: ToolIntrospect) -> CommandSpec {
    // Heuristics:
    // - Parse lines like: "  -o, --out <FILE>   Write to file"
    // - Parse lines like: "      --threads N    Number of threads"
    // - Parse Usage: line for positionals: "Usage: prog [opts] INPUT OUTPUT"
    let re_flag = Regex::new(
        r#"(?x)
        ^\s*
        (?P<short>-\w)?
        (?:,\s*)?
        (?P<long>--[a-zA-Z0-9][a-zA-Z0-9\-_]*)?
        (?:\s+(?P<val><[^>]+>|[A-Z][A-Z0-9_\-]+))?
        (?:\s+|\t+)
        (?P<help>.*)
        $"#,
    )
    .unwrap();

    let re_usage = Regex::new(r#"(?i)^\s*usage:\s*(?P<rest>.+)$"#).unwrap();

    let mut args: Vec<ArgSpec> = vec![];
    let mut positionals: Vec<(String, bool)> = vec![]; // (name, required)
    let mut usage_program: Option<String> = None;

    // 1) Find usage positionals (best-effort)
    for line in ti.help.lines() {
        if let Some(c) = re_usage.captures(line) {
            let rest = c.name("rest").unwrap().as_str().trim();

            // rest often like: "prog [OPTIONS] <INPUT> [OUTPUT]"
            // first token = program
            let mut tokens: Vec<&str> = rest.split_whitespace().collect();
            if !tokens.is_empty() {
                usage_program = Some(tokens.remove(0).to_string());
            }

            for t in tokens {
                // Skip obvious option markers
                let up = t.to_uppercase();
                if up.contains("[OPTIONS]") || up.contains("OPTIONS") || t.starts_with('-') {
                    continue;
                }
                // Required if not wrapped in []
                let required = !(t.starts_with('[') && t.ends_with(']'));
                let name = normalize_positional(t);
                if name.is_empty() || name == "..." {
                    continue;
                }
                // Avoid duplicates
                if !positionals.iter().any(|(n, _)| n == &name) {
                    positionals.push((name, required));
                }
            }
            break; // first usage line only
        }
    }

    // 2) Parse flags from help text
    for line in ti.help.lines() {
        let line = line.trim_end();
        // Skip empty
        if line.trim().is_empty() {
            continue;
        }
        // Skip headers
        let lower = line.to_lowercase();
        if lower.starts_with("options:") || lower.starts_with("flags:") || lower.starts_with("arguments:") {
            continue;
        }

        if let Some(c) = re_flag.captures(line) {
            let short = c.name("short").map(|m| m.as_str().to_string());
            let long = c.name("long").map(|m| m.as_str().to_string());
            let val = c.name("val").map(|m| m.as_str().to_string());
            let help = c.name("help").map(|m| m.as_str().trim().to_string()).filter(|s| !s.is_empty());

            // Ignore lines that don't look like flags
            if short.is_none() && long.is_none() {
                continue;
            }

            // Choose arg name key: prefer long
            let key = long.clone().or(short.clone()).unwrap();

            let (kind, required) = if let Some(v) = val {
                (guess_kind(&v), false)
            } else {
                (ArgKind::FlagBool, false)
            };

            // Avoid duplicates
            if args.iter().any(|a| a.name == key) {
                continue;
            }

            args.push(ArgSpec {
                name: key,
                kind,
                required,
                default: None,
                help,
            });
        }
    }

    // 3) Append positionals at the end (so UI shows them clearly)
    for (p, required) in positionals {
        if args.iter().any(|a| a.name == p) {
            continue;
        }
        args.push(ArgSpec {
            name: p,
            kind: ArgKind::String,
            required,
            default: None,
            help: Some("Positional argument (from Usage)".to_string()),
        });
    }

    // stdin/stdout guess:
    // - Most CLIs print to stdout; we always capture stdout/stderr anyway.
    // - We’ll mark stdin=false unless we see hints (Phase 3 improves this).
    let io = IOSpec { stdin: false, stdout: true, stderr: true, outputs_files: false };

    let desc = if let Some(v) = ti.version.as_deref() {
        format!("Imported CLI: {} (version {})", ti.program, v)
    } else {
        format!("Imported CLI: {}", ti.program)
    };

    CommandSpec {
        id: tool_id,
        title,
        description: desc,
        program: usage_program.unwrap_or(ti.program),
        args,
        io,
    }
}
