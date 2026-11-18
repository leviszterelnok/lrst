//! rust-analyzer backend for the LSP server: finds (or downloads) the binary,
//! keeps one long-lived instance per project on the generated crate, and runs
//! jobs against it on a worker thread so the main loop never blocks on it.

use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Pinned rust-analyzer release used when no `rust-analyzer` is on PATH.
pub const RA_TAG: &str = "2025-01-06";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// A unit of work for the worker. `None` means rust-analyzer is unavailable,
/// and the job must then answer with its fallback.
pub type Job = Box<dyn FnOnce(Option<&mut Conn>) + Send>;

/// Handle to one worker thread (and so one rust-analyzer process).
pub struct Ra {
    tx: Sender<Job>,
}

impl Ra {
    /// Start a worker for the crate at `root`. Nothing is spawned until the first job.
    pub fn start(root: PathBuf) -> Ra {
        let (tx, rx) = channel::<Job>();
        std::thread::spawn(move || {
            let mut conn: Option<Conn> = None;
            let mut failed = false;
            for job in rx {
                if conn.is_none() && !failed {
                    match find_binary().and_then(|bin| Conn::spawn(&bin, &root)) {
                        Ok(c) => conn = Some(c),
                        Err(e) => {
                            eprintln!("lrs: rust-analyzer unavailable: {e}");
                            failed = true;
                        }
                    }
                }
                job(conn.as_mut());
            }
        });
        Ra { tx }
    }

    pub fn run(&self, job: Job) {
        let _ = self.tx.send(job);
    }
}

/// A running rust-analyzer process and its JSON-RPC plumbing.
pub struct Conn {
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    responses: Receiver<Value>,
    next_id: i64,
    /// document uri -> last sent version
    versions: HashMap<String, i64>,
    progress: Arc<Mutex<Progress>>,
    warmed: bool,
}

/// rust-analyzer's work-done progress: finished once every token has ended.
#[derive(Default)]
struct Progress {
    seen: bool,
    open: HashSet<String>,
}

const WARMUP: Duration = Duration::from_secs(60);

impl Conn {
    fn spawn(bin: &Path, root: &Path) -> Result<Conn, String> {
        let mut child = Command::new(bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("cannot start {}: {e}", bin.display()))?;
        let stdin = Arc::new(Mutex::new(child.stdin.take().ok_or("no stdin")?));
        let stdout = child.stdout.take().ok_or("no stdout")?;
        let (tx, responses) = channel::<Value>();
        let reply_stdin = stdin.clone();
        let progress = Arc::new(Mutex::new(Progress::default()));
        let reader_progress = progress.clone();
        std::thread::spawn(move || {
            let mut r = BufReader::new(stdout);
            while let Some(msg) = read_message(&mut r) {
                match (msg.get("id").cloned(), msg.get("method").and_then(Value::as_str)) {
                    (None, Some("$/progress")) => {
                        let token = msg["params"]["token"].to_string();
                        if let Ok(mut p) = reader_progress.lock() {
                            match msg["params"]["value"]["kind"].as_str() {
                                Some("begin") => {
                                    p.open.insert(token);
                                }
                                Some("end") => {
                                    p.open.remove(&token);
                                    p.seen = true;
                                }
                                _ => {}
                            }
                        }
                    }
                    // a request from the server: answer it so it never blocks on us
                    (Some(id), Some(method)) => {
                        let result = if method == "workspace/configuration" {
                            let n = msg["params"]["items"].as_array().map_or(0, Vec::len);
                            Value::Array(vec![json!({}); n])
                        } else {
                            Value::Null
                        };
                        write_message(&reply_stdin, &json!({"jsonrpc": "2.0", "id": id, "result": result}));
                    }
                    // a response to one of our requests
                    (Some(_), None) => {
                        let _ = tx.send(msg);
                    }
                    // notifications (diagnostics, progress) are not needed
                    _ => {}
                }
            }
        });
        let mut conn = Conn { child, stdin, responses, next_id: 1, versions: HashMap::new(), progress, warmed: false };
        let root_uri = crate::lsp::path_to_uri(root);
        conn.request(
            "initialize",
            json!({
                "processId": null,
                "rootUri": root_uri,
                "capabilities": {
                    // progress reports tell us when the initial indexing is done
                    "window": {"workDoneProgress": true},
                    "textDocument": {
                        "completion": {
                            "completionItem": {
                                "snippetSupport": false,
                                "documentationFormat": ["markdown", "plaintext"],
                                "resolveSupport": {"properties": ["documentation", "detail"]}
                            }
                        }
                    }
                },
                // our own rustc diagnostics already cover checking
                "initializationOptions": {"checkOnSave": false}
            }),
        )
        .ok_or("rust-analyzer did not answer initialize")?;
        conn.notify("initialized", json!({}));
        Ok(conn)
    }

