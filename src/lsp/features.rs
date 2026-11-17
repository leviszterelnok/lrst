//! Editor features computed from an `Analysis`: go-to-definition, hover,
//! completion and document symbols.

use super::path_to_uri;
use crate::driver::Analysis;
use crate::lexer::Kind;
use crate::model::*;
use crate::resolve::*;
use serde_json::{json, Value};

pub fn lsp_range(f: &SourceFile, s: usize, e: usize) -> Value {
    let (l1, c1) = f.lines.position(&f.text, s);
    let (l2, c2) = f.lines.position(&f.text, e);
    json!({"start": {"line": l1, "character": c1}, "end": {"line": l2, "character": c2}})
}

fn location(a: &Analysis, fid: usize, s: usize, e: usize) -> Value {
    let f = &a.project.files[fid];
    json!({"uri": path_to_uri(&f.path), "range": lsp_range(f, s, e)})
}

/// The token under `off`, preferring identifiers at token boundaries.
fn token_at(f: &SourceFile, off: usize) -> Option<usize> {
    let i = f.tokens.partition_point(|t| t.start <= off);
    let mut best = None;
    for k in [i.checked_sub(1), i.checked_sub(2)].into_iter().flatten() {
        let t = &f.tokens[k];
        if t.start <= off && off <= t.end {
            if t.kind == Kind::Ident {
                return Some(k);
            }
            best.get_or_insert(k);
        }
    }
    best
}

fn in_use_stmt(f: &SourceFile, from: usize) -> bool {
    let mut k = from;
    while k > 0 {
        k -= 1;
        match f.tok(k) {
            ";" | "{" | "}" => return false,
            "use" => return true,
            _ => {}
        }
    }
    false
}

/// A cursor on a segment of `namespace A.B.C {`.
fn ns_header_at(a: &Analysis, fid: usize, off: usize) -> Option<usize> {
    let p = &a.project;
    let f = &p.files[fid];
    for (id, ns) in p.nss.iter().enumerate() {
        for d in &ns.decls {
            if d.file != fid || off < d.name_start || off > d.name_end {
                continue;
            }
            let idents: Vec<(usize, usize)> = f
                .tokens
                .iter()
                .filter(|t| t.kind == Kind::Ident && t.start >= d.name_start && t.end <= d.name_end)
                .map(|t| (t.start, t.end))
                .collect();
            let k = idents.iter().position(|&(s, e)| s <= off && off <= e)?;
            let mut path = p.ns_path(id);
            let drop = idents.len() - 1 - k;
            if drop > path.len() {
                return None;
            }
            path.truncate(path.len() - drop);
            return ns_by_path(p, &path);
        }
    }
    None
}

fn resolve_at(a: &Analysis, fid: usize, off: usize) -> Option<Found> {
    if let Some(ns) = ns_header_at(a, fid, off) {
        return Some(Found::Ns(ns));
    }
    let f = &a.project.files[fid];
    let ti = token_at(f, off)?;
    let (chain, which) = path_containing(&a.project, fid, ti)?;
    let in_use = in_use_stmt(f, chain[0] - 1);
    let names: Vec<&str> = chain.iter().map(|&i| f.tok(i)).collect();
    let r = resolve_path(&a.project, &names, in_use, Some(fid));
    r.found.get(which).copied().flatten()
}

pub fn definition(a: &Analysis, fid: usize, off: usize) -> Vec<Value> {
    let p = &a.project;
    for inc in &p.includes {
        if inc.file == fid && inc.start <= off && off <= inc.end {
            if let Some(t) = p.file_index(&inc.target) {
                return vec![location(a, t, 0, 0)];
            }
        }
    }
    match resolve_at(a, fid, off) {
        Some(Found::Ns(id)) => p.nss[id].decls.iter().map(|d| location(a, d.file, d.name_start, d.name_end)).collect(),
        Some(Found::Item(ns, i) | Found::Hidden(ns, i)) => {
            let it = &p.nss[ns].items[i];
            let f = &p.files[it.file];
            let t = it.name_tok.unwrap_or(it.head);
            vec![location(a, it.file, f.tokens[t].start, f.tokens[t].end)]
        }
        None => vec![],
    }
}

fn signature(f: &SourceFile, it: &Item) -> String {
    let mut end = it.head;
    while end < it.last && f.tok(end) != "{" && f.tok(end) != ";" {
        end += 1;
    }
    let last = if end > it.head { end - 1 } else { it.head };
    let slice = &f.text[f.tokens[it.head].start..f.tokens[last].end];
    let mut s = slice.split_whitespace().collect::<Vec<_>>().join(" ");
    if s.len() > 300 {
        s.truncate(300);
        s.push('…');
    }
    s
}

