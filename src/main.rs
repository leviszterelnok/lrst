use lrs::driver::*;
use lrs::model::Severity;
use lrs::parse::Overlay;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const HELP: &str = "\
lrs - Rust with namespaces and includes

USAGE:
    lrs transpile [PATH] [-o FILE]     print (or write) the generated Rust
    lrs check     [PATH] [--release]   transpile + cargo check
    lrs build     [PATH] [--release]   transpile + cargo build
    lrs run       [PATH] [--release] [-- ARGS...]
    lrs lsp                            run the language server on stdio

PATH is a project directory (with lrs.toml / main.lrs / lib.lrs) or an .lrs file.
";

fn main() {
    std::process::exit(real_main());
}

fn real_main() -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first().map(|s| s.as_str()) else {
        eprint!("{}", HELP);
        return 2;
    };
    match cmd {
        "-h" | "--help" | "help" => {
            print!("{}", HELP);
            0
        }
        "-V" | "--version" | "version" => {
            println!("lrs {}", env!("CARGO_PKG_VERSION"));
            0
        }
        "lsp" => lrs::lsp::run(),
        "transpile" | "check" | "build" | "run" => compile(cmd, &args[1..]),
        other => {
            eprintln!("unknown command `{}`\n\n{}", other, HELP);
            2
        }
    }
}

fn compile(cmd: &str, args: &[String]) -> i32 {
    let mut path = PathBuf::from(".");
    let mut out: Option<PathBuf> = None;
    let mut release = false;
    let mut run_args: Vec<String> = vec![];
    let mut seen_path = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--" => {
                run_args = args[i + 1..].to_vec();
                break;
            }
            "-o" if i + 1 < args.len() => {
                out = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--release" => release = true,
            a if !seen_path && !a.starts_with('-') => {
                path = PathBuf::from(a);
                seen_path = true;
            }
            a => {
                eprintln!("unexpected argument `{}`", a);
                return 2;
            }
        }
        i += 1;
    }

    let cfg = match find_project(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {}", e);
            return 1;
        }
    };
    let cwd = std::env::current_dir().unwrap_or_default();
    let analysis = match analyze(&cfg.entry, &cfg.root, &Overlay::new()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {}", e);
            return 1;
        }
    };
    let diags = analysis.diags();
    for d in &diags {
        eprintln!("{}", render_diag(&analysis.project, d, &cwd));
    }
    if diags.iter().any(|d| d.severity == Severity::Error) {
        eprintln!("lrs: aborting due to errors in the namespace/include stage");
        return 1;
    }

    if cmd == "transpile" {
        return match out {
            Some(f) => match std::fs::write(&f, &analysis.emitted.code) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("error: {}: {}", f.display(), e);
                    1
                }
            },
            None => {
                print!("{}", analysis.emitted.code);
                0
            }
        };
    }

    let dir = match write_gen(&cfg, &analysis.emitted.code) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {}", e);
            return 1;
        }
    };
    let sub = if cmd == "check" { "check" } else { "build" };
    let res = match cargo_run(&dir, sub, release, cfg.is_lib, &analysis.project, &analysis.emitted.map) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {}", e);
            return 1;
        }
    };
    for d in &res.diags {
        eprintln!("{}", render_diag(&analysis.project, d, &cwd));
    }
    for o in &res.other {
        eprintln!("{}", o);
    }
    if !res.success {
        return 1;
    }
    if cmd == "run" {
        if cfg.is_lib {
            eprintln!("error: cannot run a library project");
            return 1;
        }
        let Some(exe) = res.executable.clone() else {
            eprintln!("error: cargo did not produce an executable");
            return 1;
        };
        let mut c = Command::new(exe);
        c.args(&run_args).current_dir(&cwd).stdin(Stdio::inherit());
        return match c.status() {
            Ok(s) => s.code().unwrap_or(1),
            Err(e) => {
                eprintln!("error: failed to run cargo: {}", e);
                1
            }
        };
    }
    0
}