    /// Send a request and wait for its response (the result, or `None` on error/timeout).
    pub fn request(&mut self, method: &str, params: Value) -> Option<Value> {
        let id = self.next_id;
        self.next_id += 1;
        write_message(&self.stdin, &json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        loop {
            match self.responses.recv_timeout(REQUEST_TIMEOUT) {
                Ok(msg) if msg["id"].as_i64() == Some(id) => return msg.get("result").cloned().filter(|r| !r.is_null()),
                Ok(_) => continue,
                Err(RecvTimeoutError::Timeout) | Err(RecvTimeoutError::Disconnected) => return None,
            }
        }
    }

    /// The first request waits (up to `WARMUP`) until rust-analyzer finished its initial indexing.
    pub fn wait_warm(&mut self) {
        if self.warmed {
            return;
        }
        let end = std::time::Instant::now() + WARMUP;
        while std::time::Instant::now() < end {
            if let Ok(p) = self.progress.lock() {
                if p.seen && p.open.is_empty() {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        self.warmed = true;
    }

    pub fn notify(&mut self, method: &str, params: Value) {
        write_message(&self.stdin, &json!({"jsonrpc": "2.0", "method": method, "params": params}));
    }

    /// Make rust-analyzer see `text` as the content of `path` (open first, then full-text changes).
    pub fn sync(&mut self, path: &Path, text: &str) {
        let uri = crate::lsp::path_to_uri(path);
        match self.versions.get_mut(&uri) {
            Some(v) => {
                *v += 1;
                let version = *v;
                self.notify(
                    "textDocument/didChange",
                    json!({"textDocument": {"uri": uri, "version": version}, "contentChanges": [{"text": text}]}),
                );
            }
            None => {
                self.versions.insert(uri.clone(), 1);
                self.notify(
                    "textDocument/didOpen",
                    json!({"textDocument": {"uri": uri, "languageId": "rust", "version": 1, "text": text}}),
                );
            }
        }
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn write_message(stdin: &Arc<Mutex<ChildStdin>>, v: &Value) {
    let body = v.to_string();
    if let Ok(mut s) = stdin.lock() {
        let _ = write!(s, "Content-Length: {}\r\n\r\n{}", body.len(), body);
        let _ = s.flush();
    }
}

fn read_message(r: &mut impl BufRead) -> Option<Value> {
    let mut len = None;
    loop {
        let mut line = String::new();
        if r.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(v) = line.strip_prefix("Content-Length:") {
            len = v.trim().parse::<usize>().ok();
        }
    }
    let mut buf = vec![0u8; len?];
    r.read_exact(&mut buf).ok()?;
    serde_json::from_slice(&buf).ok()
}

/// The rust-analyzer binary: PATH first, then a cached copy of the pinned release,
/// downloaded from GitHub on first use.
pub fn find_binary() -> Result<PathBuf, String> {
    // a rustup proxy without the component is on PATH but fails: check that it actually runs
    if let Some(p) = which("rust-analyzer") {
        let runs = Command::new(&p)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if runs {
            return Ok(p);
        }
    }
    let cache_dir = cache_root().join("lrs");
    let bin = cache_dir.join(format!("rust-analyzer-{RA_TAG}"));
    if bin.is_file() {
        return Ok(bin);
    }
    let target = if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "aarch64-unknown-linux-gnu"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else {
        return Err("no rust-analyzer download for this platform; install it on PATH".into());
    };
    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    let url = format!("https://github.com/rust-lang/rust-analyzer/releases/download/{RA_TAG}/rust-analyzer-{target}.gz");
    let tmp = cache_dir.join("download.tmp");
    eprintln!("lrs: downloading rust-analyzer {RA_TAG} to {}", bin.display());
    let ok = Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(cache_dir.join("download.gz"))
        .arg(&url)
        .status()
        .map_err(|e| format!("curl: {e}"))?
        .success();
    if !ok {
        return Err(format!("download failed: {url}"));
    }
    let gunzipped = Command::new("gzip")
        .arg("-dc")
        .arg(cache_dir.join("download.gz"))
        .stdout(std::fs::File::create(&tmp).map_err(|e| e.to_string())?)
        .status()
        .map_err(|e| format!("gzip: {e}"))?;
    let _ = std::fs::remove_file(cache_dir.join("download.gz"));
    if !gunzipped.success() {
        return Err("cannot unpack rust-analyzer".into());
    }
    make_executable(&tmp)?;
    std::fs::rename(&tmp, &bin).map_err(|e| e.to_string())?;
    Ok(bin)
}

fn cache_root() -> PathBuf {
    if let Some(x) = std::env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(x);
    }
    match std::env::var_os("HOME") {
        Some(h) => PathBuf::from(h).join(".cache"),
        None => std::env::temp_dir(),
    }
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).map(|d| d.join(name)).find(|p| p.is_file())
}

#[cfg(unix)]
fn make_executable(p: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())
}

#[cfg(not(unix))]
fn make_executable(_p: &Path) -> Result<(), String> {
    Ok(())
}
