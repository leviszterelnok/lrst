//! End-to-end tests: write small projects to a temp dir, run the `lrs` binary.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lrs-e2e-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    for (path, content) in files {
        let p = dir.join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }
    dir
}

fn lrs(args: &[&str], dir: &PathBuf) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lrs")).args(args).current_dir(dir).output().unwrap()
}

fn norm(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn plain_rust_passes_through_unchanged() {
    let src = r####"
#![allow(dead_code)]
use std::collections::HashMap;
/// docs
#[derive(Debug, Clone)]
pub struct S<'a, T: 'a + Clone> { x: &'a T, m: HashMap<String, Vec<Vec<u8>>> }
impl<'a, T: Clone> S<'a, T> {
    fn f(&self) -> Option<T> { Some(self.x.clone()) }
}
const R: &str = r#"raw "string" with .Foo.bar inside"#;
static C: char = '.';
fn g(v: Vec<i32>) -> i32 {
    let a = 1.0f64.max(2.0) as i32;
    let r = 0..=3;
    let t = (1, (2, 3));
    let z = t.1.0;
    match a { 1 | 2 => {} _ => {} }
    let w = v
        .iter()
        .map(|x| x.pow(2))
        .collect::<Vec<_>>();
    if true { 1 } else { 2 }
        .max(3);
    a + z + w.len() as i32 + r.count() as i32 + std::mem::size_of::<S<'static, u8>>() as i32
}
macro_rules! m { ($x:expr) => { $x.len() }; }
mod inner { pub fn h() {} }
fn main() { println!("{}", g(vec![1])); inner::h(); let _ = m!("ab"); }
"####;
    let dir = project("plain", &[("main.lrs", src)]);
    let out = lrs(&["transpile", "main.lrs"], &dir);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let code = norm(&String::from_utf8_lossy(&out.stdout));
    // the crate-level attribute is hoisted; the items sit inside the file's hidden module
    let body = norm(src.split_once("#![allow(dead_code)]").unwrap().1);
    let (before_main, main_fn) = body.split_once("fn main").unwrap();
    assert!(code.starts_with("#![allow(dead_code)]"), "{}", code);
    assert!(code.contains(before_main.trim()), "items must be copied token for token");
    assert!(code.contains(&format!("pub(crate) fn main{}", main_fn)), "main is exported crate-wide");
    // and it is still a valid, compiling Rust program
    let out = lrs(&["check", "main.lrs"], &dir);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn paths_in_every_position() {
    let src = r#"
namespace Geo {
    pub trait Area { fn area(&self) -> f64; }
    pub struct Sq(pub f64);
    impl Area for Sq { fn area(&self) -> f64 { self.0 * self.0 } }
    pub enum Kind { A(i32), B }
    pub const K: Kind = Kind::B;
    pub fn id<T>(t: T) -> T { t }
    pub static NAMES: [&str; 2] = ["a", "b"];
    pub type Pair = (i32, i32);
}
namespace Mod.Deep {
    pub struct Wrapper<T> { pub v: T }
    impl<T: .Geo.Area> Wrapper<T> { pub fn go(&self) -> f64 { self.v.area() } }
}

use .Geo.{Sq, Kind as K2};
pub use .Geo.id as ident;

struct Mine;
impl .Geo.Area for Mine { fn area(&self) -> f64 { 2.0 } }
fn takes<T>(t: T) -> f64 where T: .Geo.Area { t.area() }
fn describe(k: .Geo.Kind) -> String {
    match k { .Geo.Kind::A(n) => format!("A{}", n), .Geo.Kind::B => "B".into() }
}
fn main() {
    let w = .Mod.Deep.Wrapper { v: Sq(3.0) };
    println!("{}", w.go());
    println!("{}", takes(Mine));
    println!("{}", describe(.Geo.Kind::A(4)));
    println!("{}", describe(.Geo.K));
    let p: .Geo.Pair = (1, 2);
    println!("{:?} {}", p, .Geo.NAMES[1]);
    println!("{}", ident(5));
    let f: fn(i32) -> i32 = .Geo.id;
    println!("{}", f(7) + .Geo.id::<i32>(1));
    let c = |x: i32| .Geo.id(x) * 2;
    println!("{}", c(4));
    let s = if true { .Geo.NAMES[0] } else { "z" };
    println!("{}", s);
    let v = vec![.Geo.Kind::B];
    println!("{}", v.len());
    let t = (.Geo.id(1), .Geo.id(2));
    println!("{:?}", t);
    let n = [1, 2, 3].iter().map(|x| x * 2).collect::<Vec<_>>();
    println!("{:?}", n);
    let u = Some(3).map(|x| .Geo.id(x)).unwrap_or(0);
    println!("{}", u);
    struct Local { a: i32 }
    let l = Local { a: .Geo.id(9) };
    println!("{}", l.a);
    let _ = K2::B;
}
"#;
    let dir = project("paths", &[("main.lrs", src)]);
    let out = lrs(&["run", "main.lrs"], &dir);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{}", stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let want = "9\n2\nA4\nB\n(1, 2) b\n5\n8\n8\na\n1\n(1, 2)\n[2, 4, 6]\n3\n9\n";
    assert_eq!(stdout, want);
}

#[test]
fn mount_precedence_and_privacy() {
    let files = [
        (
            "main.lrs",
            r#"
include "a.lrs"
include "/lib/b.lrs"
namespace { .A; .B }
namespace C { pub fn x() -> &'static str { "C" } }
namespace C { .A }          // local `x` wins over the mounted one
fn main() {
    println!("{}", x());       // A wins over B (first mount)
    println!("{}", y());       // only B has y
    println!("{}", .B.x());    // explicit paths see both
    println!("{}", .C.x());    // local definition wins inside C
    println!("{}", .C.y2());   // C's mount exposes A's pub items
}
"#,
        ),
        ("a.lrs", "namespace A { pub fn x() -> &'static str { \"A\" } pub fn y2() -> &'static str { \"A.y2\" } fn hidden() {} }"),
        ("lib/b.lrs", "namespace B { pub fn x() -> &'static str { \"B\" } pub fn y() -> &'static str { \"B.y\" } }"),
    ];
    let dir = project("mounts", &files);
    let out = lrs(&["run", "main.lrs"], &dir);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "A\nB.y\nB\nC\nA.y2\n");

    // private items are not importable through a mount
    let bad = project(
        "privacy",
        &[("main.lrs", "namespace A { fn hidden() {} pub fn shown() {} }\nnamespace { .A }\nfn main() { shown(); hidden(); }\n")],
    );
    let out = lrs(&["check", "main.lrs"], &bad);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("hidden"), "{}", err);
    assert!(err.contains("main.lrs:3:"), "error should point into the .lrs file: {}", err);
}

#[test]
fn include_rules() {
    // includes are once-only, may be circular, and are root-relative with a leading slash
    let files = [
        ("main.lrs", "include \"a.lrs\"\ninclude \"sub/../a.lrs\"\nfn main() { println!(\"{}\", .A.f() + .B.g()); }\n"),
        ("a.lrs", "include \"/sub/b.lrs\"\nnamespace A { pub fn f() -> i32 { 1 } }\n"),
        ("sub/b.lrs", "include \"../a.lrs\"\nnamespace B { pub fn g() -> i32 { 2 } }\n"),
    ];
    let dir = project("includes", &files);
    let out = lrs(&["run", "main.lrs"], &dir);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");

    // includes are rejected inside namespaces
    let bad = project("include-nested", &[("main.lrs", "namespace A { include \"x.lrs\" }\nfn main() {}\n")]);
    let out = lrs(&["check", "main.lrs"], &bad);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("top level"));
}

#[test]
fn project_with_dependencies_and_lib() {
    let files = [
        ("lrs.toml", "[package]\nname = \"libdemo\"\nversion = \"0.1.0\"\n\n[lrs]\nentry = \"lib.lrs\"\n"),
        ("lib.lrs", "pub namespace_marker();\n"),
    ];
    // a library entry that does not parse as valid Rust must fail with a mapped error
    let dir = project("lib-bad", &files);
    let out = lrs(&["check", "."], &dir);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("lib.lrs:1"), "{}", String::from_utf8_lossy(&out.stderr));

    let files = [
        ("lrs.toml", "[package]\nname = \"libdemo\"\nversion = \"0.1.0\"\n\n[lrs]\nentry = \"lib.lrs\"\n"),
        ("lib.lrs", "namespace Util { pub fn two() -> i32 { 2 } }\n#[test]\nfn t() { assert_eq!(.Util.two(), 2); }\n"),
    ];
    let dir = project("lib-ok", &files);
    let out = lrs(&["build", "."], &dir);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn nesting_and_path_end() {
    let src = r#"
fn top() -> i32 { 1 }
namespace Config { pub const LIMIT: &str = "abcd"; pub fn local() -> i32 { 10 } }
namespace Config.Inner {
    pub fn f() -> i32 { top() + local() }    // sees the enclosing namespaces
}
fn main() {
    println!("{}", .Config.LIMIT.len());      // path ends at the item; `.len()` is a method call
    println!("{}", .Config.Inner.f());
}
"#;
    let dir = project("nesting", &[("main.lrs", src)]);
    let out = lrs(&["run", "main.lrs"], &dir);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "4\n11\n");
}

