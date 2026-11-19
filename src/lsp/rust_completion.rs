//! Rust completion inside `.lrs` files: the cursor is mapped into the generated
//! crate, rust-analyzer answers there, and its items (ranges included) are mapped
//! back to `.lrs`. Positions that have no exact counterpart in the generated code
//! (namespace headers, mounts, include strings) get no Rust completion.

use super::features;
use super::ra::{Conn, Ra};
use crate::driver::Analysis;
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// One rust-analyzer answer, kept so `completionItem/resolve` can be forwarded
/// (rust-analyzer returns the `use` edit of an auto-import only on resolve).
pub struct Response {
    pub entry: PathBuf,
    analysis: Arc<Analysis>,
    fid: usize,
    raw: Vec<Value>,
}

/// The latest few answers, and the number of the latest request. A request that is no
/// longer the latest is answered at once without asking rust-analyzer, so typing fast
/// never leaves a queue of stale requests in front of the one that matters.
#[derive(Default)]
pub struct ItemStore {
    responses: Mutex<VecDeque<(u64, Arc<Response>)>>,
    latest: AtomicU64,
}

impl ItemStore {
    pub fn next_seq(&self) -> u64 {
        self.latest.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn is_latest(&self, seq: u64) -> bool {
        self.latest.load(Ordering::SeqCst) == seq
    }

    fn put(&self, seq: u64, resp: Arc<Response>) {
        if let Ok(mut q) = self.responses.lock() {
            q.push_back((seq, resp));
            while q.len() > 4 {
                q.pop_front();
            }
        }
    }

    pub fn get(&self, seq: u64) -> Option<Arc<Response>> {
        self.responses.lock().ok()?.iter().find(|(s, _)| *s == seq).map(|(_, r)| r.clone())
    }
}

/// How long a completion keeps asking while rust-analyzer is still loading the workspace.
const LOAD_WAIT: Duration = Duration::from_secs(90);
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

/// Like `gen_to_lrs`, for a point between two characters (both ends of a span count).
fn gen_to_lrs_point(a: &Analysis, fid: usize, gen: usize) -> Option<usize> {
    a.emitted.map.entries.iter().find_map(|e| {
        (e.linear && e.file == fid && gen >= e.out && gen <= e.out + e.out_len).then(|| e.src + (gen - e.out).min(e.src_len))
    })
}

/// Map an edit range of the generated crate to `.lrs`. With `glue_top`, an insertion into the
/// generated glue above this file's own code (where rust-analyzer puts an auto-`use`) goes to
/// the top of the `.lrs` file, which is where a `use` line belongs.
fn map_range(a: &Analysis, fid: usize, code: &str, range: &Value, glue_top: bool) -> Option<Value> {
    let pos = |p: &Value| gen_offset(code, p["line"].as_u64()? as u32, p["character"].as_u64()? as u32);
    let (s, e) = (pos(&range["start"])?, pos(&range["end"])?);
    let file = &a.project.files[fid];
    if let (Some(ls), Some(le)) = (gen_to_lrs(a, fid, s, false), gen_to_lrs(a, fid, e, true)) {
        if ls <= le {
            return Some(features::lsp_range(file, ls, le));
        }
    }
    if s != e {
        return None;
    }
    let first_own = || a.emitted.map.entries.iter().filter(|x| x.linear && x.file == fid).map(|x| x.out).min();
    let at = gen_to_lrs_point(a, fid, s).or_else(|| if glue_top && first_own().is_some_and(|f| s <= f) { Some(0) } else { None })?;
    Some(features::lsp_range(file, at, at))
}

/// The edits of `additionalTextEdits`, mapped to `.lrs`; those without a place there are dropped.
fn map_additional(a: &Analysis, fid: usize, code: &str, edits: &Value) -> Vec<Value> {
    edits
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|e| Some(json!({"range": map_range(a, fid, code, &e["range"], true)?, "newText": e["newText"]})))
                .collect()
        })
        .unwrap_or_default()
}

