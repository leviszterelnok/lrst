# lrs — Rust with namespaces and includes

`lrs` is a transpiler for a Rust superset (`.lrs` files). Everything that is valid Rust stays valid;
on top of it you get a different module system: **namespaces**, **includes**, and **mounts**.
`lrs` flattens a project into one ordinary Rust crate and runs `cargo` on it, so you keep the
whole Rust ecosystem, with compiler errors mapped back to your `.lrs` lines. It also ships a
language server (`lrs lsp`).

```
lrs transpile [PATH] [-o FILE]     print (or write) the generated Rust
lrs check     [PATH] [--release]   transpile + cargo check
lrs build     [PATH] [--release]   transpile + cargo build
lrs run       [PATH] [--release] [-- ARGS...]
lrs lsp                            language server on stdio
```

`PATH` is a project directory or an `.lrs` file. To try it:

```
cargo install --path .
cd examples/demo && lrs run .
```

## The demo

This README explains the language with the project in [`examples/demo`](examples/demo). Every snippet
below is copied from it, with its real names, so open the demo next to this file.

```
examples/demo/
├── main.lrs         includes, mounts, fn main()
├── shapes.lrs       namespace Shapes      (Circle, Rect, label)
└── util/
    ├── math.lrs     namespace Util.Math   (PI, sqr, clamp)
    └── text.lrs     namespace Util.Text   (shout, label)
```

## The language

### Namespaces

`namespace Util.Math { .. }` (in `util/math.lrs`) is short for `namespace Util { namespace Math { .. } }`.
A namespace can be declared in any number of files and the declarations merge: `util/text.lrs` declares
`Util.Text`, so `Util` ends up holding both `Math` and `Text`. Code outside any namespace lives in the
root. Nested namespaces see the names around them, like in C++.

### Include

```rust
// main.lrs
include "shapes.lrs"          // relative to this file
include "util/text.lrs"
include "/util/math.lrs"      // leading `/` = from the project root
```

* Top level only (not inside a `namespace` or a function).
* Every file is loaded once, so `main.lrs` and `shapes.lrs` can both include `util/math.lrs`. Cycles are fine.
* The `namespace` blocks inside an included file say where its items go.

### Paths

```rust
println!("{}", .Util.Math.clamp(15, 0, 10));
radii.iter().map(|r| .Shapes.Circle::new(*r).area())
```

A path starts with `.` and is absolute (from the root). It ends at the first item, so in
`.Shapes.Circle::new(*r).area()` the path is `.Shapes.Circle::new` and `.area()` is an ordinary method
call. A namespace is not a value: `.Util.Math` on its own is an error.

A `.` starts a path where it cannot be a field or method access: after `=`, `(`, `,`, `{`, `|`, `&`, `->`,
`:`, `<` or a keyword like `return`. After a `}` the name that follows decides: a namespace name is a path,
a method name is a method call (if both exist, a blank line means path, otherwise method call).

### Mounts

A mount lets you use a namespace's names without its prefix (like `using namespace`).

```rust
namespace { .Shapes; .Util.Text }     // main.lrs

let circle = Circle::new(1.0);        // same as .Shapes.Circle::new(1.0)
shout("hello")                        // same as .Util.Text.shout("hello")
label("x")                            // "[shape] x"
.Util.Text.label("x")                 // "[text] x"
```

* `Shapes` and `Util.Text` both define `label`: **the first mount wins**. The other is still reachable by path.
* A mount goes into the root, or into another namespace: `main.lrs` writes `namespace { .Shapes }` inside
  `namespace Report { .. }`, so `Circle` and `Rect` need no prefix there (`namespace Report { .Shapes }` is
  the short form when the block holds nothing else).
* Priority: names defined in the namespace itself, then mounts in order. Mounts re-export what they
  import (so `.Report.Circle` works) and chain (mounting `.A` also brings what `.A` mounted). They only
  import what Rust privacy allows.

### Privacy

Items without `pub` are private to **the file** that defines them, even when other files add to the same
namespace. Namespaces and includes need no `pub`.

```rust
// util/math.lrs
namespace Util.Math {
    pub fn sqr(x: f64) -> f64 { internal_mul(x, x) }
    fn internal_mul(a: f64, b: f64) -> f64 { a * b }    // private to this file
}

// main.lrs:   .Util.Math.internal_mul(2.0, 3.0)   -> error
```

Inside `util/math.lrs`, every namespace of that file can use `.Util.Math.internal_mul` by path. Two files
may each have their own private item with the same name. Struct fields keep Rust's rule.

## How it works

1. A Rust-aware lexer feeds a small parser that only understands `namespace`, `include`, mounts and
   item boundaries. Items stay opaque, which is what keeps Rust compatibility.
2. The files are merged into a namespace tree. Each namespace becomes a `pub mod` that re-exports the
   public items of every file. The real items live in a hidden module per file, and non-`pub` items are
   visible only inside it (that is the file-privacy). Mounts become explicit `use` lists, `.A.b`
   becomes `crate::A::b`, everything else is copied byte for byte.
3. A source map ties every output byte to its `.lrs` origin, for rustc diagnostics and the LSP.

The generated crate lives in `<project>/target/lrs/`. Put the usual Cargo keys in `lrs.toml`:

```toml
[package]
name = "demo"
version = "0.1.0"

[lrs]
entry = "main.lrs"       # default: main.lrs, else lib.lrs (a lib.lrs entry builds a library)

[dependencies]
serde = "1"
```

## Language server

`lrs lsp` speaks LSP over stdio and works for every `.lrs` file in a project (the project is found via
`lrs.toml`; a loose file is its own project):

* diagnostics: include/namespace/mount errors as you type; rustc errors (mapped back to `.lrs`) on save
* go to definition: paths, `namespace A.B` segments, mount lists, `include "…"` strings
* hover (signature, doc comments, namespace summary), `.`-completion of namespaces and items,
  document symbols

Editor setup — any LSP client, e.g. Neovim:

```lua
vim.filetype.add({ extension = { lrs = "lrs" } })
vim.lsp.config("lrs", { cmd = { "lrs", "lsp" }, filetypes = { "lrs" }, root_markers = { "lrs.toml" } })
vim.lsp.enable("lrs")
```

(`rust` syntax highlighting works well for `.lrs` in most editors: associate the extension with Rust.)

## Limitations

* Rustc diagnostics map to the exact `.lrs` token; spans inside generated glue map to the
  `namespace` declaration.
* Names produced by macros are unknown to `lrs`: `.Ns.generated_name` resolves with a warning
  (rustc has the final say), and `macro_rules!` macros are not exported through mounts.
* Mounts import names, not glob-imported names (`pub use x::*` inside a namespace is not forwarded).
* LSP completion/definition work on the structure `lrs` knows (namespaces, items, mounts), not on
  types or methods — pair it with rust-analyzer on the generated crate if you need that.

## TODO

* Privacy variants: a stricter `private` that is not even visible to namespaces nested in the same
  file, and `pub(file)`-style visibility are not implemented (non-`pub` = file-private only).
* Private items of the defining file cannot be mounted into other namespaces of that file
  (mounts import public names only).
* Forward glob imports (`pub use x::*`) through mounts.
* Export `macro_rules!` macros through mounts and resolve macro-generated names.
* Hand the generated crate to rust-analyzer so types/methods get completion and navigation.
* `lrs new` project scaffolding, a syntax-highlighting grammar and an editor extension.

## Tests

`cargo test` runs unit tests, end-to-end tests (transpile, build and run real projects, check privacy,
precedence, include rules, mapped diagnostics) and an LSP session against `examples/demo`.
