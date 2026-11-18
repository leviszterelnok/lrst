; lrs-specifikus szavak és szerkezetek
[
  "namespace"
  "include"
] @keyword.import

(include path: (string) @string.special.path)

(namespace name: (namespace_path (identifier) @module))
(namespace_path "." @punctuation.delimiter)
(namespace_body (dot) . (identifier) @module)
(dot) @punctuation.special

; Rust-szöveg (token-szinten)
(string) @string
(char) @character
(number) @number
(lifetime) @label
(line_comment) @comment
(block_comment) @comment

((identifier) @keyword
  (#any-of? @keyword "fn" "let" "mut" "pub" "struct" "enum" "impl" "trait" "use" "mod"
            "const" "static" "return" "if" "else" "match" "for" "while" "loop" "in"
            "where" "as" "ref" "move" "async" "await" "dyn" "unsafe" "type"))

((identifier) @type.builtin
  (#any-of? @type.builtin "i8" "i16" "i32" "i64" "i128" "isize"
            "u8" "u16" "u32" "u64" "u128" "usize" "f32" "f64" "bool" "char" "str"))

((identifier) @boolean
  (#any-of? @boolean "true" "false"))

((identifier) @type
  (#match? @type "^[A-Z]"))

; függvényhívás: ident közvetlenül "(" előtt
((identifier) @function.call
  .
  (punct) @_paren
  (#eq? @_paren "("))

(identifier) @variable
