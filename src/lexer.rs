//! A Rust-aware lexer. It only needs to be exact about token boundaries
//! (strings, comments, lifetimes, numbers) so that the rest of the pipeline can
//! re-emit source text verbatim.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Ident,
    Lifetime,
    Str,
    Char,
    Num,
    Punct,
}

#[derive(Clone, Copy, Debug)]
pub struct Token {
    pub kind: Kind,
    pub start: usize,
    pub end: usize,
}

pub struct Lexed {
    pub tokens: Vec<Token>,
    pub errors: Vec<(usize, usize, String)>,
}

fn id_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c >= 0x80
}
fn id_cont(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c >= 0x80
}

/// `k` is just after the opening quote.
fn scan_quoted(b: &[u8], mut k: usize) -> (usize, bool) {
    let n = b.len();
    while k < n {
        match b[k] {
            b'\\' => k += 2,
            b'"' => return (k + 1, true),
            _ => k += 1,
        }
    }
    (n, false)
}

fn scan_raw(b: &[u8], mut k: usize, hashes: usize) -> (usize, bool) {
    let n = b.len();
    while k < n {
        if b[k] == b'"' {
            let mut h = 0;
            while h < hashes && k + 1 + h < n && b[k + 1 + h] == b'#' {
                h += 1;
            }
            if h == hashes {
                return (k + 1 + hashes, true);
            }
        }
        k += 1;
    }
    (n, false)
}

/// b"..", br#".."#, r"..", c"..", cr"..", b'x'
fn prefixed(b: &[u8], i: usize) -> Option<(usize, Kind, bool)> {
    let n = b.len();
    let mut j = i;
    let mut raw = false;
    match b[j] {
        b'b' | b'c' => {
            j += 1;
            if j < n && b[j] == b'r' {
                raw = true;
                j += 1;
            }
        }
        b'r' => {
            raw = true;
            j += 1;
        }
        _ => return None,
    }
    if j >= n {
        return None;
    }
    if raw {
        let mut h = 0;
        while j < n && b[j] == b'#' {
            h += 1;
            j += 1;
        }
        if j < n && b[j] == b'"' {
            let (e, ok) = scan_raw(b, j + 1, h);
            return Some((e.min(n), Kind::Str, ok));
        }
        return None;
    }
    if b[j] == b'"' {
        let (e, ok) = scan_quoted(b, j + 1);
        return Some((e.min(n), Kind::Str, ok));
    }
    if b[j] == b'\'' && b[i] == b'b' {
        let mut k = j + 1;
        while k < n && b[k] != b'\'' && b[k] != b'\n' {
            if b[k] == b'\\' {
                k += 1;
            }
            k += 1;
        }
        if k < n && b[k] == b'\'' {
            return Some((k + 1, Kind::Char, true));
        }
    }
    None
}

pub fn lex(src: &str) -> Lexed {
    let b = src.as_bytes();
    let n = b.len();
    let mut i = 0;
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    while i < n {
        let c = b[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c == b'/' && i + 1 < n && b[i + 1] == b'/' {
            while i < n && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            let s = i;
            let mut depth = 0;
            loop {
                if i >= n {
                    errors.push((s, n, "unterminated block comment".to_string()));
                    break;
                }
                if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
                    depth += 1;
                    i += 2;
                } else if b[i] == b'*' && i + 1 < n && b[i + 1] == b'/' {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            continue;
        }

        let s = i;
        if id_start(c) {
            if let Some((e, kind, ok)) = prefixed(b, i) {
                if !ok {
                    errors.push((s, e, "unterminated literal".to_string()));
                }
                tokens.push(Token { kind, start: s, end: e });
                i = e;
                continue;
            }
            if c == b'r' && i + 2 < n && b[i + 1] == b'#' && id_start(b[i + 2]) {
                i += 2;
            }
            while i < n && id_cont(b[i]) {
                i += 1;
            }
            tokens.push(Token { kind: Kind::Ident, start: s, end: i });
            continue;
        }
        if c == b'"' {
            let (e, ok) = scan_quoted(b, i + 1);
            let e = e.min(n);
            if !ok {
                errors.push((s, e, "unterminated string literal".to_string()));
            }
            tokens.push(Token { kind: Kind::Str, start: s, end: e });
            i = e;
            continue;
        }
        if c == b'\'' {
            if i + 1 < n && id_start(b[i + 1]) {
                let mut j = i + 1;
                while j < n && id_cont(b[j]) {
                    j += 1;
                }
                if j < n && b[j] == b'\'' {
                    tokens.push(Token { kind: Kind::Char, start: s, end: j + 1 });
                    i = j + 1;
                } else {
                    tokens.push(Token { kind: Kind::Lifetime, start: s, end: j });
                    i = j;
                }
                continue;
            }
            let mut k = i + 1;
            while k < n && b[k] != b'\'' && b[k] != b'\n' {
                if b[k] == b'\\' {
                    k += 1;
                }
                k += 1;
            }
            if k < n && b[k] == b'\'' {
                tokens.push(Token { kind: Kind::Char, start: s, end: k + 1 });
                i = k + 1;
            } else {
                tokens.push(Token { kind: Kind::Punct, start: s, end: s + 1 });
                i += 1;
            }
            continue;
        }
        if c.is_ascii_digit() {
            i += 1;
            let hex = c == b'0' && i < n && (b[i] == b'x' || b[i] == b'X');
            while i < n {
                let d = b[i];
                if d.is_ascii_alphanumeric() || d == b'_' {
                    if !hex
                        && (d == b'e' || d == b'E')
                        && i + 2 < n
                        && (b[i + 1] == b'+' || b[i + 1] == b'-')
                        && b[i + 2].is_ascii_digit()
                    {
                        i += 2;
                    } else {
                        i += 1;
                    }
                } else if d == b'.' && i + 1 < n && b[i + 1].is_ascii_digit() {
                    i += 1;
                } else {
                    break;
                }
            }
            tokens.push(Token { kind: Kind::Num, start: s, end: i });
            continue;
        }
        let rest = &b[i..];
        let len = if rest.starts_with(b"..=") || rest.starts_with(b"...") {
            3
        } else if rest.starts_with(b"::")
            || rest.starts_with(b"..")
            || rest.starts_with(b"->")
            || rest.starts_with(b"=>")
        {
            2
        } else {
            1
        };
        tokens.push(Token { kind: Kind::Punct, start: s, end: s + len });
        i += len;
    }
    Lexed { tokens, errors }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(s: &str) -> Vec<String> {
        lex(s).tokens.iter().map(|t| s[t.start..t.end].to_string()).collect()
    }

    #[test]
    fn basics() {
        assert_eq!(texts("a..=b"), ["a", "..=", "b"]);
        assert_eq!(texts("'a' 'a x"), ["'a'", "'a", "x"]);
        assert_eq!(texts(r##"r#"x"y"# z"##), [r##"r#"x"y"#"##, "z"]);
        assert_eq!(texts("1.5e-3 1..2 x.0.1"), ["1.5e-3", "1", "..", "2", "x", ".", "0.1"]);
        assert_eq!(texts("/* a /* b */ c */ x // y\nz"), ["x", "z"]);
        assert_eq!(texts("b'\\'' b\"q\""), ["b'\\''", "b\"q\""]);
        assert_eq!(texts("r#type"), ["r#type"]);
    }
}
