//! Rust completion inside `.lrs` files: the cursor is mapped into the generated
//! crate, rust-analyzer answers there, and its items (ranges included) are mapped
//! back to `.lrs`. Positions that have no exact counterpart in the generated code
//! (namespace headers, mounts, include strings) get no Rust completion.

use super::features;
use super::ra::{Conn, Ra};
use crate::driver::Analysis;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Raw rust-analyzer items, kept so `completionItem/resolve` can be forwarded.
#[derive(Default)]
pub struct ItemStore {
    items: Mutex<HashMap<u64, (PathBuf, Value)>>,
    next: AtomicU64,
}

const STORE_CAP: usize = 5000;
const RETRY_FOR: std::time::Duration = std::time::Duration::from_secs(5);
const COPIED_FIELDS: &[&str] = &[
    "label", "labelDetails", "kind", "tags", "detail", "documentation", "deprecated", "preselect", "sortText",
    "filterText", "insertText", "insertTextFormat", "insertTextMode", "commitCharacters",
];

/// Byte offset in the generated code that a `.lrs` byte offset maps to, if the
/// byte is copied verbatim.
pub fn lrs_to_gen(a: &Analysis, fid: usize, off: usize) -> Option<usize> {
    a.emitted.map.entries.iter().find_map(|e| {
        (e.linear && e.file == fid && off >= e.src && off <= e.src + e.src_len).then(|| e.out + (off - e.src))
    })
}

/// Map a generated offset back to `.lrs`. `end` selects the exclusive-end rule.
fn gen_to_lrs(a: &Analysis, fid: usize, gen: usize, end: bool) -> Option<usize> {
    a.emitted.map.entries.iter().find_map(|e| {
        if !e.linear || e.file != fid {
            return None;
        }
        let (lo, hi) = (e.out, e.out + e.out_len);
        let inside = if end { gen > lo && gen <= hi } else { gen >= lo && gen < hi };
        inside.then(|| e.src + (gen - lo).min(e.src_len))
    })
}

/// LSP (line, utf-16 column) of a byte offset in `code`.
fn gen_position(code: &str, off: usize) -> (u32, u32) {
    let before = &code[..off];
    let line = before.matches('\n').count() as u32;
    let col = before.rsplit('\n').next().unwrap_or("").encode_utf16().count() as u32;
    (line, col)
}

/// Byte offset of an LSP (line, utf-16 column) position in `code`.
fn gen_offset(code: &str, line: u32, col: u32) -> Option<usize> {
    let mut start = 0;
    for _ in 0..line {
        start += code[start..].find('\n')? + 1;
    }
    let rest = &code[start..];
    let mut units = 0u32;
    for (i, ch) in rest.char_indices() {
        if ch == '\n' || units >= col {
            return Some(start + i);
        }
        units += ch.len_utf16() as u32;
    }
    Some(start + rest.len())
}

fn map_range(a: &Analysis, fid: usize, code: &str, range: &Value) -> Option<Value> {
    let pos = |p: &Value| gen_offset(code, p["line"].as_u64()? as u32, p["character"].as_u64()? as u32);
    let s = pos(&range["start"])?;
    let e = pos(&range["end"])?;
    let ls = gen_to_lrs(a, fid, s, false)?;
    let le = gen_to_lrs(a, fid, e, true)?;
    if le < ls {
        return None;
    }
    Some(features::lsp_range(&a.project.files[fid], ls, le))
}

/// An edit that lands in the generated glue above this file's own code (rust-analyzer's
/// auto-`use` goes there) is placed at the top of the `.lrs` file, where a `use` line belongs.
fn glue_top(a: &Analysis, fid: usize, code: &str, range: &Value) -> Option<Value> {
    let s = gen_offset(code, range["start"]["line"].as_u64()? as u32, range["start"]["character"].as_u64()? as u32)?;
    let first_own = a.emitted.map.entries.iter().filter(|e| e.linear && e.file == fid).map(|e| e.out).min()?;
    (s < first_own).then(|| features::lsp_range(&a.project.files[fid], 0, 0))
}

/// Convert one rust-analyzer item into a `.lrs` item. Edits that cannot be mapped are dropped.
fn map_item(a: &Analysis, fid: usize, code: &str, raw: &Value, key: u64) -> Value {
    let mut out = json!({});
    for k in COPIED_FIELDS {
        if let Some(v) = raw.get(*k) {
            out[*k] = v.clone();
        }
    }
    if let Some(te) = raw.get("textEdit") {
        let mapped = match te.get("range") {
            Some(r) => map_range(a, fid, code, r).map(|r| json!({"range": r, "newText": te["newText"]})),
            None => match (map_range(a, fid, code, &te["insert"]), map_range(a, fid, code, &te["replace"])) {
                (Some(i), Some(r)) => Some(json!({"insert": i, "replace": r, "newText": te["newText"]})),
                _ => None,
            },
        };
        if let Some(m) = mapped {
            out["textEdit"] = m;
        }
    }
    if let Some(extra) = raw.get("additionalTextEdits").and_then(Value::as_array) {
        let edits: Vec<Value> = extra
            .iter()
            .filter_map(|e| {
                let range = map_range(a, fid, code, &e["range"]).or_else(|| glue_top(a, fid, code, &e["range"]))?;
                Some(json!({"range": range, "newText": e["newText"]}))
            })
            .collect();
        if !edits.is_empty() {
            out["additionalTextEdits"] = Value::Array(edits);
        }
    }
    out["data"] = json!({"lrs_ra": key});
    out
}

