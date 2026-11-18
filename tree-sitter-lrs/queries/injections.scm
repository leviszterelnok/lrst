; A teljes forrás Rust-ként is parse-olódik: a Rust színezés (fn, típusok,
; függvények, stringek, kommentek) innen jön. Az lrs-specifikus részeket
; (include, namespace, mount) a highlights.scm emeli ki.
((source_file) @injection.content
  (#set! injection.language "rust")
  (#set! injection.include-children))
