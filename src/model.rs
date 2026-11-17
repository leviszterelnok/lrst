//! Data model shared by the parser, resolver, emitter and language server.

use crate::lexer::Token;
use std::path::{Path, PathBuf};

pub struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(s: &str) -> Self {
        let mut starts = vec![0];
        for (i, b) in s.bytes().enumerate() {
            if b == b'\n' {
                starts.push(i + 1);
            }
        }
        LineIndex { starts }
    }

    pub fn line_of(&self, off: usize) -> usize {
        match self.starts.binary_search(&off) {
            Ok(i) => i,
            Err(i) => i - 1,
        }
    }

    fn clamp(s: &str, mut off: usize) -> usize {
        off = off.min(s.len());
        while !s.is_char_boundary(off) {
            off -= 1;
        }
        off
    }

    /// 0-based (line, column in chars) — used for terminal output.
    pub fn line_col_chars(&self, s: &str, off: usize) -> (usize, usize) {
        let off = Self::clamp(s, off);
        let l = self.line_of(off);
        (l, s[self.starts[l]..off].chars().count())
    }

    /// 0-based (line, column in UTF-16 units) — used by LSP.
    pub fn position(&self, s: &str, off: usize) -> (u32, u32) {
        let off = Self::clamp(s, off);
        let l = self.line_of(off);
        (l as u32, s[self.starts[l]..off].encode_utf16().count() as u32)
    }

    pub fn offset(&self, s: &str, line: u32, col: u32) -> usize {
        let line = (line as usize).min(self.starts.len() - 1);
        let ls = self.starts[line];
        let mut units = 0u32;
        for (i, ch) in s[ls..].char_indices() {
            if ch == '\n' || units >= col {
                return ls + i;
            }
            units += ch.len_utf16() as u32;
        }
        s.len()
    }

    pub fn line_text<'a>(&self, s: &'a str, line: usize) -> &'a str {
        let a = self.starts[line];
        let b = self.starts.get(line + 1).copied().unwrap_or(s.len());
        s[a..b].trim_end_matches(['\n', '\r'])
    }
}

pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
    pub tokens: Vec<Token>,
    pub lines: LineIndex,
    /// For each bracket token, the index of its partner (usize::MAX if unmatched).
    pub matching: Vec<usize>,
}

impl SourceFile {
    pub fn tok(&self, i: usize) -> &str {
        match self.tokens.get(i) {
            Some(t) => &self.text[t.start..t.end],
            None => "",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug)]
pub struct Diag {
    pub file: Option<usize>,
    pub start: usize,
    pub end: usize,
    pub severity: Severity,
    pub message: String,
    pub code: Option<String>,
}

impl Diag {
    pub fn new(sev: Severity, file: usize, start: usize, end: usize, message: impl Into<String>) -> Diag {
        Diag { file: Some(file), start, end, severity: sev, message: message.into(), code: None }
    }
    pub fn error(file: usize, start: usize, end: usize, message: impl Into<String>) -> Diag {
        Diag::new(Severity::Error, file, start, end, message)
    }
    pub fn warning(file: usize, start: usize, end: usize, message: impl Into<String>) -> Diag {
        Diag::new(Severity::Warning, file, start, end, message)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Vis {
    Pub,
    Crate,
    Priv,
}

impl Vis {
    pub fn use_prefix(self) -> &'static str {
        match self {
            Vis::Pub => "pub use",
            Vis::Crate => "pub(crate) use",
            Vis::Priv => "use",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub file: usize,
    /// token range [first, last) including attributes
    pub first: usize,
    pub last: usize,
    /// first token after the attributes (for signatures / hover)
    pub head: usize,
    pub name: Option<String>,
    pub name_tok: Option<usize>,
    pub kind: String,
    pub vis: Vis,
    /// names introduced by a `use` item
    pub use_names: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Mount {
    pub file: usize,
    pub path: Vec<String>,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug)]
pub struct Decl {
    pub file: usize,
    pub name_start: usize,
    pub name_end: usize,
    pub header_start: usize,
}

pub struct Ns {
    pub name: String,
    pub parent: Option<usize>,
    pub children: Vec<(String, usize)>,
    pub items: Vec<Item>,
    pub mounts: Vec<Mount>,
    /// `#![...]` attributes: (file, first token, last token exclusive)
    pub inner_attrs: Vec<(usize, usize, usize)>,
    pub decls: Vec<Decl>,
}

impl Ns {
    pub fn new(name: String, parent: Option<usize>) -> Ns {
        Ns { name, parent, children: vec![], items: vec![], mounts: vec![], inner_attrs: vec![], decls: vec![] }
    }
}

#[derive(Clone, Debug)]
pub struct IncludeRef {
    pub file: usize,
    pub start: usize,
    pub end: usize,
    pub target: PathBuf,
}

pub struct Project {
    pub root_dir: PathBuf,
    pub entry: PathBuf,
    pub files: Vec<SourceFile>,
    pub nss: Vec<Ns>,
    pub includes: Vec<IncludeRef>,
    pub diags: Vec<Diag>,
}

impl Project {
    pub fn new(root_dir: PathBuf, entry: PathBuf) -> Project {
        Project {
            root_dir,
            entry,
            files: vec![],
            nss: vec![Ns::new(String::new(), None)],
            includes: vec![],
            diags: vec![],
        }
    }

    pub fn file_index(&self, path: &Path) -> Option<usize> {
        self.files.iter().position(|f| f.path == path)
    }

    /// Path of a namespace from the root, e.g. ["Foo", "Bar"].
    pub fn ns_path(&self, mut id: usize) -> Vec<String> {
        let mut v = vec![];
        while let Some(p) = self.nss[id].parent {
            v.push(self.nss[id].name.clone());
            id = p;
        }
        v.reverse();
        v
    }

    pub fn is_descendant_or_eq(&self, mut id: usize, of: usize) -> bool {
        loop {
            if id == of {
                return true;
            }
            match self.nss[id].parent {
                Some(p) => id = p,
                None => return false,
            }
        }
    }
}
