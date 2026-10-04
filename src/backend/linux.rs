//! Linux: Canonical inference snaps.
//!
//! An inference snap packages one model with hardware-specific engines and
//! serves it through an OpenAI-compatible HTTP API on localhost. Snaps are
//! discovered on disk (`/snap/<name>/current/engines`) and their API URL is
//! read from `<name> status --format=json`, the documented way to find it.
//!
//! Discovery never installs a snap, starts or restarts a service, switches an
//! engine, or downloads a model: a stopped service is reported as not ready.

use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use serde_json::Value;

use super::{
    openai::{self, Service},
    worker::run_blocking,
    Backend, BackendSession,
};
use crate::{
    Availability, AvailabilityReason, BackendInfo, BackendKind, Capabilities, LocalAiError, Result,
};

/// Environment variable naming the inference snap to use when several are
/// installed (for example `qwen3`).
pub const INFERENCE_SNAP_ENV: &str = "RUST_LOCAL_AI_INFERENCE_SNAP";

const SNAP_ROOT: &str = "/snap";
const STATUS_TIMEOUT: Duration = Duration::from_secs(20);
const DOCS_URL: &str = "https://documentation.ubuntu.com/inference-snaps/";

pub(crate) fn platform_backend() -> Arc<dyn Backend> {
    backend(env::var(INFERENCE_SNAP_ENV).ok().filter(|v| !v.is_empty()))
}

/// The inference-snap backend, using `preferred` or the first usable snap.
pub(crate) fn backend(preferred: Option<String>) -> Arc<dyn Backend> {
    Arc::new(SnapBackend(Arc::new(InferenceSnapBackend {
        preferred,
        resolved: Mutex::new(None),
    })))
}

/// Uses the named inference snap, or the first suitable one when `None`.
struct InferenceSnapBackend {
    preferred: Option<String>,
    /// The last successful discovery. Failures are not cached, so a snap that
    /// is installed or started later is picked up.
    resolved: Mutex<Option<Resolved>>,
}

#[derive(Debug, Clone)]
struct Resolved {
    snap: String,
    service: Service,
}

impl InferenceSnapBackend {
    /// Discovers (or re-validates) the snap to use. Blocking.
    fn resolve(&self) -> std::result::Result<Resolved, Availability> {
        if let Some(resolved) = self.resolved.lock().expect("resolve lock").clone() {
            match resolved.service.probe() {
                Ok(_) => return Ok(resolved),
                Err(_) => *self.resolved.lock().expect("resolve lock") = None,
            }
        }
        let resolved = discover(Path::new(SNAP_ROOT), self.preferred.as_deref(), &|name| {
            snap_status(name)
        })?;
        *self.resolved.lock().expect("resolve lock") = Some(resolved.clone());
        Ok(resolved)
    }

    async fn resolve_async(
        self: &Arc<Self>,
    ) -> Result<std::result::Result<Resolved, Availability>> {
        let this = self.clone();
        run_blocking(BackendKind::UbuntuInferenceSnap, move || Ok(this.resolve())).await
    }
}

/// Holds the state in an `Arc` so discovery can run on a worker thread.
struct SnapBackend(Arc<InferenceSnapBackend>);

#[async_trait]
impl Backend for SnapBackend {
    fn info(&self) -> BackendInfo {
        let snap = self
            .0
            .resolved
            .lock()
            .expect("resolve lock")
            .as_ref()
            .map(|r| r.snap.clone())
            .or_else(|| self.0.preferred.clone());
        BackendInfo {
            kind: BackendKind::UbuntuInferenceSnap,
            name: match snap {
                Some(snap) => format!("Ubuntu inference snap ({snap})"),
                None => "Ubuntu inference snaps".into(),
            },
            system_managed_model: true,
        }
    }

    async fn availability(&self) -> Availability {
        match self.0.resolve_async().await {
            Ok(Ok(resolved)) => Availability::available().with_detail(format!(
                "{} serving at {}",
                resolved.snap, resolved.service.url
            )),
            Ok(Err(availability)) => availability,
            Err(error) => Availability::unavailable(AvailabilityReason::BackendFailure)
                .with_detail(error.to_string()),
        }
    }

    async fn capabilities(&self) -> Capabilities {
        openai::capabilities(true)
    }

