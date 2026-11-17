//! Namespace lookup, mount visibility, and recognition/resolution of `.A.B.c`
//! paths in token streams.

use crate::lexer::{Kind, Token};
use crate::model::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Found {
    Ns(usize),
    /// (namespace id, index into that namespace's `items`)
    Item(usize, usize),
    /// exists, but is private to another file
    Hidden(usize, usize),
}

pub fn ns_by_path(p: &Project, path: &[String]) -> Option<usize> {
    let mut cur = 0;
    for seg in path {
        cur = p.nss[cur].children.iter().find(|(n, _)| n == seg)?.1;
    }
    Some(cur)
}

fn item_matches(it: &Item, name: &str) -> bool {
    if it.kind == "use" {
        it.use_names.iter().any(|x| x == name)
    } else {
        it.name.as_deref() == Some(name)
    }
}

/// Look a name up in a namespace as seen from `from_file`: child namespaces,
/// then the file's own items (private or not), then public items of other
/// files, then mounts. Private items of other files are `Found::Hidden`.
pub fn lookup(p: &Project, ns: usize, name: &str, from_file: Option<usize>) -> Option<Found> {
    lookup_inner(p, ns, name, from_file, &mut vec![])
}

fn lookup_inner(p: &Project, ns: usize, name: &str, from_file: Option<usize>, seen: &mut Vec<usize>) -> Option<Found> {
    if seen.contains(&ns) {
        return None;
    }
    seen.push(ns);
    let n = &p.nss[ns];
    if let Some((_, id)) = n.children.iter().find(|(c, _)| c == name) {
        return Some(Found::Ns(*id));
    }
    if let Some(f) = from_file {
        for (i, it) in n.items.iter().enumerate() {
            if it.file == f && item_matches(it, name) {
                return Some(Found::Item(ns, i));
            }
        }
    }
    for (i, it) in n.items.iter().enumerate() {
        if it.vis != Vis::Priv && item_matches(it, name) {
            return Some(Found::Item(ns, i));
        }
    }
    for m in &n.mounts {
        if let Some(t) = ns_by_path(p, &m.path) {
            // mounts only ever import public names
            if let Some(f @ (Found::Ns(_) | Found::Item(..))) = lookup_inner(p, t, name, None, seen) {
                return Some(f);
            }
        }
    }
    for (i, it) in n.items.iter().enumerate() {
        if item_matches(it, name) {
            return Some(Found::Hidden(ns, i));
        }
    }
    None
}

/// Names of `target` visible from `from_file` (`None`: public names only,
/// which is what a mount can import), with the visibility each import gets.
pub fn visible_names(p: &Project, target: usize, from_file: Option<usize>) -> Vec<(String, Vis)> {
    let mut out = vec![];
    collect_visible(p, target, from_file, &mut vec![], &mut out);
    out
}

/// Names an item introduces into its namespace.
pub fn item_names(it: &Item) -> Vec<&str> {
    match it.kind.as_str() {
        "use" => it.use_names.iter().map(|s| s.as_str()).collect(),
        "macro_rules" | "impl" | "extern" | "other" | "extern crate" => vec![],
        _ => it.name.as_deref().into_iter().collect(),
    }
}

fn collect_visible(p: &Project, target: usize, from_file: Option<usize>, stack: &mut Vec<usize>, out: &mut Vec<(String, Vis)>) {
    if stack.contains(&target) {
        return;
    }
    stack.push(target);
    let push = |name: &str, vis: Vis, out: &mut Vec<(String, Vis)>| {
        if !out.iter().any(|(n, _)| n == name) {
            out.push((name.to_string(), vis));
        }
    };
    let n = &p.nss[target];
    for (c, _) in &n.children {
        push(c, Vis::Pub, out);
    }
    for it in &n.items {
        if it.vis == Vis::Priv && Some(it.file) != from_file {
            continue;
        }
        for name in item_names(it) {
            push(name, it.vis, out);
        }
    }
    for m in &n.mounts {
        if let Some(t) = ns_by_path(p, &m.path) {
            let mut sub = vec![];
            collect_visible(p, t, None, stack, &mut sub);
            for (name, vis) in sub {
                push(&name, vis, out);
            }
        }
    }
    stack.pop();
}

// ---------------------------------------------------------------------------
// Path recognition in token streams
// ---------------------------------------------------------------------------

/// Keywords after which an expression or type may begin (so `.` starts a path).
const KEYWORDS: &[&str] = &[
    "as", "break", "const", "dyn", "else", "enum", "extern", "fn", "for", "if", "impl", "in", "let", "loop", "match",
    "mod", "move", "mut", "pub", "ref", "return", "static", "struct", "trait", "type", "unsafe", "use", "where",
    "while", "async", "yield",
];

fn text<'a>(src: &'a str, t: &Token) -> &'a str {
    &src[t.start..t.end]
}

pub fn is_single_dot(src: &str, toks: &[Token], i: usize) -> bool {
    toks.get(i).map_or(false, |t| t.kind == Kind::Punct && text(src, t) == ".")
}