fn docs(f: &SourceFile, it: &Item) -> String {
    let prev_end = if it.first == 0 { 0 } else { f.tokens[it.first - 1].end };
    let trivia = &f.text[prev_end..f.tokens[it.first].start];
    trivia
        .lines()
        .filter_map(|l| l.trim().strip_prefix("///"))
        .map(|l| l.strip_prefix(' ').unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn hover(a: &Analysis, fid: usize, off: usize) -> Option<Value> {
    let p = &a.project;
    let md = match resolve_at(a, fid, off)? {
        Found::Ns(id) => {
            let n = &p.nss[id];
            let mut files: Vec<String> = n
                .decls
                .iter()
                .map(|d| {
                    let path = &p.files[d.file].path;
                    path.strip_prefix(&p.root_dir).unwrap_or(path).display().to_string()
                })
                .collect();
            files.sort();
            files.dedup();
            let name = p.ns_path(id).join(".");
            format!(
                "```lrs\nnamespace .{}\n```\n{} item(s), {} sub-namespace(s), {} mount(s)\n\nDeclared in: {}",
                name,
                n.items.len(),
                n.children.len(),
                n.mounts.len(),
                files.join(", ")
            )
        }
        Found::Item(ns, i) | Found::Hidden(ns, i) => {
            let it = &p.nss[ns].items[i];
            let f = &p.files[it.file];
            let mut s = format!("```rust\n{}\n```\n", signature(f, it));
            let d = docs(f, it);
            if !d.is_empty() {
                s.push('\n');
                s.push_str(&d);
                s.push('\n');
            }
            let mut path = p.ns_path(ns).join(".");
            if !path.is_empty() {
                path.insert(0, '.');
            }
            s.push_str(&format!("\nIn namespace `{}`", if path.is_empty() { "." } else { &path }));
            s
        }
    };
    Some(json!({"contents": {"kind": "markdown", "value": md}}))
}

fn item_kind(kind: &str) -> u32 {
    match kind {
        "fn" => 3,       // Function
        "struct" => 22,  // Struct
        "enum" => 13,    // Enum
        "union" => 22,
        "trait" => 8,    // Interface
        "const" => 21,   // Constant
        "static" => 6,   // Variable
        "type" => 7,     // Class
        "mod" => 9,      // Module
        "macro_rules" | "macro" => 3,
        _ => 6,
    }
}

pub fn completion(a: &Analysis, fid: usize, off: usize) -> Vec<Value> {
    let p = &a.project;
    let f = &p.files[fid];
    let i = f.tokens.partition_point(|t| t.start < off);
    let Some(ti) = i.checked_sub(1) else { return vec![] };
    let t = f.tokens[ti];
    let d = if t.kind == Kind::Ident && t.end >= off && ti > 0 {
        ti - 1
    } else if t.kind == Kind::Punct && t.end == off {
        ti
    } else {
        return vec![];
    };
    if d != ti && f.tokens[d].end != f.tokens[ti].start {
        return vec![];
    }
    let Some(prev) = chain_before(p, fid, d) else { return vec![] };
    let mut cur = 0;
    for &pi in &prev {
        match lookup(p, cur, f.tok(pi), Some(fid)) {
            Some(Found::Ns(id)) => cur = id,
            _ => return vec![], // an item: this is a field access, not a path
        }
    }
    visible_names(p, cur, Some(fid))
        .into_iter()
        .map(|(name, _)| {
            let (kind, detail) = match lookup(p, cur, &name, Some(fid)) {
                Some(Found::Ns(_)) => (9, "namespace".to_string()),
                Some(Found::Item(ns, idx) | Found::Hidden(ns, idx)) => {
                    let it = &p.nss[ns].items[idx];
                    (item_kind(&it.kind), it.kind.clone())
                }
                None => (6, String::new()),
            };
            json!({"label": name, "kind": kind, "detail": detail})
        })
        .collect()
}

pub fn symbols(a: &Analysis, fid: usize) -> Vec<Value> {
    let p = &a.project;
    let f = &p.files[fid];
    let mut out = vec![];
    for (id, ns) in p.nss.iter().enumerate() {
        let container = {
            let c = p.ns_path(id).join(".");
            if c.is_empty() { String::new() } else { format!(".{}", c) }
        };
        for d in ns.decls.iter().filter(|d| d.file == fid) {
            out.push(json!({
                "name": format!(".{}", p.ns_path(id).join(".")),
                "kind": 3,
                "location": {"uri": path_to_uri(&f.path), "range": lsp_range(f, d.name_start, d.name_end)},
            }));
        }
        for it in ns.items.iter().filter(|it| it.file == fid) {
            let Some(name) = &it.name else { continue };
            let t = f.tokens[it.name_tok.unwrap_or(it.head)];
            out.push(json!({
                "name": name,
                "kind": item_kind(&it.kind),
                "containerName": container,
                "location": {"uri": path_to_uri(&f.path), "range": lsp_range(f, t.start, t.end)},
            }));
        }
    }
    out
}
