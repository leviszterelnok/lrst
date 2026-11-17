//! Emission of plain Rust from a loaded project, with a source map back to
//! the `.lrs` files.

use crate::lexer::Kind;
use crate::model::*;
use crate::resolve::*;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct MapEntry {
    pub out: usize,
    pub out_len: usize,
    pub file: usize,
    pub src: usize,
    pub src_len: usize,
    /// verbatim copy (offsets map 1:1) vs. generated text (maps to a fixed span)
    pub linear: bool,
}

#[derive(Clone, Default, Debug)]
pub struct SourceMap {
    pub entries: Vec<MapEntry>,
}

impl SourceMap {
    /// Map the start of an output span to (file, source offset).
    pub fn start(&self, off: usize) -> Option<(usize, usize)> {
        let i = self.entries.partition_point(|e| e.out <= off);
        if i == 0 {
            return None;
        }
        let e = &self.entries[i - 1];
        Some((e.file, if e.linear { e.src + (off - e.out).min(e.src_len) } else { e.src }))
    }

    /// Map the (exclusive) end of an output span.
    pub fn end(&self, off: usize) -> Option<(usize, usize)> {
        let i = self.entries.partition_point(|e| e.out < off);
        if i == 0 {
            return None;
        }
        let e = &self.entries[i - 1];
        Some((e.file, if e.linear { e.src + (off - e.out).min(e.src_len) } else { e.src + e.src_len }))
    }
}

pub struct Emitted {
    pub code: String,
    pub map: SourceMap,
    pub diags: Vec<Diag>,
}

struct Em<'a> {
    p: &'a Project,
    out: String,
    map: SourceMap,
    diags: Vec<Diag>,
}

pub fn emit(p: &Project) -> Emitted {
    let mut em = Em { p, out: String::new(), map: SourceMap::default(), diags: vec![] };
    em.emit_ns(0);
    Emitted { code: em.out, map: em.map, diags: em.diags }
}

fn path_str(p: &Project, ns: usize) -> String {
    let mut s = String::from("crate");
    for seg in p.ns_path(ns) {
        s.push_str("::");
        s.push_str(&seg);
    }
    s
}

impl<'a> Em<'a> {
    fn raw(&mut self, s: &str) {
        self.out.push_str(s);
    }

    fn mapped(&mut self, s: &str, file: usize, src: usize, src_len: usize, linear: bool) {
        if s.is_empty() {
            return;
        }
        self.map.entries.push(MapEntry { out: self.out.len(), out_len: s.len(), file, src, src_len, linear });
        self.out.push_str(s);
    }

    /// Copy source bytes [a, b) of a file verbatim.
    fn copy(&mut self, fid: usize, a: usize, b: usize) {
        if a >= b {
            return;
        }
        let p = self.p;
        let s = &p.files[fid].text[a..b];
        self.mapped(s, fid, a, b - a, true);
    }

    fn emit_ns(&mut self, id: usize) {
        let p = self.p;
        let ns = &p.nss[id];
        let anchor = ns.decls.first().map(|d| (d.file, d.header_start, d.name_end - d.header_start));
        if id != 0 {
            let hdr = format!("#[allow(non_snake_case)]\npub mod {} {{\n", ns.name);
            match anchor {
                Some((f, s, l)) => self.mapped(&hdr, f, s, l, false),
                None => self.raw(&hdr),
            }
        }
        for &(f, a, b) in &ns.inner_attrs {
            self.range(f, a, b);
            self.raw("\n");
        }
        if id != 0 {
            let g = "#[allow(unused_imports)]\nuse super::*;\n";
            match anchor {
                Some((f, s, l)) => self.mapped(g, f, s, l, false),
                None => self.raw(g),
            }
        }
        self.emit_mounts(id);
        for it in &ns.items {
            self.range(it.file, it.first, it.last);
            self.raw("\n");
        }
        for (_, c) in &ns.children {
            self.emit_ns(*c);
        }
        if id != 0 {
            self.raw("}\n");
        }
    }

    fn emit_mounts(&mut self, id: usize) {
        let p = self.p;
        let ns = &p.nss[id];
        if ns.mounts.is_empty() {
            return;
        }
        // local definitions always win over mounted names
        let mut taken: HashSet<String> = HashSet::new();
        for (c, _) in &ns.children {
            taken.insert(c.clone());
        }
        for it in &ns.items {
            if let Some(n) = &it.name {
                taken.insert(n.clone());
            }
            for n in &it.use_names {
                taken.insert(n.clone());
            }
        }
        for m in &ns.mounts {
            let Some(t) = ns_by_path(p, &m.path) else {
                self.diags.push(Diag::error(
                    m.file,
                    m.start,
                    m.end,
                    format!("cannot find namespace `.{}`", m.path.join(".")),
                ));
                continue;
            };
            let prefix = path_str(p, t);
            for (name, vis) in visible_names(p, t, id) {
                if !taken.insert(name.clone()) {
                    continue; // first definition wins
                }
                let line = format!("#[allow(unused_imports)]\n{} {}::{};\n", vis.use_prefix(), prefix, name);
                self.mapped(&line, m.file, m.start, m.end - m.start, false);
            }
        }
    }

    /// Emit the tokens [a, b) of a file, rewriting `.A.b` paths.
    fn range(&mut self, fid: usize, a: usize, b: usize) {
        let p = self.p;
        let f = &p.files[fid];
        let toks = &f.tokens;
        let mut in_use = false;
        let mut t = a;
        while t < b {
            let tok = toks[t];
            let prev_end = if t == 0 { 0 } else { toks[t - 1].end };
            self.copy(fid, prev_end, tok.start);
            if tok.kind == Kind::Punct && f.tok(t) == "." {
                if let Some(chain) = scan_path(&f.text, toks, t, b) {
                    t = self.path(fid, t, &chain, in_use);
                    continue;
                }
            }
            self.copy(fid, tok.start, tok.end);
            if tok.kind == Kind::Ident && f.tok(t) == "use" {
                in_use = true;
            } else if f.tok(t) == ";" {
                in_use = false;
            }
            t += 1;
        }
    }

    fn path(&mut self, fid: usize, dot_t: usize, chain: &[usize], in_use: bool) -> usize {
        let p = self.p;
        let f = &p.files[fid];
        let names: Vec<&str> = chain.iter().map(|&i| f.tok(i)).collect();
        let r = resolve_path(p, &names, in_use);
        if let Some((idx, msg, sev)) = &r.problem {
            let tk = f.tokens[chain[*idx]];
            self.diags.push(Diag::new(*sev, fid, tk.start, tk.end, msg.clone()));
        }
        self.mapped("crate", fid, f.tokens[dot_t].start, 1, false);
        let mut last = dot_t;
        for &ident in chain.iter().take(r.consumed) {
            self.mapped("::", fid, f.tokens[ident - 1].start, 1, false);
            let tk = f.tokens[ident];
            self.copy(fid, tk.start, tk.end);
            last = ident;
        }
        let mut next = last + 1;
        if in_use
            && is_single_dot(&f.text, &f.tokens, next)
            && next + 1 < f.tokens.len()
            && matches!(f.tok(next + 1), "*" | "{")
        {
            self.mapped("::", fid, f.tokens[next].start, 1, false);
            next += 1;
        }
        next
    }
}