fn adjacent(toks: &[Token], a: usize, b: usize) -> bool {
    toks[a].end == toks[b].start
}

/// Decide from the previous token whether a `.` at `i` begins a namespace path
/// (as opposed to being a field/method access).
pub fn dot_prev_ok(proj: &Project, fid: usize, i: usize) -> bool {
    let f = &proj.files[fid];
    let (src, toks) = (f.text.as_str(), f.tokens.as_slice());
    if i == 0 {
        return true;
    }
    let p = &toks[i - 1];
    let pt = text(src, p);
    match p.kind {
        Kind::Ident => KEYWORDS.contains(&pt),
        Kind::Str | Kind::Char | Kind::Num => false,
        Kind::Lifetime => true,
        Kind::Punct => match pt {
            ")" | "]" | "?" => false,
            // after a block the `.` is ambiguous: see `brace_dot_is_path`
            "}" => brace_dot_is_path(proj, fid, i),
            _ => true,
        },
    }
}

/// After a `}`, the `.` at `i` is resolved by the name after it:
/// * only a namespace of that name exists -> namespace path;
/// * only a method of that name exists -> method call;
/// * both (collision) -> with 0 or 1 newline before the `.` the method wins;
///   with 2 or more newlines the namespace wins.
fn brace_dot_is_path(proj: &Project, fid: usize, i: usize) -> bool {
    let f = &proj.files[fid];
    let Some(name_tok) = f.tokens.get(i + 1) else { return false };
    if name_tok.kind != Kind::Ident || !adjacent(&f.tokens, i, i + 1) {
        return false;
    }
    let name = text(&f.text, name_tok);
    if !matches!(lookup(proj, 0, name, Some(fid)), Some(Found::Ns(_))) {
        return false;
    }
    if !declares_method(proj, name) {
        return true;
    }
    let gap = &f.text[f.tokens[i - 1].end..f.tokens[i].start];
    gap.matches('\n').count() >= 2
}

/// Whether the project declares a method (a `fn` with a `self` receiver) called `name`.
fn declares_method(proj: &Project, name: &str) -> bool {
    proj.files.iter().any(|f| {
        (0..f.tokens.len()).any(|k| {
            if f.tok(k) != "fn" || f.tok(k + 1) != name || f.tok(k + 2) != "(" {
                return false;
            }
            let mut m = k + 3;
            while matches!(f.tok(m), "&" | "mut") || f.tokens.get(m).map_or(false, |t| t.kind == Kind::Lifetime) {
                m += 1;
            }
            f.tok(m) == "self"
        })
    })
}

/// If a namespace path starts at the `.` token `i`, return the token indices of
/// its (greedy) identifier chain `.A.b.c`.
pub fn scan_path(proj: &Project, fid: usize, i: usize, limit: usize) -> Option<Vec<usize>> {
    let f = &proj.files[fid];
    let (src, toks) = (f.text.as_str(), f.tokens.as_slice());
    if !is_single_dot(src, toks, i) || !dot_prev_ok(proj, fid, i) {
        return None;
    }
    let mut segs = vec![];
    let mut k = i;
    while k + 1 < limit
        && is_single_dot(src, toks, k)
        && toks[k + 1].kind == Kind::Ident
        && adjacent(toks, k, k + 1)
        && (k == i || adjacent(toks, k - 1, k))
    {
        segs.push(k + 1);
        k += 2;
    }
    if segs.is_empty() {
        None
    } else {
        Some(segs)
    }
}

/// For a `.` at `d` that is a path start or a separator inside a chain, return
/// the identifier token indices before it. `None` if it is a field access.
pub fn chain_before(proj: &Project, fid: usize, d: usize) -> Option<Vec<usize>> {
    let f = &proj.files[fid];
    let (src, toks) = (f.text.as_str(), f.tokens.as_slice());
    let mut segs = vec![];
    let mut j = d;
    loop {
        if !is_single_dot(src, toks, j) {
            return None;
        }
        if dot_prev_ok(proj, fid, j) {
            return Some(segs);
        }
        if j >= 2
            && toks[j - 1].kind == Kind::Ident
            && adjacent(toks, j - 1, j)
            && is_single_dot(src, toks, j - 2)
            && adjacent(toks, j - 2, j - 1)
        {
            segs.insert(0, j - 1);
            j -= 2;
        } else {
            return None;
        }
    }
}

/// For the identifier token `ti`, find the whole path chain containing it:
/// (token indices of all segments, index of the segment `ti` is in).
pub fn path_containing(proj: &Project, fid: usize, ti: usize) -> Option<(Vec<usize>, usize)> {
    let f = &proj.files[fid];
    let (src, toks) = (f.text.as_str(), f.tokens.as_slice());
    if toks.get(ti)?.kind != Kind::Ident || ti == 0 || !is_single_dot(src, toks, ti - 1) || !adjacent(toks, ti - 1, ti) {
        return None;
    }
    let before = chain_before(proj, fid, ti - 1)?;
    let start_dot = match before.first() {
        Some(&f) => f - 1,
        None => ti - 1,
    };
    let all = scan_path(proj, fid, start_dot, toks.len())?;
    Some((all, before.len()))
}

