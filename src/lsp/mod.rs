//! `lrs lsp`: a Language Server Protocol server over stdio.
//!
//! Every `.lrs` file gets: diagnostics (include/namespace errors immediately,
//! rustc errors mapped back to .lrs on save), go-to-definition (paths,
//! namespace headers, `include` strings), hover, `.`-completion of namespaces
//! and items, and document symbols.

mod features;
mod ra;
mod rust_completion;

use crate::driver::*;
use ra::{Ra, RaCaps};
use rust_completion::ItemStore;
use crate::model::*;
use crate::parse::{normalize, Overlay};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

pub fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let bytes = rest.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Some(v) = std::str::from_utf8(&bytes[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    let mut s = String::from_utf8(out).ok()?;
    // file:///C:/dir -> C:/dir
    if s.len() > 2 && s.as_bytes()[0] == b'/' && s.as_bytes()[2] == b':' {
        s.remove(0);
    }
    Some(PathBuf::from(s))
}

pub fn path_to_uri(p: &Path) -> String {
    let s = p.to_string_lossy().replace('\\', "/");
    let mut out = String::from("file://");
    if !s.starts_with('/') {
        out.push('/');
    }
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

enum Event {
    Msg(Value),
    Eof,
    CargoDone { entry: PathBuf, analysis: Arc<Analysis>, result: Result<CargoResult, String> },
}

fn reader_thread(tx: Sender<Event>) {
    let stdin = std::io::stdin();
    let mut r = stdin.lock();
    loop {
        let mut len = None;
        loop {
            let mut line = String::new();
            match r.read_line(&mut line) {
                Ok(0) | Err(_) => {
                    let _ = tx.send(Event::Eof);
                    return;
                }
                Ok(_) => {}
            }
            let l = line.trim();
            if l.is_empty() {
                break;
            }
            if let Some(v) = l.strip_prefix("Content-Length:") {
                len = v.trim().parse::<usize>().ok();
            }
        }
        let Some(n) = len else { continue };
        let mut buf = vec![0u8; n];
        if r.read_exact(&mut buf).is_err() {
            let _ = tx.send(Event::Eof);
            return;
        }
        if let Ok(v) = serde_json::from_slice::<Value>(&buf) {
            if tx.send(Event::Msg(v)).is_err() {
                return;
            }
        }
    }
}

struct Server {
    tx: Sender<Event>,
    docs: Overlay,
    /// entry path -> analysis
    cache: HashMap<PathBuf, Arc<Analysis>>,
    /// open doc -> entry key
    doc_entry: HashMap<PathBuf, PathBuf>,
    /// entry -> rustc diagnostics (already LSP-formatted) keyed by file path
    cargo_diags: HashMap<PathBuf, Vec<(PathBuf, Value)>>,
    cargo_running: HashSet<PathBuf>,
    cargo_rerun: HashSet<PathBuf>,
    published: HashSet<String>,
    shutdown: bool,
    /// entry path -> rust-analyzer on its generated crate
    ra_sessions: HashMap<PathBuf, RaSession>,
    store: Arc<ItemStore>,
    ra_caps: RaCaps,
}

/// One rust-analyzer instance and the analysis it was last given.
struct RaSession {
    ra: Ra,
    synced: Option<Arc<Analysis>>,
}

/// Why the generated crate is not written: appended to `$TMP/lrs-debug.log`.
fn debug_log(msg: &str) {
    use std::io::Write;
    let path = std::env::temp_dir().join("lrs-debug.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{msg}");
    }
}

/// Enter between `{` and `}` (cursor on the `}` line, after the client's own indent).
/// The middle line takes the `{` line's indent plus one unit; `}` keeps the `{` line's indent.
/// The unit follows the file: a tab if the file already indents with tabs, otherwise four spaces.
/// Edits are computed from the text itself, not from the whitespace the client left before the cursor.
fn braces_enter_edits(text: &str, line: usize, col: usize) -> Option<Value> {
    let lines: Vec<&str> = text.split('\n').collect();
    let cur = lines.get(line)?;
    let prev = lines.get(line.checked_sub(1)?)?;
    let (before, after) = (cur.get(..col)?, cur.get(col..)?);
    // only whitespace before the cursor, `}` after it, and `{` ending the previous line
    if !before.chars().all(|c| c == ' ' || c == '\t') || !after.starts_with('}') || !prev.trim_end().ends_with('{') {
        return None;
    }
    let prev_indent: String = prev.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
    let unit = if prev_indent.contains('\t') || text.lines().any(|l| l.starts_with('\t')) { "\t" } else { "    " };
    let at = |c: usize| json!({"line": line, "character": c});
    // replace the cursor's leading whitespace with the middle line's indent, then open the `}` line after the cursor
    Some(json!([
        {"range": {"start": at(0), "end": at(col)}, "newText": format!("{prev_indent}{unit}")},
        {"range": {"start": at(col), "end": at(col)}, "newText": format!("\n{prev_indent}")}
    ]))
}

fn respond(id: Value, result: Value) {
    send(&json!({"jsonrpc":"2.0","id":id,"result":result}));
}

fn send(v: &Value) {
    let body = v.to_string();
    let stdout = std::io::stdout();
    let mut o = stdout.lock();
    let _ = write!(o, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = o.flush();
}

fn lsp_diag(a: &Analysis, d: &Diag, source: &str) -> Option<(PathBuf, Value)> {
    let fid = d.file?;
    let f = a.project.files.get(fid)?;
    Some((
        f.path.clone(),
        json!({
            "range": features::lsp_range(f, d.start, d.end),
            "severity": match d.severity { Severity::Error => 1, Severity::Warning => 2, Severity::Info => 3 },
            "source": source,
            "code": d.code,
            "message": d.message,
        }),
    ))
}

impl Server {
    fn invalidate(&mut self) {
        self.cache.clear();
    }

    /// Analysis of the project a document belongs to (cached).
    fn analysis_for(&mut self, path: &Path) -> Option<Arc<Analysis>> {
        let path = normalize(path);
        if let Some(entry) = self.doc_entry.get(&path) {
            if let Some(a) = self.cache.get(entry) {
                return Some(a.clone());
            }
        }
        let dir = path.parent()?.to_path_buf();
        if let Some(cd) = find_config_dir(&dir) {
            if let Ok(cfg) = find_project(&cd) {
                let a = match self.cache.get(&cfg.entry) {
                    Some(a) => a.clone(),
                    None => {
                        let a = Arc::new(analyze(&cfg.entry, &cfg.root, &self.docs).ok()?);
                        self.cache.insert(cfg.entry.clone(), a.clone());
                        a
                    }
                };
                if a.project.file_index(&path).is_some() {
                    self.doc_entry.insert(path, cfg.entry);
                    return Some(a);
                }
            }
        }
        // standalone file
        let a = match self.cache.get(&path) {
            Some(a) => a.clone(),
            None => {
                let a = Arc::new(analyze(&path, &dir, &self.docs).ok()?);
                self.cache.insert(path.clone(), a.clone());
                a
            }
        };
        self.doc_entry.insert(path.clone(), path);
        Some(a)
    }

    fn publish_all(&mut self) {
        let docs: Vec<PathBuf> = self.docs.keys().cloned().collect();
        let mut per_uri: HashMap<String, Vec<Value>> = HashMap::new();
        let mut seen_entries = HashSet::new();
        for d in docs {
            let Some(a) = self.analysis_for(&d) else { continue };
            let entry = self.doc_entry.get(&d).cloned().unwrap_or_else(|| d.clone());
            if !seen_entries.insert(entry.clone()) {
                continue;
            }
            for f in &a.project.files {
                per_uri.entry(path_to_uri(&f.path)).or_default();
            }
            for diag in a.diags() {
                if let Some((path, v)) = lsp_diag(&a, &diag, "lrs") {
                    per_uri.entry(path_to_uri(&path)).or_default().push(v);
                }
            }
            for (path, v) in self.cargo_diags.get(&entry).into_iter().flatten() {
                per_uri.entry(path_to_uri(path)).or_default().push(v.clone());
            }
        }
        for uri in self.published.iter() {
            if !per_uri.contains_key(uri) {
                send(&json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"diagnostics":[]}}));
            }
        }
        self.published.clear();
        for (uri, diags) in per_uri {
            send(&json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"diagnostics":diags}}));
            self.published.insert(uri);
        }
    }

    fn start_cargo(&mut self, path: &Path) {
        let Some(a) = self.analysis_for(path) else { return };
        let path = normalize(path);
        let Some(entry) = self.doc_entry.get(&path).cloned() else { return };
        if a.has_errors() {
            return;
        }
        if self.cargo_running.contains(&entry) {
            self.cargo_rerun.insert(entry);
            return;
        }
        let cfg_arg = find_config_dir(path.parent().unwrap_or(Path::new("."))).unwrap_or_else(|| entry.clone());
        let Ok(mut cfg) = find_project(&cfg_arg) else { return };
        cfg.entry = entry.clone();
        cfg.is_lib = entry.file_stem().map_or(false, |s| s == "lib");
        let Ok(dir) = write_gen(&cfg, &a.emitted.code) else { return };
        self.cargo_running.insert(entry.clone());
        let tx = self.tx.clone();
        let is_lib = cfg.is_lib;
        std::thread::spawn(move || {
            let result = cargo_run(&dir, "check", false, is_lib, &a.project, &a.emitted.map);
            let _ = tx.send(Event::CargoDone { entry, analysis: a, result });
        });
    }

    fn on_cargo_done(&mut self, entry: PathBuf, a: Arc<Analysis>, result: Result<CargoResult, String>) {
        self.cargo_running.remove(&entry);
        if let Ok(r) = result {
            let mut v: Vec<(PathBuf, Value)> = r.diags.iter().filter_map(|d| lsp_diag(&a, d, "rustc")).collect();
            // a failure with no location (e.g. dependency resolution) is shown on the entry file
            if !r.success && v.is_empty() && !r.other.is_empty() {
                let f = &a.project.files[0];
                v.push((
                    f.path.clone(),
                    json!({"range": features::lsp_range(f, 0, 0), "severity": 1, "source": "cargo", "message": r.other.join("\n")}),
                ));
            }
            self.cargo_diags.insert(entry.clone(), v);
        }
        if self.cargo_rerun.remove(&entry) {
            self.start_cargo(&entry);
        }
        self.publish_all();
    }

    fn handle(&mut self, msg: Value) -> bool {
        let method = msg["method"].as_str().unwrap_or("").to_string();
        let id = msg.get("id").cloned();
        let params = &msg["params"];
        match (method.as_str(), id) {
            ("initialize", Some(id)) => {
                // what the editor can do with completion items is what rust-analyzer is told too
                let item = &params["capabilities"]["textDocument"]["completion"]["completionItem"];
                self.ra_caps = RaCaps {
                    snippets: item["snippetSupport"].as_bool().unwrap_or(false),
                    label_details: item["labelDetailsSupport"].as_bool().unwrap_or(false),
                    insert_replace: item["insertReplaceSupport"].as_bool().unwrap_or(false),
                };
                send(&json!({"jsonrpc":"2.0","id":id,"result":{
                    "capabilities": {
                        "textDocumentSync": {"openClose": true, "change": 1, "save": {"includeText": false}},
                        "definitionProvider": true,
                        "hoverProvider": true,
                        "completionProvider": {"triggerCharacters": [".", ":"], "resolveProvider": true},
                        "documentOnTypeFormattingProvider": {"firstTriggerCharacter": "\n"},
                        "documentSymbolProvider": true
                    },
                    "serverInfo": {"name": "lrs", "version": env!("CARGO_PKG_VERSION")}
                }}));
            }
            ("shutdown", Some(id)) => {
                self.shutdown = true;
                send(&json!({"jsonrpc":"2.0","id":id,"result":null}));
            }
            ("exit", _) => return false,
            ("textDocument/didOpen", _) => {
                if let Some(path) = params["textDocument"]["uri"].as_str().and_then(uri_to_path) {
                    let text = params["textDocument"]["text"].as_str().unwrap_or("").to_string();
                    let path = normalize(&path);
                    self.docs.insert(path.clone(), text);
                    self.invalidate();
                    // start rust-analyzer and its indexing now, so the first completion is not kept waiting
                    if let Some(a) = self.analysis_for(&path) {
                        self.sync_ra(&path, &a);
                    }
                    self.publish_all();
                }
            }
            ("textDocument/didChange", _) => {
                if let Some(path) = params["textDocument"]["uri"].as_str().and_then(uri_to_path) {
                    if let Some(text) = params["contentChanges"].as_array().and_then(|c| c.last()).and_then(|c| c["text"].as_str()) {
                        self.docs.insert(normalize(&path), text.to_string());
                        self.invalidate();
                        // rustc positions are stale after an edit
                        self.cargo_diags.clear();
                        self.publish_all();
                    }
                }
            }
            ("textDocument/didSave", _) => {
                if let Some(path) = params["textDocument"]["uri"].as_str().and_then(uri_to_path) {
                    self.start_cargo(&path);
                }
            }
            ("textDocument/didClose", _) => {
                if let Some(path) = params["textDocument"]["uri"].as_str().and_then(uri_to_path) {
                    let path = normalize(&path);
                    self.docs.remove(&path);
                    self.doc_entry.remove(&path);
                    self.invalidate();
                    self.publish_all();
                }
            }
            ("textDocument/completion", Some(id)) => self.complete(id, params),
            ("textDocument/onTypeFormatting", Some(id)) => respond(id, self.on_type(params).unwrap_or(json!([]))),
            ("completionItem/resolve", Some(id)) => self.resolve(id, params),
            (m @ ("textDocument/definition" | "textDocument/hover" | "textDocument/documentSymbol"), Some(id)) => {
                let result = self.query(m, params).unwrap_or(Value::Null);
                send(&json!({"jsonrpc":"2.0","id":id,"result":result}));
            }
            (_, Some(id)) => {
                send(&json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":format!("method not found: {}", method)}}));
            }
            _ => {}
        }
        true
    }

    /// Enter between `{` and `}`: opens an indented middle line, leaving `}` on its own line.
    fn on_type(&self, params: &Value) -> Option<Value> {
        let none = json!([]);
        if params["ch"].as_str()? != "\n" {
            return Some(none);
        }
        let path = normalize(&uri_to_path(params["textDocument"]["uri"].as_str()?)?);
        let text = self.docs.get(&path)?;
        let line = params["position"]["line"].as_u64()? as usize;
        let col = params["position"]["character"].as_u64()? as usize;
        Some(braces_enter_edits(text.as_str(), line, col).unwrap_or(none))
    }

    /// Completion: `.lrs` namespace paths come from the namespace tree; everywhere else
    /// that maps into the generated Rust, rust-analyzer answers (asynchronously).
    fn complete(&mut self, id: Value, params: &Value) {
        let empty = json!({"isIncomplete": false, "items": []});
        // numbering the requests lets older ones that are still waiting give way to this one
        let seq = self.store.next_seq();
        let Some(path) = params["textDocument"]["uri"].as_str().and_then(uri_to_path).map(|p| normalize(&p)) else {
            return respond(id, empty);
        };
        let Some(a) = self.analysis_for(&path) else { return respond(id, empty) };
        let Some(fid) = a.project.file_index(&path) else { return respond(id, empty) };
        let off = {
            let f = &a.project.files[fid];
            let p = &params["position"];
            f.lines.offset(&f.text, p["line"].as_u64().unwrap_or(0) as u32, p["character"].as_u64().unwrap_or(0) as u32)
        };
        let lrs_items = features::completion(&a, fid, off);
        if !lrs_items.is_empty() {
            return respond(id, json!({"isIncomplete": false, "items": lrs_items}));
        }
        let Some(gen_off) = rust_completion::lrs_to_gen(&a, fid, off) else { return respond(id, empty) };
        let Some((cfg, gen_path)) = self.sync_ra(&path, &a) else { return respond(id, empty) };
        let Some(sess) = self.ra_sessions.get(&cfg.entry) else { return respond(id, empty) };
        rust_completion::completion_job(&sess.ra, a.clone(), fid, gen_off, gen_path, cfg.entry.clone(), self.store.clone(), seq, id);
    }

    /// Make sure rust-analyzer has the generated crate of `a` (the analysis of the `.lrs` file
    /// at `path`). Returns the project config and the generated file. Loose files have none.
    fn sync_ra(&mut self, path: &Path, a: &Arc<Analysis>) -> Option<(Config, PathBuf)> {
        let cfg = match path.parent().and_then(find_config_dir).map(|cd| find_project(&cd)) {
            Some(Ok(c)) => c,
            Some(Err(e)) => {
                debug_log(&format!("project for {} failed: {e}", path.display()));
                return None;
            }
            None => {
                debug_log(&format!("no Cargo.toml [lrs] / lrs.toml above {}", path.display()));
                return None;
            }
        };
        let gen_path = gen_dir(&cfg).join("src").join(if cfg.is_lib { "lib.rs" } else { "main.rs" });
        let synced = self.ra_sessions.get(&cfg.entry).and_then(|s| s.synced.as_ref()).is_some_and(|x| Arc::ptr_eq(x, a));
        if !synced {
            // the text goes to rust-analyzer in memory; the disk copy only has to exist once per session
            // (every disk write would make it reload the file)
            if !self.ra_sessions.contains_key(&cfg.entry) {
                if let Err(e) = write_gen(&cfg, &a.emitted.code) {
                    debug_log(&format!("writing the generated crate failed: {e}"));
                    return None;
                }
            }
            let root = gen_dir(&cfg);
            let caps = self.ra_caps;
            let sess = self
                .ra_sessions
                .entry(cfg.entry.clone())
                .or_insert_with(|| RaSession { ra: Ra::start(root, caps), synced: None });
            rust_completion::sync_job(&sess.ra, gen_path.clone(), a.emitted.code.clone());
            sess.synced = Some(a.clone());
        }
        Some((cfg, gen_path))
    }

    fn resolve(&mut self, id: Value, params: &Value) {
        let key = &params["data"]["lrs_ra"];
        let (seq, idx) = (key["r"].as_u64(), key["i"].as_u64().map(|i| i as usize));
        let found = seq.and_then(|s| self.store.get(s)).zip(seq).zip(idx);
        match found.and_then(|((resp, seq), idx)| self.ra_sessions.get(&resp.entry).map(|s| (s, resp, seq, idx))) {
            Some((sess, resp, seq, idx)) => {
                rust_completion::resolve_job(&sess.ra, resp, idx, seq, self.store.clone(), params.clone(), id)
            }
            None => respond(id, params.clone()),
        }
    }

    fn query(&mut self, method: &str, params: &Value) -> Option<Value> {
        let path = normalize(&uri_to_path(params["textDocument"]["uri"].as_str()?)?);
        let a = self.analysis_for(&path)?;
        let fid = a.project.file_index(&path)?;
        let f = &a.project.files[fid];
        let off = || {
            let p = &params["position"];
            f.lines.offset(&f.text, p["line"].as_u64().unwrap_or(0) as u32, p["character"].as_u64().unwrap_or(0) as u32)
        };
        Some(match method {
            "textDocument/definition" => Value::Array(features::definition(&a, fid, off())),
            "textDocument/hover" => features::hover(&a, fid, off())?,
            _ => Value::Array(features::symbols(&a, fid)),
        })
    }
}