#[test]
fn private_items_are_file_private() {
    let files = [
        (
            "main.lrs",
            r#"
include "b.lrs"
include "c.lrs"
namespace B { pub fn from_main() -> i32 { public_stuff() + 1 } }   // pub item of another file, unqualified
fn main() {
    println!("{}", .B.public_stuff());
    println!("{}", .B.from_main());
    println!("{}", .Other.peek());
    println!("{}", .B.c_view());
}
"#,
        ),
        (
            "b.lrs",
            r#"
namespace B {
    fn internal_stuff() -> i32 { 40 }
    fn same_name() -> i32 { 1 }
    pub fn public_stuff() -> i32 { internal_stuff() + 2 + same_name() - 1 }
}
// another namespace in the same file may use the file's private items by path
namespace Other { pub fn peek() -> i32 { .B.internal_stuff() } }
"#,
        ),
        ("c.lrs", "namespace B { fn same_name() -> i32 { 7 } pub fn c_view() -> i32 { same_name() } }\n"),
    ];
    let dir = project("file-private", &files);
    let out = lrs(&["run", "main.lrs"], &dir);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "42\n43\n40\n7\n");

    // by path from another file: rejected with an explanation
    let b = "namespace B { fn internal_stuff() {} pub fn ok() {} }\n";
    let dir = project("file-private-path", &[("main.lrs", "include \"b.lrs\"\nfn main() { .B.ok(); .B.internal_stuff(); }\n"), ("b.lrs", b)]);
    let out = lrs(&["check", "main.lrs"], &dir);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(err.contains("`internal_stuff` is private to b.lrs"), "{}", err);
    assert!(err.contains("main.lrs:2:"), "{}", err);

    // unqualified, from the same namespace in another file: rustc rejects it, mapped to main.lrs
    let main = "include \"b.lrs\"\nnamespace B { pub fn x() -> i32 { internal_stuff() } }\nfn main() { .B.x(); }\n";
    let dir = project("file-private-unq", &[("main.lrs", main), ("b.lrs", b)]);
    let out = lrs(&["check", "main.lrs"], &dir);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    assert!(err.contains("internal_stuff") && err.contains("main.lrs:2:"), "{}", err);
}