// ---------------------------------------------------------------------------
// Resolution
// ---------------------------------------------------------------------------

pub struct Resolution {
    /// number of leading segments that belong to the path (the rest are field accesses)
    pub consumed: usize,
    pub found: Vec<Option<Found>>,
    /// (segment index, message, severity)
    pub problem: Option<(usize, String, Severity)>,
}

pub fn resolve_path(p: &Project, names: &[&str], use_mode: bool, from_file: Option<usize>) -> Resolution {
    let mut cur = 0;
    let mut found = vec![];
    let mut problem = None;
    let mut consumed = names.len();
    for (idx, s) in names.iter().enumerate() {
        let rest = names.len() - idx - 1;
        let stop = |found: &mut Vec<Option<Found>>, consumed: &mut usize, f: Option<Found>| {
            found.push(f);
            if use_mode {
                // in `use` paths everything after an item (enum variants, ...) is part of the path
                for _ in 0..rest {
                    found.push(None);
                }
            } else {
                *consumed = idx + 1;
            }
        };
        match lookup(p, cur, s, from_file) {
            Some(Found::Ns(id)) => {
                found.push(Some(Found::Ns(id)));
                cur = id;
            }
            Some(f @ Found::Item(..)) => {
                stop(&mut found, &mut consumed, Some(f));
                break;
            }
            Some(Found::Hidden(n, i)) => {
                let file = &p.files[p.nss[n].items[i].file].path;
                let fname = file.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                problem = Some((
                    idx,
                    format!("`{}` is private to {} (mark it `pub` to use it from other files)", s, fname),
                    Severity::Error,
                ));
                stop(&mut found, &mut consumed, None);
                break;
            }
            None => {
                if idx == 0 {
                    problem = Some((0, format!("cannot find `{}` in the root namespace", s), Severity::Error));
                    for _ in 0..names.len() {
                        found.push(None);
                    }
                    break;
                }
                let ns_name = p.ns_path(cur).join(".");
                problem = Some((idx, format!("cannot find `{}` in namespace `.{}`", s, ns_name), Severity::Warning));
                stop(&mut found, &mut consumed, None);
                break;
            }
        }
    }
    Resolution { consumed, found, problem }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{add_file, Overlay};
    use std::path::PathBuf;

    fn path_at(src: &str, nth_dot: usize) -> Option<Vec<String>> {
        let mut proj = Project::new(PathBuf::from("/x"), PathBuf::from("/x/main.lrs"));
        add_file(&mut proj, PathBuf::from("/x/main.lrs"), src.to_string(), &Overlay::new());
        let toks = &proj.files[0].tokens;
        let dots: Vec<usize> = (0..toks.len()).filter(|&i| is_single_dot(src, toks, i)).collect();
        let d = dots[nth_dot];
        scan_path(&proj, 0, d, toks.len()).map(|v| v.iter().map(|&i| text(src, &toks[i]).to_string()).collect())
    }

    #[test]
    fn recognizes_paths() {
        assert_eq!(path_at("let x = .Foo.Bar.f(1);", 0).unwrap(), ["Foo", "Bar", "f"]);
        assert!(path_at("let x = a.b.c;", 0).is_none());
        assert!(path_at("let x = f().g;", 0).is_none());
        assert_eq!(path_at("fn f() -> .Foo.T {}", 0).unwrap(), ["Foo", "T"]);
        assert!(path_at("v.iter()\n.map(f)", 1).is_none());
        assert!(path_at("x.await.foo()", 1).is_none());
    }

    #[test]
    fn block_dot_rule() {
        let decl = "namespace my_math { pub fn f() -> i32 { 1 } }\n";
        let method = "impl S { fn my_math(&self) {} }\n";
        let ns_only_one = format!("{decl}fn main() {{ if c() {{ }}\n.my_math.f(); }}");
        let ns_only_two = format!("{decl}fn main() {{ if c() {{ }}\n\n.my_math.f(); }}");
        let both_one = format!("{decl}{method}fn main() {{ if c() {{ }}\n.my_math.f(); }}");
        let both_two = format!("{decl}{method}fn main() {{ if c() {{ }}\n\n.my_math.f(); }}");
        let not_ns = format!("{decl}fn main() {{ if c() {{ }}\n\n.other.f(); }}");
        let chain = format!("{decl}fn main() {{ if c() {{ }}\n.map(f); }}");
        let ok = ["my_math", "f"];
        assert_eq!(path_at(&ns_only_one, 0).unwrap(), ok); // only a namespace
        assert_eq!(path_at(&ns_only_two, 0).unwrap(), ok);
        assert!(path_at(&both_one, 0).is_none()); // collision, 1 newline: method
        assert_eq!(path_at(&both_two, 0).unwrap(), ok); // collision, blank line: namespace
        assert!(path_at(&not_ns, 0).is_none()); // no such namespace: method
        assert!(path_at(&chain, 0).is_none());
    }
}
