//! Drives `lrs lsp` over stdio against the demo project.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

struct Client {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<Value>,
    next_id: u64,
}

impl Client {
    fn start() -> Client {
        let mut child = Command::new(env!("CARGO_BIN_EXE_lrs"))
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let mut out = BufReader::new(child.stdout.take().unwrap());
        let (tx, rx) = channel();
        std::thread::spawn(move || loop {
            let mut len = 0usize;
            loop {
                let mut line = String::new();
                if out.read_line(&mut line).unwrap_or(0) == 0 {
                    return;
                }
                let l = line.trim();
                if l.is_empty() {
                    break;
                }
                if let Some(v) = l.strip_prefix("Content-Length:") {
                    len = v.trim().parse().unwrap();
                }
            }
            let mut buf = vec![0; len];
            if out.read_exact(&mut buf).is_err() {
                return;
            }
            if tx.send(serde_json::from_slice(&buf).unwrap()).is_err() {
                return;
            }
        });
        Client { child, stdin, rx, next_id: 1 }
    }

    fn send(&mut self, v: Value) {
        let body = v.to_string();
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
        self.stdin.flush().unwrap();
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.send(json!({"jsonrpc": "2.0", "method": method, "params": params}));
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        loop {
            let m = self.rx.recv_timeout(Duration::from_secs(20)).expect("timed out waiting for response");
            if m["id"] == json!(id) {
                return m["result"].clone();
            }
        }
    }

    /// Wait for a publishDiagnostics for `uri_suffix` satisfying `pred`.
    fn wait_diag(&mut self, uri_suffix: &str, pred: impl Fn(&Vec<Value>) -> bool) -> Vec<Value> {
        loop {
            let m = self.rx.recv_timeout(Duration::from_secs(60)).expect("timed out waiting for diagnostics");
            if m["method"] == "textDocument/publishDiagnostics" && m["params"]["uri"].as_str().unwrap().ends_with(uri_suffix) {
                let d = m["params"]["diagnostics"].as_array().unwrap().clone();
                if pred(&d) {
                    return d;
                }
            }
        }
    }
}

fn pos_of(text: &str, needle: &str, extra: usize) -> Value {
    let off = text.find(needle).unwrap() + extra;
    let line = text[..off].matches('\n').count();
    let col = off - text[..off].rfind('\n').map_or(0, |i| i + 1);
    json!({"line": line, "character": col})
}

fn uri(p: &std::path::Path) -> String {
    format!("file://{}", p.display())
}

