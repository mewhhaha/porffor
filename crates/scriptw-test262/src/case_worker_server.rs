//! Bounded, sequential requests share immutable compiler caches, never a Realm.

use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use super::case_process::{CaseDeadline, CaseProcess};
use super::{ConformanceRunVerdict, RunConfig, SuiteConfig, TestExecutionId};

pub(super) const CASE_LIMIT: u64 = 64;
pub(super) const FRAME_LIMIT: usize = 16 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    sequence: u64,
    test_id: TestExecutionId,
    snapshot_dir: PathBuf,
    snapshot_name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Outcome {
    Passed,
    Failed,
    WorkerFailure,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reply {
    sequence: u64,
    test_id: TestExecutionId,
    outcome: Outcome,
}

fn frame<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    if bytes.len() > FRAME_LIMIT {
        return Err("case worker protocol frame exceeds its byte limit".into());
    }
    Ok(bytes)
}

fn read_request(reader: &mut impl BufRead) -> Result<Option<Request>, String> {
    let mut bytes = Vec::new();
    reader
        .take((FRAME_LIMIT + 1) as u64)
        .read_until(b'\n', &mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() > FRAME_LIMIT || bytes.last() != Some(&b'\n') {
        return Err("case worker request is oversized or unterminated".into());
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|error| format!("invalid case worker request: {error}"))
}

/// The CLI owns stdin/stdout; user-program output remains typed host events.
pub(super) fn serve(config: SuiteConfig, run: RunConfig) -> Result<(), String> {
    if run.filter.is_some()
        || run.resume
        || run.shard_index != 0
        || run.shard_count != 1
        || run.max_matrix_nodes.is_some()
        || config.worker_count != 1
    {
        return Err("case worker server requires fresh, unsharded single-case requests".into());
    }
    let mut input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();
    for sequence in 1..=CASE_LIMIT {
        let Some(request) = read_request(&mut input)? else {
            return Ok(());
        };
        if request.sequence != sequence {
            return Err("case worker request sequence is out of order".into());
        }
        let mut case_config = config.clone();
        case_config.snapshot_dir = request.snapshot_dir;
        case_config.reuse_case_workers = true;
        let mut case_run = run.clone();
        case_run.snapshot_name = request.snapshot_name;
        case_run.filter = Some(request.test_id.wire_key());
        // This existing entry discovers exactly one ID and creates/join a new
        // execution thread. The Engine, Realm, Store, roots and agents belong
        // only to this request and are gone before its snapshot is acknowledged.
        let summary = super::run_case_worker(case_config, case_run)?;
        let outcome = match summary.verdict()? {
            ConformanceRunVerdict::Passed { total } if total.get() == 1 => Outcome::Passed,
            ConformanceRunVerdict::Failed { total, failed }
                if total.get() == 1 && failed.get() == 1 =>
            {
                let failure = &summary.failures[0];
                let suspect = match failure.kind {
                    super::FailureKind::HostHarness | super::FailureKind::WasmBackend => true,
                    super::FailureKind::Parser
                    | super::FailureKind::EarlyError
                    | super::FailureKind::Lowering
                    | super::FailureKind::Runtime
                    | super::FailureKind::Unsupported => false,
                };
                if suspect
                    || failure.outcome == super::OutcomeKind::Crash
                    || !summary.timeouts.is_empty()
                {
                    Outcome::WorkerFailure
                } else {
                    Outcome::Failed
                }
            }
            _ => return Err("case worker response requires exactly one completed case".into()),
        };
        let failed = outcome == Outcome::WorkerFailure;
        output
            .write_all(&frame(&Reply {
                sequence,
                test_id: request.test_id,
                outcome,
            })?)
            .and_then(|()| output.flush())
            .map_err(|error| error.to_string())?;
        if failed {
            return Ok(());
        }
    }
    Ok(())
}

struct Session {
    process: CaseProcess,
    completed: u64,
}

/// The pool slot owns the live session and all reuse/retirement decisions.
/// Callers cannot reuse a process after a malformed, late or exhausted reply.
#[derive(Default)]
pub(super) struct Worker {
    session: Option<Session>,
}

pub(super) enum RequestCompletion {
    Completed {
        outcome: Outcome,
        retirement: Result<(), String>,
    },
    TimedOut {
        retirement: Result<(), String>,
    },
    Failed {
        primary: String,
        retirement: Result<(), String>,
    },
}

impl Worker {
    pub(super) fn request(
        &mut self,
        binary: &Path,
        config: &SuiteConfig,
        run: &RunConfig,
        test_id: &TestExecutionId,
        snapshot_dir: &Path,
        snapshot_name: &str,
        deadline: CaseDeadline,
    ) -> RequestCompletion {
        let result = (|| {
            if self.session.is_none() {
                self.session = Some(Session::spawn(binary, config, run, deadline)?);
            }
            self.session
                .as_mut()
                .expect("session admitted above")
                .request(test_id, snapshot_dir, snapshot_name, deadline)
        })();
        match result {
            Err(primary) => RequestCompletion::Failed {
                primary,
                retirement: self.retire(),
            },
            Ok(None) => RequestCompletion::TimedOut {
                retirement: self.retire(),
            },
            Ok(Some(outcome)) => {
                let retirement = if outcome == Outcome::WorkerFailure
                    || self
                        .session
                        .as_ref()
                        .expect("completed request owns session")
                        .exhausted()
                {
                    self.retire()
                } else {
                    Ok(())
                };
                RequestCompletion::Completed {
                    outcome,
                    retirement,
                }
            }
        }
    }

    pub(super) fn retire(&mut self) -> Result<(), String> {
        match self.session.take() {
            Some(mut session) => session.retire(),
            None => Ok(()),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        // Explicit retirement reports errors; unwinding still owns cleanup.
        let _ = self.retire();
    }
}

impl Session {
    pub(super) fn spawn(
        binary: &Path,
        config: &SuiteConfig,
        run: &RunConfig,
        deadline: CaseDeadline,
    ) -> Result<Self, String> {
        let mut command = Command::new(binary);
        command
            .arg("--jobs")
            .arg(scriptw_engine::compilation_jobs().to_string())
            .arg("test262")
            .arg("__case-worker-server-v1")
            .arg("--suite-root")
            .arg(&config.suite_root)
            .arg("--case-suite-output")
            .arg(config.source_output_root())
            .arg("--threads")
            .arg("1")
            .arg("--timeout-ms")
            .arg(config.timeout_ms.to_string())
            .arg("--execution-backend")
            .arg(run.execution_backend.as_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        super::append_case_harness(&mut command, &config.local_harness);
        let process = CaseProcess::spawn(&mut command, deadline)?;
        Ok(Self {
            process,
            completed: 0,
        })
    }

    pub(super) fn request(
        &mut self,
        test_id: &TestExecutionId,
        snapshot_dir: &Path,
        snapshot_name: &str,
        deadline: CaseDeadline,
    ) -> Result<Option<Outcome>, String> {
        if self.completed >= CASE_LIMIT {
            return Err("case worker lifetime is exhausted".into());
        }
        let sequence = self.completed + 1;
        let request = frame(&Request {
            sequence,
            test_id: test_id.clone(),
            snapshot_dir: snapshot_dir.to_owned(),
            snapshot_name: snapshot_name.to_owned(),
        })?;
        let Some(bytes) = self.process.exchange(&request, deadline, FRAME_LIMIT)? else {
            return Ok(None);
        };
        let reply: Reply = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid case worker response: {error}"))?;
        if reply.sequence != sequence || reply.test_id != *test_id {
            return Err("case worker response does not match the admitted request".into());
        }
        self.completed = sequence;
        Ok(Some(reply.outcome))
    }

    pub(super) fn exhausted(&self) -> bool {
        self.completed == CASE_LIMIT
    }

    pub(super) fn retire(&mut self) -> Result<(), String> {
        self.process.retire()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn requests_reject_unknown_fields_oversize_and_truncation() {
        assert!(read_request(&mut Cursor::new(b"{\"extra\":true}\n")).is_err());
        assert!(read_request(&mut Cursor::new(vec![b'x'; FRAME_LIMIT + 2])).is_err());
        assert!(read_request(&mut Cursor::new(b"{}")).is_err());
        assert!(read_request(&mut Cursor::new(b"")).unwrap().is_none());
    }

    #[test]
    #[cfg(unix)]
    fn pool_retires_bad_timed_out_and_exhausted_sessions_before_reuse() {
        let id = TestExecutionId::parse_wire_key("raw-script:language/a.js").unwrap();
        let config = SuiteConfig::default();
        for (reply, completed, expected) in [
            ("not JSON".to_owned(), 0, "failure"),
            (String::new(), 0, "timeout"),
            (
                format!(
                    r#"{{"sequence":{CASE_LIMIT},"test_id":"raw-script:language/a.js","outcome":"passed"}}"#
                ),
                CASE_LIMIT - 1,
                "passed",
            ),
            (
                r#"{"sequence":1,"test_id":"raw-script:language/a.js","outcome":"worker-failure"}"#
                    .into(),
                0,
                "worker-failure",
            ),
        ] {
            let mut command = Command::new("sh");
            command
                .arg("-c")
                .arg("read line; if [ -n \"$1\" ]; then printf '%s\\n' \"$1\"; fi; read another")
                .arg("worker")
                .arg(reply)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped());
            let deadline =
                CaseDeadline::new(500, scriptw_engine::ExecutionBackend::SpecExec).unwrap();
            let mut worker = Worker {
                session: Some(Session {
                    process: CaseProcess::spawn(&mut command, deadline).unwrap(),
                    completed,
                }),
            };
            let run = RunConfig::default();
            match worker.request(
                Path::new("absent-worker"),
                &config,
                &run,
                &id,
                Path::new("snapshot"),
                "case",
                deadline,
            ) {
                RequestCompletion::Failed { retirement, .. } => {
                    assert_eq!(expected, "failure");
                    retirement.unwrap();
                }
                RequestCompletion::TimedOut { retirement } => {
                    assert_eq!(expected, "timeout");
                    retirement.unwrap();
                }
                RequestCompletion::Completed {
                    outcome,
                    retirement,
                } => {
                    retirement.unwrap();
                    assert_eq!(
                        match outcome {
                            Outcome::Passed => "passed",
                            Outcome::WorkerFailure => "worker-failure",
                            Outcome::Failed => "failed",
                        },
                        expected
                    );
                }
            }
            assert!(
                worker.session.is_none(),
                "{expected} session was kept reusable"
            );
            // A new request must spawn: the deliberately absent binary must
            // fail immediately rather than write to the former process.
            match worker.request(
                Path::new("absent-worker"),
                &config,
                &run,
                &id,
                Path::new("snapshot"),
                "case",
                CaseDeadline::new(500, scriptw_engine::ExecutionBackend::SpecExec).unwrap(),
            ) {
                RequestCompletion::Failed {
                    primary,
                    retirement,
                } => {
                    assert!(!primary.is_empty());
                    retirement.unwrap();
                }
                _ => panic!("retired process was reused"),
            }
        }
    }

    #[test]
    #[cfg(unix)]
    fn terminal_replies_must_match_both_sequence_and_execution_id() {
        for reply in [
            r#"{"sequence":2,"test_id":"raw-script:language/a.js","outcome":"passed"}"#,
            r#"{"sequence":1,"test_id":"raw-script:language/b.js","outcome":"passed"}"#,
        ] {
            let mut command = Command::new("sh");
            command
                .arg("-c")
                .arg("read line; printf '%s\\n' \"$1\"")
                .arg("worker")
                .arg(reply)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped());
            let deadline =
                CaseDeadline::new(500, scriptw_engine::ExecutionBackend::SpecExec).unwrap();
            let mut session = Session {
                process: CaseProcess::spawn(&mut command, deadline).unwrap(),
                completed: 0,
            };
            let id = TestExecutionId::parse_wire_key("raw-script:language/a.js").unwrap();
            let error = session
                .request(&id, Path::new("snapshot"), "case", deadline)
                .unwrap_err();
            assert!(error.contains("does not match"), "{error}");
            assert_eq!(session.completed, 0);
            session.retire().unwrap();
        }
    }

    #[test]
    #[cfg(unix)]
    fn process_crash_cannot_publish_a_terminal_pass() {
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg("read line; exit 7")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        let deadline = CaseDeadline::new(500, scriptw_engine::ExecutionBackend::SpecExec).unwrap();
        let mut session = Session {
            process: CaseProcess::spawn(&mut command, deadline).unwrap(),
            completed: 0,
        };
        let id = TestExecutionId::parse_wire_key("raw-script:language/a.js").unwrap();
        assert!(session
            .request(&id, Path::new("snapshot"), "case", deadline)
            .unwrap_err()
            .contains("crashed"));
        assert_eq!(session.completed, 0);
        session.retire().unwrap();
    }
}