    async fn open_session(&self, instructions: Option<&str>) -> Result<Arc<dyn BackendSession>> {
        let resolved =
            self.0
                .resolve_async()
                .await?
                .map_err(|availability| LocalAiError::Unavailable {
                    reason: availability
                        .reason
                        .unwrap_or(AvailabilityReason::BackendFailure),
                })?;
        openai::open_session(
            resolved.service,
            BackendKind::UbuntuInferenceSnap,
            instructions,
        )
        .await
    }
}

/// The status a snap reports, reduced to what discovery needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SnapStatus {
    openai_url: Option<String>,
    /// `true` when every reported service is active.
    services_active: bool,
}

pub(crate) fn parse_status(stdout: &str) -> std::result::Result<SnapStatus, String> {
    // The CLI prints JSON, but be tolerant of anything around it.
    let start = stdout.find('{').ok_or("status printed no JSON")?;
    let end = stdout.rfind('}').ok_or("status printed no JSON")?;
    let value: Value = serde_json::from_str(&stdout[start..=end])
        .map_err(|e| format!("status JSON is invalid: {e}"))?;
    let services = value["services"].as_object();
    Ok(SnapStatus {
        openai_url: value["entrypoints"]["openai"]["url"]
            .as_str()
            .filter(|url| !url.is_empty())
            .map(str::to_owned),
        services_active: services.is_some_and(|services| {
            !services.is_empty() && services.values().all(|s| s.as_str() == Some("active"))
        }),
    })
}

/// Lists installed inference snaps: snaps that ship an `engines` directory.
pub(crate) fn installed_snaps(root: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().join("current").join("engines").is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| is_snap_name(name))
        .collect();
    names.sort();
    names
}

/// Snap names are lowercase letters, digits and hyphens. Checking keeps any
/// odd directory name out of the command line.
fn is_snap_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 40
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn discover(
    root: &Path,
    preferred: Option<&str>,
    status: &dyn Fn(&str) -> std::result::Result<SnapStatus, String>,
) -> std::result::Result<Resolved, Availability> {
    let installed = installed_snaps(root);
    let candidates: Vec<&str> = match preferred {
        Some(name) if installed.iter().any(|s| s == name) => vec![name],
        Some(name) => {
            return Err(
                Availability::unavailable(AvailabilityReason::ProviderNotInstalled).with_detail(
                    format!(
                        "inference snap {name:?} (from {INFERENCE_SNAP_ENV}) is not installed; \
                         installed: {installed:?}"
                    ),
                ),
            )
        }
        None => installed.iter().map(String::as_str).collect(),
    };
    if candidates.is_empty() {
        return Err(
            Availability::unavailable(AvailabilityReason::ProviderNotInstalled).with_detail(
                format!("no Ubuntu inference snap is installed; see {DOCS_URL}"),
            ),
        );
    }

    let mut problems = Vec::new();
    for snap in candidates {
        let status = match status(snap) {
            Ok(status) => status,
            Err(error) => {
                problems.push(format!("{snap}: {error}"));
                continue;
            }
        };
        let Some(url) = status.openai_url else {
            problems.push(format!("{snap}: no OpenAI-compatible API"));
            continue;
        };
        if !status.services_active {
            problems.push(format!(
                "{snap}: service is not running (start it with `sudo snap start {snap}`)"
            ));
            continue;
        }
        let service = match Service::new(&url, None) {
            Ok(service) => service,
            Err(error) => {
                problems.push(format!("{snap}: {error}"));
                continue;
            }
        };
        match service.probe() {
            Ok(_) => {
                return Ok(Resolved {
                    snap: snap.to_owned(),
                    service,
                })
            }
            Err(error) => problems.push(format!("{snap}: {error}")),
        }
    }
    Err(Availability::unavailable(AvailabilityReason::ModelNotReady)
        .with_detail(problems.join("; ")))
}