#[test]
fn lsp_features() {
    let demo = std::fs::canonicalize(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/demo")).unwrap();
    let main = demo.join("main.lrs");
    let text = std::fs::read_to_string(&main).unwrap();
    let main_uri = uri(&main);

    let mut c = Client::start();
    let init = c.request("initialize", json!({"processId": null, "rootUri": uri(&demo), "capabilities": {}}));
    assert_eq!(init["capabilities"]["definitionProvider"], json!(true));
    c.notify("initialized", json!({}));
    c.notify("textDocument/didOpen", json!({"textDocument": {"uri": main_uri, "languageId": "lrs", "version": 1, "text": text}}));
    let d = c.wait_diag("main.lrs", |_| true);
    assert!(d.is_empty(), "demo should have no lrs diagnostics: {:?}", d);

    // go to definition: .Graphics.Core.Shape::square  ->  graphics/core.lrs
    let def = c.request(
        "textDocument/definition",
        json!({"textDocument": {"uri": main_uri}, "position": pos_of(&text, ".Graphics.Core.Shape::square", 16)}),
    );
    let loc = &def[0];
    assert!(loc["uri"].as_str().unwrap().ends_with("graphics/core.lrs"), "{}", def);
    let core = std::fs::read_to_string(demo.join("graphics/core.lrs")).unwrap();
    assert_eq!(loc["range"]["start"]["line"], pos_of(&core, "pub enum Shape", 9)["line"]);

    // go to definition on an `include` string
    let def = c.request(
        "textDocument/definition",
        json!({"textDocument": {"uri": main_uri}, "position": pos_of(&text, "\"utils/math.lrs\"", 3)}),
    );
    assert!(def[0]["uri"].as_str().unwrap().ends_with("utils/math.lrs"), "{}", def);

    // hover
    let h = c.request("textDocument/hover", json!({"textDocument": {"uri": main_uri}, "position": pos_of(&text, ".Utils.Math.PI", 13)}));
    assert!(h["contents"]["value"].as_str().unwrap().contains("pub const PI: f64"), "{}", h);

    // hover on a namespace segment
    let h = c.request("textDocument/hover", json!({"textDocument": {"uri": main_uri}, "position": pos_of(&text, ".Utils.Math.PI", 3)}));
    assert!(h["contents"]["value"].as_str().unwrap().contains("namespace .Utils"), "{}", h);

    // completion after `.Utils.Math.`
    let edited = text.replacen("fn main() {", "fn main() {\n    let _z = .Utils.Math.;", 1);
    c.notify("textDocument/didChange", json!({"textDocument": {"uri": main_uri, "version": 2}, "contentChanges": [{"text": edited}]}));
    let comp = c.request(
        "textDocument/completion",
        json!({"textDocument": {"uri": main_uri}, "position": pos_of(&edited, "let _z = .Utils.Math.", 21)}),
    );
    let labels: Vec<&str> = comp["items"].as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap()).collect();
    assert!(labels.contains(&"sqr") && labels.contains(&"lerp") && labels.contains(&"PI"), "{:?}", labels);
    assert!(!labels.contains(&"private_helper"), "{:?}", labels);

    // completion at the root: namespaces
    let edited2 = text.replacen("fn main() {", "fn main() {\n    let _z = .;", 1);
    c.notify("textDocument/didChange", json!({"textDocument": {"uri": main_uri, "version": 3}, "contentChanges": [{"text": edited2}]}));
    let comp = c.request("textDocument/completion", json!({"textDocument": {"uri": main_uri}, "position": pos_of(&edited2, "let _z = .", 10)}));
    let labels: Vec<&str> = comp["items"].as_array().unwrap().iter().map(|i| i["label"].as_str().unwrap()).collect();
    assert!(labels.contains(&"Utils") && labels.contains(&"Graphics") && labels.contains(&"main"), "{:?}", labels);

    // document symbols of a file that was never opened
    let math = demo.join("utils/math.lrs");
    let syms = c.request("textDocument/documentSymbol", json!({"textDocument": {"uri": uri(&math)}}));
    let names: Vec<&str> = syms.as_array().unwrap().iter().map(|s| s["name"].as_str().unwrap()).collect();
    assert!(names.contains(&".Utils.Math") && names.contains(&"sqr") && names.contains(&"clamp"), "{:?}", names);

    // diagnostics for a bad include and a bad path
    let broken = format!("include \"missing.lrs\"\n{}\nfn broken() {{ let _ = .Nope.x(); }}\n", text);
    c.notify("textDocument/didChange", json!({"textDocument": {"uri": main_uri, "version": 4}, "contentChanges": [{"text": broken}]}));
    let d = c.wait_diag("main.lrs", |d| d.len() >= 2);
    let msgs: Vec<&str> = d.iter().map(|x| x["message"].as_str().unwrap()).collect();
    assert!(msgs.iter().any(|m| m.contains("cannot include")), "{:?}", msgs);
    assert!(msgs.iter().any(|m| m.contains("cannot find `Nope`")), "{:?}", msgs);

    // save a file with a type error -> `cargo check` runs and the rustc error is mapped back to .lrs
    let bad = format!("{}\nfn bad_types() {{ let _x: i32 = \"s\"; }}\n", text);
    c.notify("textDocument/didChange", json!({"textDocument": {"uri": main_uri, "version": 5}, "contentChanges": [{"text": bad}]}));
    c.notify("textDocument/didSave", json!({"textDocument": {"uri": main_uri}}));
    let d = c.wait_diag("main.lrs", |d| d.iter().any(|x| x["source"] == "rustc"));
    let e = d.iter().find(|x| x["source"] == "rustc" && x["message"].as_str().unwrap().contains("mismatched types")).expect("mismatched types");
    let want = bad.lines().position(|l| l.contains("bad_types")).unwrap();
    assert_eq!(e["range"]["start"]["line"], json!(want));
    assert_eq!(e["range"]["start"]["character"], json!("fn bad_types() { let _x: i32 = ".len()));

    let _ = c.request("shutdown", json!(null));
    c.notify("exit", json!(null));
    let status = c.child.wait().unwrap();
    assert!(status.success());
}