impl ItemStore {
    fn put(&self, entry: &PathBuf, raw: &Value) -> u64 {
        let key = self.next.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut m) = self.items.lock() {
            if m.len() >= STORE_CAP {
                m.clear();
            }
            m.insert(key, (entry.clone(), raw.clone()));
        }
        key
    }

    pub fn get(&self, key: u64) -> Option<(PathBuf, Value)> {
        self.items.lock().ok()?.get(&key).cloned()
    }
}

/// Queue a job that makes rust-analyzer see the generated code (`path`, `code`).
pub fn sync_job(ra: &Ra, path: PathBuf, code: String) {
    ra.run(Box::new(move |conn| {
        if let Some(c) = conn {
            c.sync(&path, &code);
        }
    }));
}

/// Queue the completion request and send its mapped answer as the response to `id`.
#[allow(clippy::too_many_arguments)]
pub fn completion_job(
    ra: &Ra,
    a: Arc<Analysis>,
    fid: usize,
    gen_off: usize,
    gen_path: PathBuf,
    entry: PathBuf,
    store: Arc<ItemStore>,
    id: Value,
) {
    ra.run(Box::new(move |conn: Option<&mut Conn>| {
        let empty = json!({"isIncomplete": false, "items": []});
        let result = match conn {
            None => empty,
            Some(c) => {
                let code = a.emitted.code.as_str();
                let (line, col) = gen_position(code, gen_off);
                let uri = super::path_to_uri(&gen_path);
                let params = json!({"textDocument": {"uri": uri}, "position": {"line": line, "character": col}});
                c.wait_warm();
                let mut reply = c.request("textDocument/completion", params.clone());
                // still busy (no answer yet): retry briefly, then ask the client to come back
                let started = std::time::Instant::now();
                while reply.is_none() && started.elapsed() < RETRY_FOR {
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    reply = c.request("textDocument/completion", params.clone());
                }
                // LRS_LOG=<file>: keep the raw rust-analyzer answers for debugging
                if let Some(log) = std::env::var_os("LRS_LOG") {
                    use std::io::Write;
                    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(log) {
                        let _ = writeln!(f, "{}", reply.as_ref().map_or("null".to_string(), |v| v.to_string()));
                    }
                }
                match reply {
                    None => json!({"isIncomplete": true, "items": []}),
                    Some(v) => {
                        let (raw_items, incomplete) = match &v {
                            Value::Array(items) => (items.clone(), false),
                            _ => (
                                v["items"].as_array().cloned().unwrap_or_default(),
                                v["isIncomplete"].as_bool().unwrap_or(false),
                            ),
                        };
                        let items: Vec<Value> = raw_items
                            .iter()
                            // the per-file modules of the generated crate are not user names
                            .filter(|raw| !raw["label"].as_str().unwrap_or("").starts_with("__lrs"))
                            .map(|raw| {
                                let key = store.put(&entry, raw);
                                map_item(&a, fid, code, raw, key)
                            })
                            .collect();
                        json!({"isIncomplete": incomplete, "items": items})
                    }
                }
            }
        };
        super::send(&json!({"jsonrpc": "2.0", "id": id, "result": result}));
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_positions_round_trip() {
        let code = "let a = \"é😀\";\nfn f() {}\n";
        let off = code.find("fn").unwrap();
        assert_eq!(gen_position(code, off), (1, 0));
        let emoji = code.find('😀').unwrap() + '😀'.len_utf8();
        // 9 ASCII units, 'é' (1), emoji (2 surrogate units)
        let (line, col) = gen_position(code, emoji);
        assert_eq!((line, col), (0, 12));
        assert_eq!(gen_offset(code, line, col), Some(emoji));
        assert_eq!(gen_offset(code, 1, 7), Some(code.find("{}").unwrap()));
    }
}

/// Queue `completionItem/resolve` for a raw rust-analyzer item and send the merged item.
pub fn resolve_job(ra: &Ra, raw: Value, params: Value, id: Value) {
    ra.run(Box::new(move |conn: Option<&mut Conn>| {
        let mut out = params;
        if let Some(c) = conn {
            if let Some(resolved) = c.request("completionItem/resolve", raw) {
                for k in ["documentation", "detail"] {
                    if let Some(v) = resolved.get(k) {
                        out[k] = v.clone();
                    }
                }
            }
        }
        super::send(&json!({"jsonrpc": "2.0", "id": id, "result": out}));
    }));
}
