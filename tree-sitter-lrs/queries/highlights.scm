; A színezés a grammar tokenjeire (identifier, punct, string, ...) épül, NEM Rust
; injectionre: az lrs-szöveg egy része (namespace, include, .Path) Rust-szempontból
; hibás, és a hibás Rust-parse elrontja a színezést (és a Neovim injection-hibákat okoz).
; A szabályok később jönnek = erősebbek; a lrs-specifikus rész a végén van.

; --- Alap: literálok, kommentek, zárójelek ---
(line_comment) @comment
(block_comment) @comment
(string) @string
(char) @string
(number) @number
(lifetime) @label
["{" "}"] @punctuation.bracket
(punct) @operator
((punct) @punctuation.bracket (#any-of? @punctuation.bracket "(" ")" "[" "]"))
((punct) @punctuation.delimiter (#any-of? @punctuation.delimiter "," ";" ":"))

; --- Azonosítók: kulcsszavak, típusok, konstansok ---
((identifier) @keyword
  (#any-of? @keyword
    "as" "async" "await" "break" "const" "continue" "crate" "dyn" "else" "enum"
    "extern" "fn" "for" "if" "impl" "in" "let" "loop" "match" "mod" "move" "mut"
    "pub" "ref" "return" "static" "struct" "super" "trait" "type" "unsafe" "use"
    "where" "while" "union"))
((identifier) @boolean (#any-of? @boolean "true" "false"))
((identifier) @variable.builtin (#eq? @variable.builtin "self"))
((identifier) @type.builtin
  (#any-of? @type.builtin
    "i8" "i16" "i32" "i64" "i128" "isize" "u8" "u16" "u32" "u64" "u128" "usize"
    "f32" "f64" "bool" "char" "str" "Self"))
((identifier) @type (#match? @type "^[A-Z]"))
((identifier) @constant (#match? @constant "^[A-Z][A-Z0-9_]+$"))

; --- Függvények, makrók, mezők, definíciók (később = erősebb) ---
((identifier) @function . (punct) @punctuation.bracket (#eq? @punctuation.bracket "("))
((identifier) @function.macro . (punct) @operator . (punct) @punctuation.bracket
  (#eq? @operator "!") (#match? @punctuation.bracket "^[(\\[{]$"))
((dot) . (identifier) @function . (punct) @punctuation.bracket (#eq? @punctuation.bracket "("))
((dot) . (identifier) @property)
((identifier) @keyword . (identifier) @function (#eq? @keyword "fn"))
((identifier) @keyword . (identifier) @type (#any-of? @keyword "struct" "enum" "trait" "type" "union"))
((identifier) @keyword . (identifier) @module (#eq? @keyword "mod"))

; --- lrs-specifikus szerkezetek (a legerősebbek) ---
[
  "namespace"
  "include"
] @keyword.import

(include path: (string) @string.special.path)

(namespace name: (namespace_path (identifier) @module))
(namespace_body (dot) . (identifier) @module)
(dot) @punctuation.special
