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
cd examples/bank && lrs run .
```

## The example

This README explains the language with the project in [`examples/bank`](examples/bank), a small bank
ledger. Every snippet below is copied from it, with its real names, so open it next to this file.

```
examples/bank/
├── lrs.toml          project settings
├── main.lrs          includes, mounts, fn main()
├── account.lrs       namespace Bank.Account   (Account, Error, describe)
├── ledger.lrs        namespace Bank.Ledger    (Journal, Entry, Kind, describe)
├── report.lrs        namespace Bank.Report    (statement, summary)
└── util/
    └── format.lrs    namespace Util.Format    (money, pad, rule)
```

## The language

### Namespaces

`namespace Bank.Ledger { .. }` (in `ledger.lrs`) is short for `namespace Bank { namespace Ledger { .. } }`.
A namespace can be declared in any number of files and the declarations merge: `account.lrs`, `ledger.lrs`
and `report.lrs` all add to `Bank`, so `Bank` ends up holding `Account`, `Ledger` and `Report`. Code outside
any namespace lives in the root. Nested namespaces see the names around them, like in C++.

### Include

```rust
// main.lrs
include "account.lrs"         // relative to this file
include "ledger.lrs"
include "report.lrs"
include "/util/format.lrs"    // leading `/` = from the project root
```

* Top level only (not inside a `namespace` or a function).
* Every file is loaded once, so `main.lrs`, `account.lrs` and `ledger.lrs` can all include `/util/format.lrs`. Cycles are fine.
* The `namespace` blocks inside an included file say where its items go.

### Paths

```rust
println!("{}", .Util.Format.money(needed));
println!("{}", .Bank.Report.statement(&alice, &journal));
```

A path starts with `.` and is absolute (from the root). It ends at the first item, so in
`.Bank.Account::open(1, "Alice").balance()` the path is `.Bank.Account::open` and `.balance()` is an ordinary
method call. A namespace is not a value: `.Bank.Account` on its own is an error.

A `.` starts a path where it cannot be a field or method access: after `=`, `(`, `,`, `{`, `|`, `&`, `->`,
`:`, `<` or a keyword like `return`. After a `}` the name that follows decides: a namespace name is a path,
a method name is a method call (if both exist, a blank line means path, otherwise method call).

### Mounts

A mount lets you use a namespace's names without its prefix (like `using namespace`).

```rust
namespace { .Bank.Account; .Bank.Ledger }     // main.lrs

let mut alice = Account::open(1, "Alice");   // same as .Bank.Account::open(1, "Alice")
describe(&alice)                              // "account #1 of Alice, balance 1075.00"
.Bank.Ledger.describe(journal.history(1)[0])  // "deposit of 1500.00 on account #1"
```

* `Bank.Account` and `Bank.Ledger` both define `describe`: **the first mount wins**. The other is still
  reachable by path.
* A mount goes into the root, or into another namespace: `report.lrs` writes
  `namespace { .Bank.Account; .Bank.Ledger; .Util.Format }` inside `namespace Bank.Report { .. }`, so `Account`,
  `Journal` and `money` need no prefix there. `ledger.lrs` does the same with `namespace { .Bank.Account }`
  inside `Bank.Ledger`.
* Priority: names defined in the namespace itself, then mounts in order. Mounts re-export what they
  import (so `.Bank.Report.Account::open` works) and chain (mounting `.A` also brings what `.A` mounted). They only
  import what Rust privacy allows.

### Privacy

Items without `pub` are private to **the file** that defines them, even when other files add to the same
namespace. Namespaces and includes need no `pub`.

```rust
// util/format.lrs
namespace Util.Format {
    pub fn money(cents: i64) -> String {
        let (sign, whole, part) = split_cents(cents);
        format!("{}{}.{:02}", sign, whole, part)
    }
    fn split_cents(cents: i64) -> (&'static str, i64, i64) { ... }    // private to this file
}

// main.lrs:   .Util.Format.split_cents(12345)   -> error
```

Inside `util/format.lrs`, every namespace of that file can use `.Util.Format.split_cents` by path. Two files
may each have their own private item with the same name. Struct fields keep Rust's rule: `Account.balance`
is private to `account.lrs`, so `alice.balance = 0` in `main.lrs` is an error.

## How it works

1. A Rust-aware lexer feeds a small parser that only understands `namespace`, `include`, mounts and
   item boundaries. Items stay opaque, which is what keeps Rust compatibility.
2. The files are merged into a namespace tree. Each namespace becomes a `pub mod` that re-exports the
   public items of every file. The real items live in a hidden module per file, and non-`pub` items are
   visible only inside it (that is the file-privacy). Mounts become explicit `use` lists, `.A.b`
   becomes `crate::A::b`, everything else is copied byte for byte.
3. A source map ties every output byte to its `.lrs` origin, for rustc diagnostics and the LSP.

The generated crate lives in `<project>/target/lrs/`. Put the usual Cargo keys in `Cargo.toml` (the `[lrs]` table holds lrs settings; `lrs.toml` also works):

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
vim.lsp.config("lrs", { cmd = { "lrs", "lsp" }, filetypes = { "lrs" }, root_markers = { "Cargo.toml", "lrs.toml" } })
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

* Private items of the defining file cannot be mounted into other namespaces of that file
  (mounts import public names only).
* Forward glob imports (`pub use x::*`) through mounts.
* Export `macro_rules!` macros through mounts and resolve macro-generated names.
* Hand the generated crate to rust-analyzer so types/methods get completion and navigation.
* `lrs new` project scaffolding, a syntax-highlighting grammar and an editor extension.

## Tests

`cargo test` runs unit tests, end-to-end tests (transpile, build and run real projects, check privacy,
precedence, include rules, mapped diagnostics) and an LSP session against `examples/bank`.