/// Runs `<snap> status --format=json` with a timeout. Blocking.
fn snap_status(name: &str) -> std::result::Result<SnapStatus, String> {
    let program = PathBuf::from(SNAP_ROOT).join("bin").join(name);
    let mut child = Command::new(&program)
        .args(["status", "--format=json"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("cannot run {}: {e}", program.display()))?;
    let mut stdout = child.stdout.take().expect("stdout is piped");
    let reader = thread::spawn(move || {
        let mut output = String::new();
        let _ = (&mut stdout).take(1024 * 1024).read_to_string(&mut output);
        output
    });
    let deadline = Instant::now() + STATUS_TIMEOUT;
    let exit = loop {
        match child.try_wait() {
            Ok(Some(exit)) => break exit,
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(50)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("`{name} status` timed out"));
            }
            Err(e) => return Err(format!("waiting for `{name} status` failed: {e}")),
        }
    };
    let output = reader.join().unwrap_or_default();
    if !exit.success() {
        return Err(format!("`{name} status` failed ({exit})"));
    }
    parse_status(&output)
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
    };

    use super::*;

    const STATUS: &str = r#"{
  "engine": "intel-cpu",
  "services": {"server": "active"},
  "entrypoints": {"openai": {"url": "http://localhost:8338/v1"}},
  "model": {"name": "qwen3-4b"}
}"#;

    #[test]
    fn parses_cli_status() {
        assert_eq!(
            parse_status(STATUS).unwrap(),
            SnapStatus {
                openai_url: Some("http://localhost:8338/v1".into()),
                services_active: true,
            }
        );
        let stopped = parse_status(
            r#"Getting status {"engine":"x","services":{"server":"inactive"},"entrypoints":{}}"#,
        )
        .unwrap();
        assert_eq!(stopped.openai_url, None);
        assert!(!stopped.services_active);
        assert!(!parse_status(r#"{"services":{}}"#).unwrap().services_active);
        assert!(parse_status("error: no active engine").is_err());
    }

    fn fake_snap_root(snaps: &[&str]) -> PathBuf {
        let root = env::temp_dir().join(format!(
            "rust_local_ai-snaps-{}-{}",
            std::process::id(),
            snaps.join("-")
        ));
        let _ = fs::remove_dir_all(&root);
        for snap in snaps {
            fs::create_dir_all(root.join(snap).join("current").join("engines")).unwrap();
        }
        // Ordinary snaps and odd names are ignored.
        fs::create_dir_all(root.join("firefox").join("current")).unwrap();
        fs::create_dir_all(root.join("Bad Name").join("current").join("engines")).unwrap();
        root
    }

    fn models_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/v1", listener.local_addr().unwrap());
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                while reader.read_line(&mut line).unwrap_or(0) > 2 {
                    line.clear();
                }
                let body = r#"{"object":"list","data":[{"id":"qwen3-4b"}]}"#;
                let _ = write!(
                    &stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                );
            }
        });
        url
    }

    #[test]
    fn lists_only_inference_snaps() {
        let root = fake_snap_root(&["qwen3", "gemma3"]);
        assert_eq!(installed_snaps(&root), vec!["gemma3", "qwen3"]);
        assert!(installed_snaps(Path::new("/definitely/not/here")).is_empty());
    }

    #[test]
    fn discovery_reports_why_nothing_is_usable() {
        let empty = fake_snap_root(&[]);
        let err = discover(&empty, None, &|_| unreachable!()).unwrap_err();
        assert_eq!(err.reason, Some(AvailabilityReason::ProviderNotInstalled));

        let root = fake_snap_root(&["deepseek-r1", "qwen3"]);
        let err = discover(&root, None, &|_| {
            Ok(SnapStatus {
                openai_url: Some("http://localhost:8338/v1".into()),
                services_active: false,
            })
        })
        .unwrap_err();
        assert_eq!(err.reason, Some(AvailabilityReason::ModelNotReady));
        let detail = err.detail.unwrap();
        assert!(detail.contains("sudo snap start deepseek-r1"), "{detail}");
        assert!(detail.contains("sudo snap start qwen3"), "{detail}");

        let err = discover(&root, Some("gemma3"), &|_| unreachable!()).unwrap_err();
        assert_eq!(err.reason, Some(AvailabilityReason::ProviderNotInstalled));
    }

    #[test]
    fn discovery_picks_a_running_snap_and_honours_preference() {
        let root = fake_snap_root(&["deepseek-r1", "qwen3", "smollm2"]);
        let url = models_server();
        let status = |name: &str| -> std::result::Result<SnapStatus, String> {
            match name {
                "deepseek-r1" => Err("`deepseek-r1 status` failed".into()),
                "qwen3" | "smollm2" => Ok(SnapStatus {
                    openai_url: Some(url.clone()),
                    services_active: true,
                }),
                _ => unreachable!(),
            }
        };
        assert_eq!(discover(&root, None, &status).unwrap().snap, "qwen3");
        assert_eq!(
            discover(&root, Some("smollm2"), &status).unwrap().snap,
            "smollm2"
        );
        let err = discover(&root, Some("deepseek-r1"), &status).unwrap_err();
        assert!(err.detail.unwrap().contains("deepseek-r1"));
    }
}
