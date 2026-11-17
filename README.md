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

`PATH` is a project directory or an `.lrs` file. Build with `cargo build --release`; try the demo with
`cd examples/demo && lrs run .`.

## The language

### Defining namespaces

```rust
namespace Foo {}
namespace Foo.Bar {}          // same as namespace Foo { namespace Bar {} }
```

A namespace can be declared any number of times, in any file; all declarations are merged.
Nested namespaces see the names of the namespaces around them (like C++), and everything not
inside a namespace lives in the root (global) namespace.

### Including files

```rust
include "foo.lrs"             // relative to the including file
include "utils/math.lrs"
include "/graphics/core.lrs"  // leading / = from the project root
```

* Only at the top level of a file (not inside `namespace` blocks or functions).
* Every file is included at most once, circular includes are fine.
* An included file contributes to the *global* namespace tree; the `namespace` blocks inside it say
  where its items go. Include order decides "first definition wins" for mounts.

### Mounting ("using namespace")

```rust
namespace { .Foo }            // everything in .Foo is usable in the root without `.Foo`
namespace { .Foo; .Bar }      // both; .Foo.x and .Bar.x still work. On a clash the first one wins
namespace Foo { .Bar }        // inside .Foo, .Bar's names are usable directly
```

Rules: names defined in the namespace itself always beat mounted names; then mounts in order.
Mounts re-export (`pub use`) what they import, so `.Foo.x` works for names mounted into `.Foo`,
and mounts chain (mounting `.A` also brings what `.A` mounted). A mount only imports what Rust
privacy allows — private items are importable only from inside the namespace.

### Paths

```rust
.Something.Subnamespace.Innerstruct::new()
use .Utils.Math.*;
use .Geo.{Sq, Kind as K2};
let p: .Geo.Pair = ...;      // types, patterns, bounds, impl headers — anywhere a path is allowed
```

A path starts with `.`, is absolute from the root, and `.Something.Sub` without anything after it is
the whole namespace. The path ends at the first item, so `.Config.LIMIT.len()` calls `len` on the
constant. A `.` begins a path when it cannot be a field or method access, i.e. when the previous
token cannot end an expression (`=`, `(`, `,`, `{`, `->`, `:`, `<`, `&`, `|`, keywords like
`return`/`in`/`impl`/`for`/`where`/`use`, …). After `}` only a capitalized segment starts a path, so
`}\n.map(f)` still chains while `}\n.Foo.bar();` calls into a namespace — name your namespaces in
`PascalCase` (they are emitted as `pub mod`).

### Visibility

Exactly like Rust: items are private unless `pub` (or `pub(crate)`); namespaces and includes
are always visible. Privacy is Rust's module privacy applied to namespaces: a private item is visible
inside its namespace and in nested namespaces.

## How it works

1. A Rust-aware lexer feeds a top-level parser that only understands `namespace`, `include`, mount
   lists and item boundaries (items are otherwise opaque — which is what keeps Rust compatibility).
2. All files are merged into a namespace tree. Each namespace becomes `pub mod`, mounts become
   explicit `use` lists computed at transpile time (so "first wins" works), and `.A.b` paths are
   rewritten to `crate::A::b`. Everything else is copied byte for byte.
3. A source map ties every output byte to its `.lrs` origin, used for rustc diagnostics and the LSP.

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

## Tests

`cargo test` runs unit tests, end-to-end tests (transpile, build and run real projects, check privacy,
precedence, include rules, mapped diagnostics) and an LSP session against `examples/demo`.
