//! Project discovery, generated Cargo project, running cargo, and mapping
//! rustc diagnostics back to `.lrs` sources.

use crate::emit::{emit, Emitted, SourceMap};
use crate::model::*;
use crate::parse::{load, normalize, Overlay};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct Config {
    pub root: PathBuf,
    pub entry: PathBuf,
    /// Cargo manifest (the `[lrs]` table removed)
    pub manifest: toml::Table,
    pub is_lib: bool,
}

pub struct Analysis {
    pub project: Project,
    pub emitted: Emitted,
}

impl Analysis {
    pub fn diags(&self) -> Vec<Diag> {
        let mut v = self.project.diags.clone();
        v.extend(self.emitted.diags.iter().cloned());
        v
    }
    pub fn has_errors(&self) -> bool {
        self.diags().iter().any(|d| d.severity == Severity::Error)
    }
}

pub fn analyze(entry: &Path, root: &Path, overlay: &Overlay) -> Result<Analysis, String> {
    let project = load(entry, root, overlay)?;
    let emitted = emit(&project);
    Ok(Analysis { project, emitted })
}

/// The project's config file in `dir`: `lrs.toml`, or a `Cargo.toml` that has an `[lrs]` table.
fn config_file_in(dir: &Path) -> Option<PathBuf> {
    let lrs = dir.join("lrs.toml");
    if lrs.is_file() {
        return Some(lrs);
    }
    let cargo = dir.join("Cargo.toml");
    let text = std::fs::read_to_string(&cargo).ok()?;
    let t: toml::Table = text.parse().ok()?;
    t.contains_key("lrs").then_some(cargo)
}

pub fn find_config_dir(start: &Path) -> Option<PathBuf> {
    let mut d = Some(start);
    while let Some(x) = d {
        if config_file_in(x).is_some() {
            return Some(x.to_path_buf());
        }
        d = x.parent();
    }
    None
}

fn sanitize(name: &str) -> String {
    let s: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect();
    if s.is_empty() || s.chars().next().unwrap().is_ascii_digit() {
        format!("lrs_{}", s)
    } else {
        s
    }
}

/// `arg` may be a project directory, a `.lrs` file, or `.`.
pub fn find_project(arg: &Path) -> Result<Config, String> {
    if !arg.exists() {
        return Err(format!("{} does not exist", arg.display()));
    }
    let arg = normalize(arg);
    let (start_dir, explicit) = if arg.is_file() {
        (arg.parent().unwrap().to_path_buf(), Some(arg.clone()))
    } else {
        (arg.clone(), None)
    };
    let cfg_dir = find_config_dir(&start_dir);
    let (root, mut table) = match &cfg_dir {
        Some(d) => {
            let path = config_file_in(d).unwrap_or_else(|| d.join("lrs.toml"));
            let s = std::fs::read_to_string(&path).map_err(|e| format!("{}: {}", path.display(), e))?;
            let t: toml::Table = s.parse().map_err(|e| format!("{}: {}", path.display(), e))?;
            (d.clone(), t)
        }
        None => (start_dir.clone(), toml::Table::new()),
    };
    let lrs_cfg = table.remove("lrs");
    // Target sections point at the `.lrs` entry (needed by `cargo add` and friends);
    // the generated crate has its own `src/main.rs` or `src/lib.rs`, so drop them.
    table.remove("bin");
    table.remove("lib");
    let entry = match explicit {
        Some(e) => e,
        None => {
            let named = lrs_cfg.as_ref().and_then(|l| l.get("entry")).and_then(|v| v.as_str()).map(|s| s.to_string());
            match named {
                Some(n) => root.join(n),
                None if root.join("main.lrs").is_file() => root.join("main.lrs"),
                None => root.join("lib.lrs"),
            }
        }
    };
    if !entry.is_file() {
        return Err(format!("entry file {} not found (create main.lrs/lib.lrs or set [lrs] entry in lrs.toml)", entry.display()));
    }
    let entry = normalize(&entry);
    let is_lib = entry.file_stem().map_or(false, |s| s == "lib");

    if !table.contains_key("package") {
        let name = sanitize(root.file_name().and_then(|s| s.to_str()).unwrap_or("lrs_project"));
        let mut pkg = toml::Table::new();
        pkg.insert("name".into(), toml::Value::String(name));
        pkg.insert("version".into(), toml::Value::String("0.1.0".into()));
        table.insert("package".into(), toml::Value::Table(pkg));
    }
    if let Some(toml::Value::Table(pkg)) = table.get_mut("package") {
        pkg.entry("edition").or_insert(toml::Value::String("2021".into()));
        pkg.entry("version").or_insert(toml::Value::String("0.1.0".into()));
    }
    table.insert("workspace".into(), toml::Value::Table(toml::Table::new()));
    Ok(Config { root, entry, manifest: table, is_lib })
}

fn write_if_changed(path: &Path, content: &str) -> Result<(), String> {
    if std::fs::read_to_string(path).map_or(false, |c| c == content) {
        return Ok(());
    }
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {}", d.display(), e))?;
    }
    std::fs::write(path, content).map_err(|e| format!("{}: {}", path.display(), e))
}

pub fn gen_dir(cfg: &Config) -> PathBuf {
    cfg.root.join("target").join("lrs")
}

/// Write the generated Cargo project; returns its directory.
pub fn write_gen(cfg: &Config, code: &str) -> Result<PathBuf, String> {
    let dir = gen_dir(cfg);
    let manifest = toml::to_string(&cfg.manifest).map_err(|e| e.to_string())?;
    write_if_changed(&dir.join("Cargo.toml"), &manifest)?;
    let (name, other) = if cfg.is_lib { ("lib.rs", "main.rs") } else { ("main.rs", "lib.rs") };
    write_if_changed(&dir.join("src").join(name), code)?;
    let _ = std::fs::remove_file(dir.join("src").join(other));
    Ok(dir)
}

