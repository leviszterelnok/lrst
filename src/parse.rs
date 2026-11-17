//! Loading files, following `include`s, and splitting each file into
//! namespace blocks, mount lists and (opaque) Rust items.

use crate::lexer::{lex, Kind, Token};
use crate::model::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type Overlay = HashMap<PathBuf, String>;

/// Canonicalize if the file exists, otherwise normalize lexically.
pub fn normalize(path: &Path) -> PathBuf {
    if let Ok(c) = std::fs::canonicalize(path) {
        return c;
    }
    let mut out = PathBuf::new();
    for comp in path.components() {
        use std::path::Component::*;
        match comp {
            CurDir => {}
            ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn read_text(path: &Path, overlay: &Overlay) -> std::io::Result<String> {
    if let Some(t) = overlay.get(path) {
        return Ok(t.clone());
    }
    std::fs::read_to_string(path)
}

/// Load the entry file and everything it includes.
pub fn load(entry: &Path, root: &Path, overlay: &Overlay) -> Result<Project, String> {
    let key = normalize(entry);
    let mut p = Project::new(normalize(root), key.clone());
    let text = read_text(&key, overlay).map_err(|e| format!("cannot read {}: {}", key.display(), e))?;
    add_file(&mut p, key, text, overlay);
    Ok(p)
}

fn compute_matching(src: &str, toks: &[Token]) -> (Vec<usize>, Vec<(usize, usize, String)>) {
    let mut m = vec![usize::MAX; toks.len()];
    let mut errs = vec![];
    let mut stack: Vec<usize> = vec![];
    for (i, t) in toks.iter().enumerate() {
        if t.kind != Kind::Punct || t.end - t.start != 1 {
            continue;
        }
        match &src[t.start..t.end] {
            "(" | "[" | "{" => stack.push(i),
            c @ (")" | "]" | "}") => {
                let want = match c {
                    ")" => "(",
                    "]" => "[",
                    _ => "{",
                };
                match stack.last() {
                    Some(&o) if &src[toks[o].start..toks[o].end] == want => {
                        stack.pop();
                        m[o] = i;
                        m[i] = o;
                    }
                    _ => errs.push((t.start, t.end, format!("unmatched closing delimiter `{}`", c))),
                }
            }
            _ => {}
        }
    }
    for o in stack {
        errs.push((toks[o].start, toks[o].end, "unclosed delimiter".to_string()));
    }
    (m, errs)
}

pub(crate) fn add_file(p: &mut Project, path: PathBuf, text: String, overlay: &Overlay) -> usize {
    let lexed = lex(&text);
    let tokens = lexed.tokens;
    let (matching, merrs) = compute_matching(&text, &tokens);
    let fid = p.files.len();
    p.files.push(SourceFile {
        path,
        text: text.clone(),
        tokens: tokens.clone(),
        lines: LineIndex::new(&text),
        matching: matching.clone(),
    });
    for (s, e, m) in lexed.errors.into_iter().chain(merrs) {
        p.diags.push(Diag::error(fid, s, e, m));
    }
    let cx = Cx { text: &text, toks: &tokens, m: &matching, fid };
    parse_body(p, &cx, 0, tokens.len(), 0, true, overlay);
    fid
}

struct Cx<'a> {
    text: &'a str,
    toks: &'a [Token],
    m: &'a [usize],
    fid: usize,
}

impl<'a> Cx<'a> {
    fn t(&self, i: usize) -> &'a str {
        match self.toks.get(i) {
            Some(t) => &self.text[t.start..t.end],
            None => "",
        }
    }
    fn kind(&self, i: usize) -> Option<Kind> {
        self.toks.get(i).map(|t| t.kind)
    }
    fn is_ident(&self, i: usize) -> bool {
        self.kind(i) == Some(Kind::Ident)
    }
}

fn ensure_ns(p: &mut Project, mut cur: usize, path: &[String]) -> usize {
    for seg in path {
        let found = p.nss[cur].children.iter().find(|(n, _)| n == seg).map(|x| x.1);
        cur = match found {
            Some(id) => id,
            None => {
                let id = p.nss.len();
                p.nss.push(Ns::new(seg.clone(), Some(cur)));
                p.nss[cur].children.push((seg.clone(), id));
                id
            }
        };
    }
    cur
}

fn parse_body(p: &mut Project, cx: &Cx, mut i: usize, end: usize, ns: usize, top: bool, overlay: &Overlay) {
    while i < end {
        if cx.is_ident(i) && cx.t(i) == "namespace" {
            if let Some(next) = parse_namespace(p, cx, i, end, ns, overlay) {
                i = next;
                continue;
            }
        }
        if cx.is_ident(i) && cx.t(i) == "include" && cx.kind(i + 1) == Some(Kind::Str) {
            let (s, e) = (cx.toks[i].start, cx.toks[i + 1].end);
            let mut next = i + 2;
            if next < end && cx.t(next) == ";" {
                next += 1;
            }
            let lit = cx.t(i + 1);
            if !top {
                p.diags.push(Diag::error(
                    cx.fid,
                    s,
                    e,
                    "`include` is only allowed at the top level of a file (not inside a namespace block)",
                ));
            } else if !(lit.starts_with('"') && lit.ends_with('"') && lit.len() >= 2) {
                p.diags.push(Diag::error(cx.fid, s, e, "`include` expects a plain string literal"));
            } else {
                load_include(p, cx.fid, &lit[1..lit.len() - 1], s, e, overlay);
            }
            i = next;
            continue;
        }

        let info = scan_item(cx, i, end);
        let last = info.last.max(i + 1).min(end);
        if info.kind == "inner_attr" {
            p.nss[ns].inner_attrs.push((cx.fid, i, last));
        } else {
            p.nss[ns].items.push(Item {
                file: cx.fid,
                first: i,
                last,
                head: info.head,
                name: info.name.as_ref().map(|x| x.0.clone()),
                name_tok: info.name.map(|x| x.1),
                kind: info.kind,
                vis: info.vis,
                use_names: info.use_names,
            });
        }
        i = last;
    }
}

fn load_include(p: &mut Project, fid: usize, lit: &str, s: usize, e: usize, overlay: &Overlay) {
    let base = if let Some(rest) = lit.strip_prefix('/') {
        p.root_dir.join(rest)
    } else {
        p.files[fid].path.parent().unwrap_or(Path::new("")).join(lit)
    };
    let key = normalize(&base);
    p.includes.push(IncludeRef { file: fid, start: s, end: e, target: key.clone() });
    if p.files.iter().any(|f| f.path == key) {
        return; // already included (include-once semantics, cycles are fine)
    }
    match read_text(&key, overlay) {
        Ok(t) => {
            add_file(p, key, t, overlay);
        }
        Err(err) => p.diags.push(Diag::error(fid, s, e, format!("cannot include \"{}\": {}", lit, err))),
    }
}

fn parse_namespace(p: &mut Project, cx: &Cx, i: usize, end: usize, ns: usize, overlay: &Overlay) -> Option<usize> {
    let mut j = i + 1;
    let mut path: Vec<(String, usize, usize)> = vec![];
    while j < end && cx.is_ident(j) {
        path.push((cx.t(j).to_string(), cx.toks[j].start, cx.toks[j].end));
        j += 1;
        if j + 1 < end && cx.t(j) == "." && cx.is_ident(j + 1) {
            j += 1;
            continue;
        }
        break;
    }
    if !(j < end && cx.t(j) == "{") {
        return None;
    }
    let open = j;
    let close = cx.m[open];
    if close == usize::MAX || close >= end {
        p.diags.push(Diag::error(cx.fid, cx.toks[i].start, cx.toks[open].end, "unclosed namespace body"));
        return Some(end);
    }
    let is_mount = cx.t(open + 1) == ".";
    if path.is_empty() && !is_mount {
        p.diags.push(Diag::error(
            cx.fid,
            cx.toks[i].start,
            cx.toks[open].end,
            "`namespace {}` needs a name (`namespace Foo { .. }`) or a mount list (`namespace { .Foo }`)",
        ));
        return Some(close + 1);
    }
    let names: Vec<String> = path.iter().map(|x| x.0.clone()).collect();
    let target = ensure_ns(p, ns, &names);
    if !path.is_empty() {
        p.nss[target].decls.push(Decl {
            file: cx.fid,
            name_start: path[0].1,
            name_end: path[path.len() - 1].2,
            header_start: cx.toks[i].start,
        });
    }
    if is_mount {
        parse_mounts(p, cx, open + 1, close, target);
    } else {
        parse_body(p, cx, open + 1, close, target, false, overlay);
    }
    let mut next = close + 1;
    if next < end && cx.t(next) == ";" {
        next += 1;
    }
    Some(next)
}

fn parse_mounts(p: &mut Project, cx: &Cx, mut j: usize, close: usize, target: usize) {
    while j < close {
        if cx.t(j) == ";" || cx.t(j) == "," {
            j += 1;
            continue;
        }
        if cx.t(j) == "." && j + 1 < close && cx.is_ident(j + 1) {
            let start = cx.toks[j].start;
            let mut segs = vec![];
            let mut k = j;
            while k + 1 < close && cx.t(k) == "." && cx.is_ident(k + 1) {
                segs.push(cx.t(k + 1).to_string());
                k += 2;
            }
            p.nss[target].mounts.push(Mount { file: cx.fid, path: segs, start, end: cx.toks[k - 1].end });
            j = k;
        } else {
            p.diags.push(Diag::error(
                cx.fid,
                cx.toks[j].start,
                cx.toks[j].end,
                "expected a namespace path such as `.Foo.Bar` in a mount list",
            ));
            j += 1;
        }
    }
}

struct ItemInfo {
    last: usize,
    head: usize,
    name: Option<(String, usize)>,
    kind: String,
    vis: Vis,
    use_names: Vec<String>,
}

/// Find the extent of one Rust item starting at `start` (attributes included).
fn scan_item(cx: &Cx, start: usize, end: usize) -> ItemInfo {
    let mut j = start;
    // attributes
    while cx.t(j) == "#" {
        let mut k = j + 1;
        let inner = cx.t(k) == "!";
        if inner {
            k += 1;
        }
        if cx.t(k) == "[" && cx.m[k] != usize::MAX {
            let close = cx.m[k];
            if inner {
                return ItemInfo {
                    last: close + 1,
                    head: start,
                    name: None,
                    kind: "inner_attr".into(),
                    vis: Vis::Priv,
                    use_names: vec![],
                };
            }
            j = close + 1;
        } else {
            break;
        }
    }
    let head = j;

    let mut vis = Vis::Priv;
    if cx.t(j) == "pub" {
        vis = Vis::Pub;
        j += 1;
        if cx.t(j) == "(" && cx.m[j] != usize::MAX {
            vis = if cx.t(j + 1) == "crate" { Vis::Crate } else { Vis::Priv };
            j = cx.m[j] + 1;
        }
    }

    let mut kind_override: Option<&str> = None;
    loop {
        match cx.t(j) {
            "default" | "unsafe" | "async" | "safe" => j += 1,
            "const" if matches!(cx.t(j + 1), "fn" | "unsafe" | "async" | "extern") => j += 1,
            "extern" => {
                j += if cx.kind(j + 1) == Some(Kind::Str) { 2 } else { 1 };
                if cx.t(j) == "crate" {
                    kind_override = Some("extern crate");
                    break;
                }
            }
            _ => break,
        }
    }

    let kw = cx.t(j).to_string();
    let kind: String = match kind_override {
        Some(k) => k.to_string(),
        None => match kw.as_str() {
            "fn" | "struct" | "enum" | "union" | "trait" | "type" | "mod" | "const" | "static" | "use"
            | "impl" | "macro_rules" | "macro" => kw.clone(),
            "{" => "extern".to_string(),
            _ => "other".to_string(),
        },
    };

    let mut name: Option<(String, usize)> = None;
    match kind.as_str() {
        "fn" | "struct" | "enum" | "union" | "trait" | "type" | "mod" | "macro" | "extern crate" => {
            if cx.is_ident(j + 1) {
                name = Some((cx.t(j + 1).to_string(), j + 1));
            }
        }
        "const" | "static" => {
            let mut k = j + 1;
            if cx.t(k) == "mut" {
                k += 1;
            }
            if cx.is_ident(k) && cx.t(k) != "_" {
                name = Some((cx.t(k).to_string(), k));
            }
        }
        "macro_rules" => {
            if cx.t(j + 1) == "!" && cx.is_ident(j + 2) {
                name = Some((cx.t(j + 2).to_string(), j + 2));
            }
        }
        _ => {}
    }

    let semi_only = matches!(kind.as_str(), "use" | "const" | "static" | "type" | "extern crate");
    let mut k = j;
    let mut last = end;
    while k < end {
        match cx.t(k) {
            "(" | "[" => {
                if cx.m[k] == usize::MAX {
                    break;
                }
                k = cx.m[k] + 1;
                continue;
            }
            "{" => {
                if cx.m[k] == usize::MAX {
                    break;
                }
                if semi_only {
                    k = cx.m[k] + 1;
                    continue;
                }
                last = cx.m[k] + 1;
                break;
            }
            ";" => {
                last = k + 1;
                break;
            }
            _ => {}
        }
        k += 1;
    }
    let last = last.min(end);

    let mut use_names = vec![];
    if kind == "use" {
        let mut q = j + 1;
        while q < last {
            if cx.is_ident(q) {
                let x = cx.t(q);
                if x == "as" {
                    if cx.is_ident(q + 1) {
                        use_names.push(cx.t(q + 1).to_string());
                    }
                    q += 2;
                    continue;
                }
                let next = cx.t(q + 1);
                let ends = q + 1 >= last || matches!(next, "," | "}" | ";");
                if ends && !matches!(x, "self" | "super" | "crate" | "_") {
                    use_names.push(x.to_string());
                }
            }
            q += 1;
        }
    }

    ItemInfo { last, head, name, kind, vis, use_names }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(src: &str) -> Project {
        let mut p = Project::new(PathBuf::from("/x"), PathBuf::from("/x/main.lrs"));
        add_file(&mut p, PathBuf::from("/x/main.lrs"), src.to_string(), &Overlay::new());
        p
    }

    #[test]
    fn splits_items_and_namespaces() {
        let p = parse_str(
            "pub fn a() {}\nconst X: i32 = { 5 };\nnamespace Foo.Bar { pub struct S { x: i32 } impl S { fn f(&self) {} } }\n\
             namespace { .Foo; .Foo.Bar }\nstruct T(u8);\nuse std::{fmt, io as i};",
        );
        assert!(p.diags.is_empty(), "{:?}", p.diags);
        let root = &p.nss[0];
        let names: Vec<_> = root.items.iter().map(|i| i.name.clone().unwrap_or_default()).collect();
        assert_eq!(names, ["a", "X", "T", ""]);
        assert_eq!(root.items[3].use_names, ["fmt", "i"]);
        assert_eq!(root.mounts.len(), 2);
        let foo = root.children[0].1;
        let bar = p.nss[foo].children[0].1;
        assert_eq!(p.nss[bar].items.len(), 2);
        assert_eq!(p.ns_path(bar), ["Foo", "Bar"]);
    }

    #[test]
    fn include_not_allowed_in_namespace() {
        let p = parse_str("namespace A { include \"x.lrs\" }");
        assert!(p.diags.iter().any(|d| d.message.contains("top level")));
    }
}
