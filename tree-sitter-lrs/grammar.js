/// Tree-sitter grammar for `.lrs`.
/// Forrás: src/parse.rs (parse_body, parse_namespace, parse_mounts), src/lexer.rs.
/// lrs-specifikus: `include "..."`, `namespace A.B { .. }`, mount lista `namespace { .Foo; .Bar }`,
/// abszolút út `.Foo.Bar`. Minden más Rust-szöveg token-szinten marad (a színezést a highlights.scm adja).

module.exports = grammar({
  name: "lrs",

  extras: ($) => [/\s/, $.line_comment, $.block_comment],

  word: ($) => $.identifier,

  rules: {
    source_file: ($) => repeat($._top),

    _top: ($) => choice($.include, $.namespace, $._token),

    // include "foo.lrs";  (csak fájl tetején; a parser ellenőrzi)
    include: ($) => seq("include", field("path", $.string), optional(";")),

    // namespace Foo.Bar { ... }  |  namespace { .Foo; .Bar }
    namespace: ($) =>
      seq(
        "namespace",
        optional(field("name", $.namespace_path)),
        field("body", $.namespace_body),
        optional(";"),
      ),

    namespace_path: ($) => seq($.identifier, repeat(seq(".", $.identifier))),

    namespace_body: ($) => seq("{", repeat(choice($.namespace, $._token)), "}"),

    _token: ($) =>
      choice(
        $.identifier,
        $.number,
        $.string,
        $.char,
        $.lifetime,
        $.dot,
        $.punct,
        $.block,
      ),

    // Rust blokk { ... } (namespace nélkül)
    block: ($) => seq("{", repeat($._token), "}"),

    dot: (_) => ".",
    punct: (_) => token(/[^\sA-Za-z0-9_"'{}.]/),

    identifier: (_) => /[A-Za-z_\u0080-\uffff][A-Za-z0-9_\u0080-\uffff]*/,
    number: (_) => /[0-9][0-9A-Za-z_]*/,
    string: (_) => token(seq('"', repeat(choice(/[^"\\]/, /\\./)), '"')),
    char: (_) => token(seq("'", repeat(choice(/[^'\\]/, /\\./)), "'")),
    lifetime: (_) => token(seq("'", /[A-Za-z_][A-Za-z0-9_]*/)),

    line_comment: (_) => token(seq("//", /[^\n]*/)),
    block_comment: (_) => token(seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/")),
  },
});
