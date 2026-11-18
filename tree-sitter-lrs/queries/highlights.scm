; Csak az lrs-specifikus szerkezetek. A Rust-szöveg színezését az injections.scm
; adja: a Rust tree-sitter parser fut rá, így a Rust szabályai érvényesülnek.

[
  "namespace"
  "include"
] @keyword.import

(include path: (string) @string.special.path)

(namespace name: (namespace_path (identifier) @module))
(namespace_body (dot) . (identifier) @module)
(dot) @punctuation.special
