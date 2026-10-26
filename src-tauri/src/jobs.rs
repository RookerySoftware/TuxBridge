//! Long tasks (installing a distro, adding Hyprland) as jobs: a list of steps the window shows
//! with their state, and a log file with everything the tools printed, for bug reports.

use std::collections::BTreeMap;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncReadExt;
use tokio::process::Command;

use crate::paths;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum State {
    Pending,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub title: String,
    pub estimate: Option<String>,
    pub state: State,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobView {
    pub id: u64,
    /// "install", "hyprland", ...
    pub kind: String,
    pub distro: String,
    pub title: String,
    pub steps: Vec<Step>,
    pub state: State,
    pub error: Option<String>,
    pub log: String,
    pub started: u64,
    pub finished: Option<u64>,
    /// The last line any tool printed, shown under the running step.
    pub last_line: Option<String>,
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[derive(Default)]
pub struct Jobs {
    next: AtomicU64,
    list: Mutex<BTreeMap<u64, Arc<Mutex<JobView>>>>,
}

impl Jobs {
    pub fn all(&self) -> Vec<JobView> {
        self.list.lock().unwrap().values().map(|j| j.lock().unwrap().clone()).collect()
    }

    /// The running job for a distro, if any: one change at a time per distro.
    pub fn busy(&self, distro: &str) -> bool {
        self.all().iter().any(|j| j.distro.eq_ignore_ascii_case(distro) && j.state == State::Running)
    }

    pub fn dismiss(&self, id: u64) {
        let mut list = self.list.lock().unwrap();
        if list.get(&id).is_some_and(|j| j.lock().unwrap().state != State::Running) {
            list.remove(&id);
        }
    }
}

pub struct Job {
    app: AppHandle,
    view: Arc<Mutex<JobView>>,
    log: Mutex<std::fs::File>,
}

impl Job {
    pub fn new(app: &AppHandle, kind: &str, distro: &str, title: &str) -> anyhow::Result<Arc<Job>> {
        let jobs = app.state::<Jobs>();
        let id = jobs.next.fetch_add(1, Ordering::SeqCst) + 1;
        std::fs::create_dir_all(paths::logs_dir())?;
        let log_path = paths::logs_dir().join(format!("{kind}-{distro}-{}.log", now()));
        let mut file = std::fs::File::create(&log_path)?;
        writeln!(file, "EasyWSL {} {kind} {distro}", env!("CARGO_PKG_VERSION"))?;
        let view = Arc::new(Mutex::new(JobView {
            id,
            kind: kind.into(),
            distro: distro.into(),
            title: title.into(),
            steps: Vec::new(),
            state: State::Running,
            error: None,
            log: log_path.to_string_lossy().into_owned(),
            started: now(),
            finished: None,
            last_line: None,
        }));
        jobs.list.lock().unwrap().insert(id, view.clone());
        let job = Arc::new(Job { app: app.clone(), view, log: Mutex::new(file) });
        job.emit();
        Ok(job)
    }

    pub fn id(&self) -> u64 {
        self.view.lock().unwrap().id
    }

    fn emit(&self) {
        let view = self.view.lock().unwrap().clone();
        let _ = self.app.emit("job", view);
    }

    /// Lists the steps ahead, so the window can show the whole way from the start.
    pub fn plan(&self, steps: &[(&str, Option<&str>)]) {
        let mut view = self.view.lock().unwrap();
        view.steps = steps
            .iter()
            .map(|(t, e)| Step { title: t.to_string(), estimate: e.map(String::from), state: State::Pending })
            .collect();
        drop(view);
        self.emit();
    }

    /// Starts the step with this title (finishing the one before); added if not planned.
    pub fn step(&self, title: &str) {
        self.log_line(&format!("\n==> {title}"));
        let mut view = self.view.lock().unwrap();
        for s in view.steps.iter_mut() {
            if s.state == State::Running {
                s.state = State::Done;
            }
        }
        match view.steps.iter_mut().find(|s| s.title == title) {
            Some(s) => s.state = State::Running,
            None => view.steps.push(Step { title: title.into(), estimate: None, state: State::Running }),
        }
        view.last_line = None;
        drop(view);
        self.emit();
    }

    pub fn log_line(&self, line: &str) {
        let _ = writeln!(self.log.lock().unwrap(), "{line}");
    }

    fn output_line(&self, line: &str) {
        let line = line.trim_end();
        if line.is_empty() {
            return;
        }
        self.log_line(line);
        self.view.lock().unwrap().last_line = Some(line.chars().take(200).collect());
        self.emit();
    }

    /// Runs a command with its output in the log; fails with the exit code. `stdin` is written
    /// to the command and never logged (it carries passwords).
    pub async fn run(&self, mut cmd: Command, stdin: Option<&str>) -> anyhow::Result<()> {
        use std::process::Stdio;
        self.log_line(&format!("$ {:?}", cmd.as_std()));
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        if stdin.is_some() {
            cmd.stdin(Stdio::piped());
        }
        let mut child = cmd.spawn()?;
        if let (Some(text), Some(mut pipe)) = (stdin, child.stdin.take()) {
            use tokio::io::AsyncWriteExt;
            pipe.write_all(text.as_bytes()).await?;
            pipe.write_all(b"\n").await?;
            drop(pipe);
        }
        let out = child.stdout.take().unwrap();
        let err = child.stderr.take().unwrap();
        tokio::join!(self.pump(out), self.pump(err));
        let status = child.wait().await?;
        if !status.success() {
            let code = status.code().unwrap_or(-1);
            self.log_line(&format!("exit code {code}"));
            anyhow::bail!("exit code {code}");
        }
        Ok(())
    }

    /// Reads a pipe line by line; carriage returns (progress bars) end a line too.
    async fn pump(&self, mut pipe: impl tokio::io::AsyncRead + Unpin) {
        let mut buf = [0u8; 4096];
        let mut line = Vec::new();
        loop {
            match pipe.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    for &b in &buf[..n] {
                        match b {
                            b'\n' | b'\r' => {
                                self.output_line(&crate::wsl::decode(&line));
                                line.clear();
                            }
                            _ => line.push(b),
                        }
                    }
                }
            }
        }
        if !line.is_empty() {
            self.output_line(&crate::wsl::decode(&line));
        }
    }

    /// The last lines of the log, to recognise common causes of a failure.
    pub fn log_tail(&self) -> String {
        let path = self.view.lock().unwrap().log.clone();
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let lines: Vec<&str> = text.lines().collect();
        lines[lines.len().saturating_sub(40)..].join("\n")
    }

    pub fn finish(&self, result: &Result<(), String>) {
        let mut view = self.view.lock().unwrap();
        let failed = result.is_err();
        for s in view.steps.iter_mut() {
            if s.state == State::Running {
                s.state = if failed { State::Failed } else { State::Done };
            }
        }
        view.state = if failed { State::Failed } else { State::Done };
        view.error = result.clone().err();
        view.finished = Some(now());
        view.last_line = None;
        drop(view);
        if let Err(e) = result {
            self.log_line(&format!("\nFAILED: {e}"));
        } else {
            self.log_line("\nDONE");
        }
        self.emit();
    }
}