pub struct CargoResult {
    pub diags: Vec<Diag>,
    /// messages that could not be tied to a source location
    pub other: Vec<String>,
    pub success: bool,
    /// built binary, if any
    pub executable: Option<PathBuf>,
}

/// Run `cargo <sub>` in the generated project and map its diagnostics.
pub fn cargo_run(
    dir: &Path,
    sub: &str,
    release: bool,
    is_lib: bool,
    project: &Project,
    map: &SourceMap,
) -> Result<CargoResult, String> {
    let mut cmd = Command::new("cargo");
    cmd.arg(sub).arg("--message-format=json").arg("--quiet");
    if release {
        cmd.arg("--release");
    }
    let out = cmd
        .current_dir(dir)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("failed to run cargo: {}", e))?;
    let gen_name = if is_lib { "src/lib.rs" } else { "src/main.rs" };
    let mut diags = vec![];
    let mut other = vec![];
    let mut success = out.status.success();
    let mut executable = None;
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        match v["reason"].as_str() {
            Some("compiler-message") => {
                if let Some(d) = convert_message(&v["message"], project, map, gen_name, &mut other) {
                    diags.push(d);
                }
            }
            Some("compiler-artifact") => {
                if let Some(e) = v["executable"].as_str() {
                    executable = Some(PathBuf::from(e));
                }
            }
            Some("build-finished") => {
                if let Some(s) = v["success"].as_bool() {
                    success = s;
                }
            }
            _ => {}
        }
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !stderr.trim().is_empty() && !success {
        other.push(stderr.trim().to_string());
    }
    Ok(CargoResult { diags, other, success, executable })
}

fn convert_message(m: &Value, project: &Project, map: &SourceMap, gen_name: &str, other: &mut Vec<String>) -> Option<Diag> {
    let level = m["level"].as_str().unwrap_or("error");
    let severity = match level {
        "error" | "error: internal compiler error" => Severity::Error,
        "warning" => Severity::Warning,
        "note" | "help" => Severity::Info,
        _ => return None,
    };
    let msg = m["message"].as_str().unwrap_or("").to_string();
    let spans = m["spans"].as_array().cloned().unwrap_or_default();
    if spans.is_empty()
        && (msg.starts_with("aborting due to")
            || msg.contains("warning emitted")
            || msg.contains("warnings emitted")
            || msg.starts_with("For more information about this error"))
    {
        return None;
    }
    let primary = spans.iter().find(|s| s["is_primary"].as_bool() == Some(true)).or(spans.first());
    let mut message = msg.clone();
    if let Some(label) = primary.and_then(|s| s["label"].as_str()) {
        if !label.is_empty() {
            message.push('\n');
            message.push_str(label);
        }
    }
    for c in m["children"].as_array().into_iter().flatten() {
        let cl = c["level"].as_str().unwrap_or("note");
        let cm = c["message"].as_str().unwrap_or("");
        if !cm.is_empty() {
            message.push_str(&format!("\n{}: {}", cl, cm));
        }
    }
    let code = m["code"]["code"].as_str().map(|s| s.to_string());
    let loc = primary.and_then(|s| {
        if !s["file_name"].as_str()?.ends_with(gen_name) {
            return None;
        }
        let bs = s["byte_start"].as_u64()? as usize;
        let be = s["byte_end"].as_u64()? as usize;
        let (f1, a) = map.start(bs)?;
        let (f2, b) = map.end(be.max(bs))?;
        if f1 != f2 || f1 >= project.files.len() {
            return Some((f1, a, a));
        }
        Some((f1, a, b.max(a)))
    });
    match loc {
        Some((f, a, b)) => Some(Diag { file: Some(f), start: a, end: b, severity, message, code }),
        None => {
            if severity != Severity::Info {
                other.push(format!("{}: {}", level, message));
            }
            None
        }
    }
}

/// Render a diagnostic for the terminal.
pub fn render_diag(p: &Project, d: &Diag, cwd: &Path) -> String {
    let level = match d.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "note",
    };
    let mut lines = d.message.lines();
    let first = lines.next().unwrap_or("");
    let mut s = match &d.code {
        Some(c) => format!("{}[{}]: {}\n", level, c, first),
        None => format!("{}: {}\n", level, first),
    };
    let rest: Vec<&str> = lines.collect();
    if let Some(fi) = d.file {
        let f = &p.files[fi];
        let (l, c) = f.lines.line_col_chars(&f.text, d.start);
        let (l2, c2) = f.lines.line_col_chars(&f.text, d.end);
        let text = f.lines.line_text(&f.text, l);
        let shown = f.path.strip_prefix(cwd).unwrap_or(&f.path).display().to_string();
        let ln = (l + 1).to_string();
        let pad = " ".repeat(ln.len());
        let width = if l2 == l { c2.saturating_sub(c) } else { text.chars().count().saturating_sub(c) }.max(1);
        s.push_str(&format!("{pad}--> {}:{}:{}\n{pad} |\n{ln} | {}\n{pad} | {}{}\n", shown, l + 1, c + 1, text, " ".repeat(c), "^".repeat(width)));
        for r in rest {
            s.push_str(&format!("{pad} = {}\n", r));
        }
    } else {
        for r in rest {
            s.push_str(&format!("  = {}\n", r));
        }
    }
    s
}