/// Convert one rust-analyzer item into a `.lrs` item. Edits that cannot be mapped are dropped.
fn map_item(a: &Analysis, fid: usize, code: &str, raw: &Value, seq: u64, idx: usize) -> Value {
    let mut out = json!({});
    for k in COPIED_FIELDS {
        if let Some(v) = raw.get(*k) {
            out[*k] = v.clone();
        }
    }
    if let Some(te) = raw.get("textEdit") {
        let mapped = match te.get("range") {
            Some(r) => map_range(a, fid, code, r, false).map(|r| json!({"range": r, "newText": te["newText"]})),
            None => match (map_range(a, fid, code, &te["insert"], false), map_range(a, fid, code, &te["replace"], false)) {
                (Some(i), Some(r)) => Some(json!({"insert": i, "replace": r, "newText": te["newText"]})),
                _ => None,
            },
        };
        if let Some(m) = mapped {
            out["textEdit"] = m;
        }
    }
    let extra = map_additional(a, fid, code, &raw["additionalTextEdits"]);
    if !extra.is_empty() {
        out["additionalTextEdits"] = Value::Array(extra);
    }
    out["data"] = json!({"lrs_ra": {"r": seq, "i": idx}});
    out
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
    seq: u64,
    id: Value,
) {
    ra.run(Box::new(move |conn: Option<&mut Conn>| {
        // the client asks again when it sees `isIncomplete`
        let pending = json!({"isIncomplete": true, "items": []});
        let result = 'done: {
            let Some(c) = conn else { break 'done json!({"isIncomplete": false, "items": []}) };
            let code = a.emitted.code.as_str();
            let (line, col) = gen_position(code, gen_off);
            let params = json!({
                "textDocument": {"uri": super::path_to_uri(&gen_path)},
                "position": {"line": line, "character": col}
            });
            // rust-analyzer answers `null` while it is still loading the workspace: keep asking,
            // but give up as soon as a newer request exists
            let deadline = Instant::now() + LOAD_WAIT;
            let reply = loop {
                if !store.is_latest(seq) {
                    break 'done pending;
                }
                if let Some(v) = c.request("textDocument/completion", params.clone()) {
                    break v;
                }
                if c.is_dead() || Instant::now() >= deadline {
                    break 'done pending;
                }
                std::thread::sleep(Duration::from_millis(250));
            };
            // LRS_LOG=<file>: keep the raw rust-analyzer answers for debugging
            if let Some(log) = std::env::var_os("LRS_LOG") {
                use std::io::Write;
                if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(log) {
                    let _ = writeln!(f, "{reply}");
                }
            }
            let (raw_items, incomplete) = match &reply {
                Value::Array(items) => (items.clone(), false),
                _ => (reply["items"].as_array().cloned().unwrap_or_default(), reply["isIncomplete"].as_bool().unwrap_or(false)),
            };
            // the per-file modules of the generated crate are not user names
            let raw: Vec<Value> =
                raw_items.into_iter().filter(|r| !r["label"].as_str().unwrap_or("").starts_with("__lrs")).collect();
            let items: Vec<Value> = raw.iter().enumerate().map(|(i, r)| map_item(&a, fid, code, r, seq, i)).collect();
            store.put(seq, Arc::new(Response { entry, analysis: a.clone(), fid, raw }));
            json!({"isIncomplete": incomplete, "items": items})
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
pub fn resolve_job(ra: &Ra, resp: Arc<Response>, idx: usize, seq: u64, store: Arc<ItemStore>, params: Value, id: Value) {
    ra.run(Box::new(move |conn: Option<&mut Conn>| {
        let mut out = params;
        // a newer completion changed what rust-analyzer sees: the old item can no longer be resolved
        if store.is_latest(seq) {
            if let (Some(c), Some(raw)) = (conn, resp.raw.get(idx)) {
                if let Some(resolved) = c.request("completionItem/resolve", raw.clone()) {
                    for k in ["documentation", "detail"] {
                        if let Some(v) = resolved.get(k) {
                            out[k] = v.clone();
                        }
                    }
                    // the `use` line of an auto-import
                    let extra = map_additional(&resp.analysis, resp.fid, &resp.analysis.emitted.code, &resolved["additionalTextEdits"]);
                    if !extra.is_empty() {
                        out["additionalTextEdits"] = Value::Array(extra);
                    }
                }
            }
        }
        super::send(&json!({"jsonrpc": "2.0", "id": id, "result": out}));
    }));
}