/// Run the server on stdio; returns the process exit code.
pub fn run() -> i32 {
    let (tx, rx): (Sender<Event>, Receiver<Event>) = channel();
    let rtx = tx.clone();
    std::thread::spawn(move || reader_thread(rtx));
    let mut s = Server {
        tx,
        docs: Overlay::new(),
        cache: HashMap::new(),
        doc_entry: HashMap::new(),
        cargo_diags: HashMap::new(),
        cargo_running: HashSet::new(),
        cargo_rerun: HashSet::new(),
        published: HashSet::new(),
        shutdown: false,
        ra_sessions: HashMap::new(),
        ra_caps: RaCaps::default(),
        store: Arc::new(ItemStore::default()),
    };
    while let Ok(ev) = rx.recv() {
        match ev {
            Event::Msg(m) => {
                if !s.handle(m) {
                    return if s.shutdown { 0 } else { 1 };
                }
            }
            Event::CargoDone { entry, analysis, result } => s.on_cargo_done(entry, analysis, result),
            Event::Eof => return if s.shutdown { 0 } else { 1 },
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_roundtrip() {
        let p = Path::new("/tmp/my project/a b.lrs");
        let u = path_to_uri(p);
        assert_eq!(u, "file:///tmp/my%20project/a%20b.lrs");
        assert_eq!(uri_to_path(&u).unwrap(), p);
    }

    #[test]
    fn enter_between_braces_indents_middle_line() {
        // client already put the `}` line at column 0 with the cursor there
        let text = "pub fn main() {\n}\n";
        let edits = braces_enter_edits(text, 1, 0).unwrap();
        assert_eq!(edits[0]["newText"], "    ");
        assert_eq!(edits[1]["newText"], "\n");
    }

    #[test]
    fn enter_between_braces_uses_tabs_when_file_uses_tabs() {
        let text = "fn f() {\n\tlet x = 1;\n}\n\nfn g() {\n}\n";
        let edits = braces_enter_edits(text, 5, 0).unwrap();
        assert_eq!(edits[0]["newText"], "\t");
        assert_eq!(edits[1]["newText"], "\n");
    }

    #[test]
    fn enter_between_braces_keeps_nested_indent() {
        let text = "impl A {\n    fn f() {\n    }\n}\n";
        let edits = braces_enter_edits(text, 2, 4).unwrap();
        assert_eq!(edits[0]["newText"], "        ");
        assert_eq!(edits[1]["newText"], "\n    ");
    }

    #[test]
    fn enter_between_braces_ignores_other_lines() {
        let text = "let a = 1;\n}\n";
        assert!(braces_enter_edits(text, 1, 0).is_none());
    }
}
