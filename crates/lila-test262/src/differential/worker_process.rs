//! Parent-only bounded process lifecycle and validating journal consumer.

use super::*;
use std::io::{self, Write};
pub(super) const MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;
#[cfg(feature = "spec-exec-oracle")]
pub(super) const MAX_FRAME_BYTES: usize = 1024 * 1024;
#[cfg(feature = "spec-exec-oracle")]
pub(super) const MAX_JOURNAL_BYTES: usize = 16 * 1024 * 1024;

struct LimitedWriter<'a, W> {
    output: &'a mut W,
    written: usize,
}
impl<W: Write> Write for LimitedWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > MAX_REQUEST_BYTES.saturating_sub(self.written) {
            return Err(io::Error::other(
                "differential worker request budget exceeded",
            ));
        }
        let written = self.output.write(bytes)?;
        self.written += written;
        Ok(written)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}
pub(super) fn write_bounded_json(
    value: &impl Serialize,
    output: &mut impl Write,
) -> io::Result<()> {
    serde_json::to_writer_pretty(LimitedWriter { output, written: 0 }, value)
        .map_err(io::Error::other)
}

/// Mandatory selected executable, not a current-test-image heuristic.
#[derive(Debug)]
pub struct DifferentialWorkerRunner {
    #[cfg(feature = "spec-exec-oracle")]
    executable: PathBuf,
    #[cfg(feature = "spec-exec-oracle")]
    selected_image: lila_engine::CompilerDigest,
    #[cfg(feature = "spec-exec-oracle")]
    parent_identity: super::super::CompilerProvenance,
}

impl DifferentialWorkerRunner {
    pub fn new(executable: impl Into<PathBuf>) -> Result<Self, DifferentialError> {
        let executable = executable.into();
        if executable.as_os_str().is_empty() {
            return Err(DifferentialError::WorkerConfiguration(
                "worker executable path is empty".into(),
            ));
        }
        #[cfg(feature = "spec-exec-oracle")]
        {
            use sha2::{Digest, Sha256};
            use std::io::Read;
            let executable = fs::canonicalize(executable)
                .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
            let mut image = fs::File::open(&executable)
                .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
            let before = image
                .metadata()
                .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
            let mut hash = Sha256::new();
            let mut block = [0u8; 64 * 1024];
            loop {
                let length = image
                    .read(&mut block)
                    .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
                if length == 0 {
                    break;
                }
                hash.update(&block[..length]);
            }
            let after = image
                .metadata()
                .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
            let before_modified = before
                .modified()
                .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
            let after_modified = after
                .modified()
                .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
            if before.len() != after.len() || before_modified != after_modified {
                return Err(DifferentialError::WorkerConfiguration(
                    "selected worker image changed while hashing".into(),
                ));
            }
            let spelling: String = hash
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            return Ok(Self {
                executable,
                selected_image: lila_engine::CompilerDigest::parse(&spelling)
                    .map_err(DifferentialError::WorkerConfiguration)?,
                parent_identity: super::super::CompilerProvenance::current()
                    .map_err(DifferentialError::WorkerConfiguration)?,
            });
        }
        #[cfg(not(feature = "spec-exec-oracle"))]
        {
            Ok(Self {})
        }
    }
}

#[cfg(feature = "spec-exec-oracle")]
use super::worker::{
    RequestProjection, WorkerBinding, WorkerFrame, BUDGET_EXIT, JOURNAL_EXIT, PROTOCOL_VERSION,
};
#[cfg(feature = "spec-exec-oracle")]
use std::io::Read;
#[cfg(all(feature = "spec-exec-oracle", unix))]
use std::os::unix::process::{CommandExt, ExitStatusExt};
#[cfg(feature = "spec-exec-oracle")]
use std::process::{Child, Command, ExitStatus, Stdio};
#[cfg(feature = "spec-exec-oracle")]
use std::time::{Duration, Instant};

/// Only the completed supervisor/decoder can mint this non-cloneable result.
#[cfg(feature = "spec-exec-oracle")]
pub(super) struct CompletedWorkerAttempt {
    observation: BackendObservation,
}
#[cfg(feature = "spec-exec-oracle")]
impl CompletedWorkerAttempt {
    pub(super) fn into_observation(self) -> BackendObservation {
        self.observation
    }
}

#[cfg(feature = "spec-exec-oracle")]
struct Stage {
    directory: PathBuf,
    token: String,
}
#[cfg(all(feature = "spec-exec-oracle", unix))]
pub(super) struct TransportAttempt {
    pub(super) token: String,
    pub(super) bytes: Vec<u8>,
    pub(super) failure: Option<DifferentialWorkerFailure>,
    pub(super) cleanup: Option<String>,
    pub(super) stderr: Vec<u8>,
    pub(super) evidence_complete: bool,
}
#[cfg(feature = "spec-exec-oracle")]
impl Stage {
    fn new() -> io::Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        loop {
            let token = format!(
                "{}-{timestamp}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            );
            let directory = std::env::temp_dir().join(format!("lila-differential-{token}"));
            match fs::create_dir(&directory) {
                Ok(()) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Err(error) =
                            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
                        {
                            let _ = fs::remove_dir(&directory);
                            return Err(error);
                        }
                    }
                    return Ok(Self { directory, token });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
    }
    fn finish(self) -> Option<String> {
        fs::remove_dir_all(&self.directory)
            .err()
            .map(|error| format!("worker staging cleanup: {error}"))
    }
}
#[cfg(feature = "spec-exec-oracle")]
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

#[cfg(all(feature = "spec-exec-oracle", unix))]
struct LiveWorker {
    child: Child,
    group: Option<i32>,
    retired: bool,
}
#[cfg(all(feature = "spec-exec-oracle", unix))]
impl LiveWorker {
    fn retire(&mut self) -> Option<String> {
        if self.retired {
            return None;
        }
        self.retired = true;
        let mut errors = Vec::new();
        // Kill the group even when the direct child has exited: descendants
        // cannot retain this journal or survive a completed attempt.
        if let Some(group) = self.group {
            let result = unsafe { libc::kill(-group, libc::SIGKILL) };
            if result != 0 {
                let error = io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ESRCH) {
                    errors.push(format!("worker group termination: {error}"));
                }
            }
        }
        let cleanup_deadline = Instant::now() + Duration::from_secs(2);
        let mut direct_signalled = false;
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => break, // try_wait reaps the direct child.
                Ok(None) => {
                    if !direct_signalled {
                        if let Err(error) = self.child.kill() {
                            if error.raw_os_error() != Some(libc::ESRCH) {
                                errors.push(format!("worker direct termination: {error}"));
                            }
                        }
                        direct_signalled = true;
                    }
                    if Instant::now() >= cleanup_deadline {
                        errors
                            .push("worker could not be reaped within the cleanup deadline".into());
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(2));
                }
                Err(error) => {
                    errors.push(format!("worker reap: {error}"));
                    break;
                }
            }
        }
        (!errors.is_empty()).then(|| errors.join("; "))
    }
}
#[cfg(all(feature = "spec-exec-oracle", unix))]
impl Drop for LiveWorker {
    fn drop(&mut self) {
        if !self.retired {
            let _ = self.retire();
        }
    }
}

#[cfg(feature = "spec-exec-oracle")]
enum JournalTail {
    Missing,
    Completed(ExecutionObservation),
    AdmissionRejected(String),
    Invalid(String),
}
#[cfg(feature = "spec-exec-oracle")]
struct DecodedJournal {
    identity: Option<super::super::CompilerProvenance>,
    events: Vec<String>,
    tail: JournalTail,
}
#[cfg(feature = "spec-exec-oracle")]
enum JournalPhase {
    Header,
    Admission,
    Printing,
    Terminal,
}

#[cfg(feature = "spec-exec-oracle")]
impl DifferentialWorkerRunner {
    pub(super) fn identity_matches(&self, identity: &super::super::CompilerProvenance) -> bool {
        let actual = identity.identity();
        let expected = self.parent_identity.identity();
        actual.source_fingerprint() == expected.source_fingerprint()
            && actual.source_revision() == expected.source_revision()
            && actual.executable_sha256() == self.selected_image
    }

    fn decode_journal(
        &self,
        bytes: &[u8],
        binding: &WorkerBinding,
        input: &DifferentialReplayInput,
    ) -> DecodedJournal {
        let mut decoded = DecodedJournal {
            identity: None,
            events: Vec::new(),
            tail: JournalTail::Missing,
        };
        let mut phase = JournalPhase::Header;
        let complete_length = bytes
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |index| index + 1);
        let complete = &bytes[..complete_length];
        for line in complete
            .strip_suffix(b"\n")
            .unwrap_or(complete)
            .split(|byte| *byte == b'\n')
        {
            if complete.is_empty() {
                break;
            }
            if line.is_empty() {
                decoded.tail = JournalTail::Invalid("empty committed journal frame".into());
                return decoded;
            }
            if line.len() > MAX_FRAME_BYTES {
                decoded.tail =
                    JournalTail::Invalid("journal frame exceeds observation budget".into());
                return decoded;
            }
            let frame = match serde_json::from_slice::<WorkerFrame>(line) {
                Ok(frame) => frame,
                Err(error) => {
                    decoded.tail = JournalTail::Invalid(format!("invalid journal frame: {error}"));
                    return decoded;
                }
            };
            match (phase, frame) {
                (
                    JournalPhase::Header,
                    WorkerFrame::Header {
                        version,
                        binding: actual,
                        compiler_identity,
                    },
                ) if version == PROTOCOL_VERSION
                    && actual == *binding
                    && self.identity_matches(&compiler_identity) =>
                {
                    decoded.identity = Some(compiler_identity);
                    phase = JournalPhase::Admission;
                }
                (JournalPhase::Admission, WorkerFrame::Admitted { binding: actual })
                    if actual == *binding =>
                {
                    phase = JournalPhase::Printing;
                }
                (JournalPhase::Admission, WorkerFrame::AdmissionRejected { message }) => {
                    decoded.tail = JournalTail::AdmissionRejected(message);
                    phase = JournalPhase::Terminal;
                }
                (JournalPhase::Printing, WorkerFrame::PrintLine { sequence, text })
                    if sequence == decoded.events.len() as u64 =>
                {
                    decoded.events.push(text);
                    phase = JournalPhase::Printing;
                }
                (
                    JournalPhase::Printing,
                    WorkerFrame::Terminal {
                        print_count,
                        execution,
                    },
                ) if print_count == decoded.events.len() as u64
                    && valid_terminal(input.protocol(), &execution)
                    && rooted_snapshot::terminal_limits_match(input, &execution) =>
                {
                    decoded.tail = JournalTail::Completed(execution);
                    phase = JournalPhase::Terminal;
                }
                _ => {
                    decoded.tail = JournalTail::Invalid(
                        "journal identity, ordering, count or result domain mismatch".into(),
                    );
                    return decoded;
                }
            }
        }
        if complete_length != bytes.len() {
            // A torn frame is not a committed print. An abnormal process exit
            // retains only the complete validated prefix; clean exit needs EOF.
            if matches!(
                decoded.tail,
                JournalTail::Completed(_) | JournalTail::AdmissionRejected(_)
            ) {
                decoded.tail = JournalTail::Invalid("bytes follow the terminal frame".into());
            }
        }
        decoded
    }

    #[cfg(not(unix))]
    pub(super) fn run(
        &self,
        _input: &DifferentialReplayInput,
        backend: DifferentialBackend,
        _oracle: SpecExecOracle,
    ) -> Result<CompletedWorkerAttempt, DifferentialError> {
        Ok(worker_failure(
            backend,
            DecodedJournal {
                identity: None,
                events: Vec::new(),
                tail: JournalTail::Missing,
            },
            DifferentialWorkerFailure::UnsupportedPlatform,
            None,
        ))
    }

    #[cfg(unix)]
    pub(super) fn run(
        &self,
        input: &DifferentialReplayInput,
        backend: DifferentialBackend,
        _oracle: SpecExecOracle,
    ) -> Result<CompletedWorkerAttempt, DifferentialError> {
        let transport = self.run_transport(input.timeout_ms().get(), |token, request| {
            let binding = WorkerBinding::new(input, backend, token.into());
            write_bounded_json(
                &RequestProjection {
                    version: PROTOCOL_VERSION,
                    binding: &binding,
                    corpus: input,
                },
                request,
            )
        })?;
        let binding = WorkerBinding::new(input, backend, transport.token);
        if std::env::var_os("LILA_WASM_TRACE").is_some() {
            // The transport has already retired the worker and bounded these
            // bytes. Relay diagnostics before dropping them; they are never
            // admitted as JavaScript print events or used to change a verdict.
            let mut stderr = io::stderr().lock();
            let _ = writeln!(
                stderr,
                "lila differential trace: backend={backend:?} case={} fingerprint={} evidence_complete={} stderr_bytes={}",
                binding.case_id,
                binding.case_fingerprint,
                transport.evidence_complete,
                transport.stderr.len(),
            );
            let _ = stderr.write_all(&transport.stderr);
            if !transport.stderr.is_empty() && !transport.stderr.ends_with(b"\n") {
                let _ = stderr.write_all(b"\n");
            }
        }
        let decoded = self.decode_journal(&transport.bytes, &binding, input);
        if let Some(failure) = transport.failure {
            return Ok(worker_failure(backend, decoded, failure, transport.cleanup));
        }
        if let Some(message) = transport.cleanup {
            return Ok(worker_failure(
                backend,
                decoded,
                DifferentialWorkerFailure::Process {
                    message: "worker cleanup failed".into(),
                },
                Some(message),
            ));
        }
        match decoded.tail {
            JournalTail::Completed(execution) => Ok(CompletedWorkerAttempt {
                observation: BackendObservation {
                    worker_identity: decoded.identity,
                    backend,
                    output_events: OutputEventsObservation::Captured {
                        events: decoded.events,
                    },
                    execution,
                },
            }),
            JournalTail::AdmissionRejected(message) => {
                Err(DifferentialError::InvalidCorpus(message))
            }
            JournalTail::Missing => Ok(worker_failure(
                backend,
                decoded,
                DifferentialWorkerFailure::Protocol {
                    message: "worker exited without a completed journal".into(),
                },
                None,
            )),
            JournalTail::Invalid(ref message) => {
                let failure = DifferentialWorkerFailure::Protocol {
                    message: message.clone(),
                };
                Ok(worker_failure(backend, decoded, failure, None))
            }
        }
    }

    /// One physical spawn/deadline/retirement/IO implementation for both
    /// developer request domains. Consumers validate their own closed frames.
    #[cfg(unix)]
    pub(super) fn run_transport(
        &self,
        timeout_ms: u64,
        write_request: impl FnOnce(&str, &mut fs::File) -> io::Result<()>,
    ) -> Result<TransportAttempt, DifferentialError> {
        let stage = Stage::new()
            .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
        let request_path = stage.directory.join("request.json");
        let journal_path = stage.directory.join("journal.jsonl");
        let stderr_path = stage.directory.join("stderr.bin");
        let stderr = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stderr_path)
            .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
        let mut request = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&request_path)
            .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
        write_request(&stage.token, &mut request)
            .and_then(|()| request.flush())
            .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
        drop(request);
        let start = Instant::now();
        let deadline = start
            .checked_add(Duration::from_millis(timeout_ms))
            .ok_or_else(|| {
                DifferentialError::WorkerConfiguration(
                    "worker deadline overflows the platform clock".into(),
                )
            })?;
        let mut command = Command::new(&self.executable);
        command
            .args(["--jobs", "1", "differential", "__worker", "--request"])
            .arg(&request_path)
            .arg("--journal")
            .arg(&journal_path)
            .args(["--oracle", "spec-exec"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(stderr))
            .process_group(0);
        let child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                let token = stage.token.clone();
                let cleanup = stage.finish();
                return Ok(TransportAttempt {
                    token,
                    bytes: Vec::new(),
                    failure: Some(DifferentialWorkerFailure::Process {
                        message: format!("worker spawn: {error}"),
                    }),
                    cleanup,
                    stderr: Vec::new(),
                    evidence_complete: false,
                });
            }
        };
        let group = i32::try_from(child.id()).ok().filter(|group| *group > 0);
        let mut live = LiveWorker {
            child,
            group,
            retired: false,
        };
        let mut failure = group.is_none().then(|| DifferentialWorkerFailure::Process {
            message: "worker process id cannot name a positive group".into(),
        });
        let status: Option<ExitStatus> = loop {
            if failure.is_some() {
                break None;
            }
            if Instant::now() >= deadline {
                failure = Some(DifferentialWorkerFailure::Timeout { timeout_ms });
                break None;
            }
            match fs::metadata(&stderr_path) {
                Ok(metadata) if metadata.len() > MAX_JOURNAL_BYTES as u64 => {
                    failure = Some(DifferentialWorkerFailure::ObservationLimit {
                        limit_bytes: MAX_JOURNAL_BYTES as u64,
                    });
                    break None;
                }
                Ok(_) => {}
                Err(error) => {
                    failure = Some(DifferentialWorkerFailure::Process {
                        message: format!("worker stderr metadata: {error}"),
                    });
                    break None;
                }
            }
            match fs::metadata(&journal_path) {
                Ok(metadata) if metadata.len() > MAX_JOURNAL_BYTES as u64 => {
                    failure = Some(DifferentialWorkerFailure::ObservationLimit {
                        limit_bytes: MAX_JOURNAL_BYTES as u64,
                    });
                    break None;
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    failure = Some(DifferentialWorkerFailure::Process {
                        message: format!("journal metadata: {error}"),
                    });
                    break None;
                }
            }
            match live.child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) => std::thread::sleep(Duration::from_millis(2)),
                Err(error) => {
                    failure = Some(DifferentialWorkerFailure::Process {
                        message: format!("worker poll: {error}"),
                    });
                    break None;
                }
            }
        };
        let mut cleanup = live.retire();
        drop(live);
        let mut evidence_complete = true;
        let bytes = match fs::File::open(&journal_path) {
            Ok(file) => {
                let mut bytes = Vec::new();
                if let Err(error) = file
                    .take(MAX_JOURNAL_BYTES as u64 + 1)
                    .read_to_end(&mut bytes)
                {
                    evidence_complete = false;
                    failure.get_or_insert(DifferentialWorkerFailure::Process {
                        message: format!("journal read: {error}"),
                    });
                }
                if bytes.len() > MAX_JOURNAL_BYTES {
                    evidence_complete = false;
                    failure.get_or_insert(DifferentialWorkerFailure::ObservationLimit {
                        limit_bytes: MAX_JOURNAL_BYTES as u64,
                    });
                    bytes.truncate(MAX_JOURNAL_BYTES);
                }
                bytes
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(error) => {
                evidence_complete = false;
                failure.get_or_insert(DifferentialWorkerFailure::Process {
                    message: format!("journal open: {error}"),
                });
                Vec::new()
            }
        };
        let token = stage.token.clone();
        let mut stderr = Vec::new();
        match fs::File::open(&stderr_path).and_then(|file| {
            file.take(MAX_JOURNAL_BYTES as u64 + 1)
                .read_to_end(&mut stderr)
        }) {
            Ok(_) => {
                if stderr.len() > MAX_JOURNAL_BYTES {
                    evidence_complete = false;
                    failure.get_or_insert(DifferentialWorkerFailure::ObservationLimit {
                        limit_bytes: MAX_JOURNAL_BYTES as u64,
                    });
                    stderr.truncate(MAX_JOURNAL_BYTES);
                }
            }
            Err(error) => {
                evidence_complete = false;
                failure.get_or_insert(DifferentialWorkerFailure::Process {
                    message: format!("worker stderr read: {error}"),
                });
            }
        }
        if let Some(error) = stage.finish() {
            cleanup = Some(match cleanup {
                Some(prior) => format!("{prior}; {error}"),
                None => error,
            });
        }
        if let Some(status) = status {
            if !status.success() {
                failure.get_or_insert_with(|| match status.code() {
                    Some(BUDGET_EXIT) => DifferentialWorkerFailure::ObservationLimit {
                        limit_bytes: MAX_JOURNAL_BYTES as u64,
                    },
                    Some(JOURNAL_EXIT) => DifferentialWorkerFailure::Protocol {
                        message: "worker journal write failed".into(),
                    },
                    code => DifferentialWorkerFailure::Exit {
                        code,
                        signal: status.signal(),
                    },
                });
            }
        }
        Ok(TransportAttempt {
            token,
            bytes,
            failure,
            cleanup,
            stderr,
            evidence_complete,
        })
    }

    pub(super) fn run_robustness(
        &self,
        input: &super::robustness::RobustnessInput,
        _oracle: SpecExecOracle,
    ) -> Result<super::robustness::RobustnessObservation, DifferentialError> {
        use super::robustness::{RobustnessObservation, RobustnessResult};
        #[cfg(not(unix))]
        {
            Ok(RobustnessObservation {
                input: input.clone(),
                compiler_identity: None,
                stages: Vec::new(),
                result: RobustnessResult::WorkerFailure {
                    failure: DifferentialWorkerFailure::UnsupportedPlatform,
                    cleanup_error: None,
                },
                journal_bytes_hex: String::new(),
                stderr_bytes_hex: String::new(),
                journal_error: None,
                interrupted_stage: None,
            })
        }
        #[cfg(unix)]
        {
            use super::robustness::worker::{Binding, RequestProjection as RobustnessRequest};
            let transport = self.run_transport(input.timeout_ms(), |token, file| {
                let binding = Binding::new(input, token.into());
                write_bounded_json(
                    &RobustnessRequest {
                        version: PROTOCOL_VERSION,
                        robustness_binding: &binding,
                        robustness: input,
                    },
                    file,
                )
            })?;
            let binding = Binding::new(input, transport.token);
            let decoded =
                self.decode_robustness_journal(&transport.bytes, &binding, input.target());
            let interrupted_stage =
                decoded.interrupted_stage(&transport.bytes, transport.evidence_complete);
            let result = if let Some(failure) = transport.failure {
                RobustnessResult::WorkerFailure {
                    failure,
                    cleanup_error: transport.cleanup,
                }
            } else if let Some(message) = transport.cleanup {
                RobustnessResult::WorkerFailure {
                    failure: DifferentialWorkerFailure::Process {
                        message: "worker cleanup failed".into(),
                    },
                    cleanup_error: Some(message),
                }
            } else if let Some(message) = &decoded.invalid {
                RobustnessResult::WorkerFailure {
                    failure: DifferentialWorkerFailure::Protocol {
                        message: message.clone(),
                    },
                    cleanup_error: None,
                }
            } else if let Some(result) = decoded.result {
                result
            } else {
                RobustnessResult::WorkerFailure {
                    failure: DifferentialWorkerFailure::Protocol {
                        message: "worker exited without a completed robustness journal".into(),
                    },
                    cleanup_error: None,
                }
            };
            Ok(RobustnessObservation {
                input: input.clone(),
                compiler_identity: decoded.identity,
                stages: decoded.stages,
                result,
                journal_error: decoded.invalid,
                interrupted_stage,
                stderr_bytes_hex: transport
                    .stderr
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect(),
                journal_bytes_hex: transport
                    .bytes
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect(),
            })
        }
    }

    fn decode_robustness_journal(
        &self,
        bytes: &[u8],
        binding: &super::robustness::worker::Binding,
        target: super::robustness::RobustnessTarget,
    ) -> DecodedRobustnessJournal {
        let mut decoded = DecodedRobustnessJournal {
            identity: None,
            stages: Vec::new(),
            result: None,
            invalid: None,
        };
        let complete_length = bytes
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |index| index + 1);
        let complete = &bytes[..complete_length];
        for line in complete
            .strip_suffix(b"\n")
            .unwrap_or(complete)
            .split(|byte| *byte == b'\n')
        {
            if complete.is_empty() {
                break;
            }
            if line.is_empty() || line.len() > MAX_FRAME_BYTES {
                decoded.invalid = Some("empty or oversized robustness frame".into());
                break;
            }
            let frame = match serde_json::from_slice::<WorkerFrame>(line) {
                Ok(frame) => frame,
                Err(error) => {
                    decoded.invalid = Some(format!("invalid robustness frame: {error}"));
                    break;
                }
            };
            match frame {
                WorkerFrame::RobustnessHeader {
                    version,
                    binding: actual,
                    compiler_identity,
                } if decoded.identity.is_none()
                    && decoded.stages.is_empty()
                    && decoded.result.is_none()
                    && version == PROTOCOL_VERSION
                    && actual == *binding
                    && self.identity_matches(&compiler_identity) =>
                {
                    decoded.identity = Some(compiler_identity)
                }
                WorkerFrame::RobustnessStage { stage }
                    if decoded.identity.is_some()
                        && decoded.result.is_none()
                        && target.stages().get(decoded.stages.len()) == Some(&stage) =>
                {
                    decoded.stages.push(stage)
                }
                WorkerFrame::RobustnessTerminal { result }
                    if decoded.identity.is_some()
                        && decoded.result.is_none()
                        && result.valid_terminal(target, &decoded.stages) =>
                {
                    decoded.result = Some(result)
                }
                _ => {
                    decoded.invalid =
                        Some("robustness identity, stage order or terminal domain mismatch".into());
                    break;
                }
            }
        }
        if complete_length != bytes.len() && decoded.result.is_some() {
            decoded.invalid = Some("bytes follow the robustness terminal frame".into());
        }
        decoded
    }
}

#[cfg(feature = "spec-exec-oracle")]
struct DecodedRobustnessJournal {
    identity: Option<crate::CompilerProvenance>,
    stages: Vec<super::robustness::RobustnessStage>,
    result: Option<super::robustness::RobustnessResult>,
    invalid: Option<String>,
}
#[cfg(feature = "spec-exec-oracle")]
impl DecodedRobustnessJournal {
    fn interrupted_stage(
        &self,
        bytes: &[u8],
        evidence_complete: bool,
    ) -> Option<super::robustness::RobustnessStage> {
        if evidence_complete
            && self.identity.is_some()
            && self.invalid.is_none()
            && self.result.is_none()
            && bytes.ends_with(b"\n")
        {
            self.stages.last().copied()
        } else {
            None
        }
    }
}

#[cfg(all(test, feature = "spec-exec-oracle"))]
fn empty_journal() -> DecodedJournal {
    DecodedJournal {
        identity: None,
        events: Vec::new(),
        tail: JournalTail::Missing,
    }
}
#[cfg(feature = "spec-exec-oracle")]
fn worker_failure(
    backend: DifferentialBackend,
    decoded: DecodedJournal,
    failure: DifferentialWorkerFailure,
    cleanup_error: Option<String>,
) -> CompletedWorkerAttempt {
    CompletedWorkerAttempt {
        observation: BackendObservation {
            worker_identity: decoded.identity,
            backend,
            output_events: OutputEventsObservation::Incomplete {
                events: decoded.events,
            },
            execution: ExecutionObservation::WorkerFailure {
                failure,
                cleanup_error,
            },
        },
    }
}

#[cfg(feature = "spec-exec-oracle")]
pub(super) fn valid_terminal(
    protocol: DifferentialProtocol,
    execution: &ExecutionObservation,
) -> bool {
    match (protocol, execution) {
        (
            DifferentialProtocol::V7Test262HostRootedCompletionGraphPrintTranscript,
            ExecutionObservation::RootedCompletionGraph { .. }
            | ExecutionObservation::EngineFailure { .. },
        ) => true,
        (
            DifferentialProtocol::V7Test262HostRootedCompletionGraphPrintTranscript,
            ExecutionObservation::Normal { .. }
            | ExecutionObservation::Error { .. }
            | ExecutionObservation::PrimitiveCompletion { .. }
            | ExecutionObservation::SelectedObjectProbe { .. }
            | ExecutionObservation::ObservationRejected { .. }
            | ExecutionObservation::UnsupportedCompletion { .. },
        ) => false,
        (
            DifferentialProtocol::V1SelfCheckingNoOutput
            | DifferentialProtocol::V2PrimitiveCompletionNoOutput
            | DifferentialProtocol::V3PrimitiveCompletionPrintTranscript
            | DifferentialProtocol::V4EmbeddedGraphPrimitivePrintTranscript
            | DifferentialProtocol::V5SelectedObjectProbePrintTranscript
            | DifferentialProtocol::V6Test262HostPrimitivePrintTranscript,
            ExecutionObservation::RootedCompletionGraph { .. },
        ) => false,
        (
            DifferentialProtocol::V5SelectedObjectProbePrintTranscript,
            ExecutionObservation::SelectedObjectProbe { .. }
            | ExecutionObservation::ObservationRejected { .. }
            | ExecutionObservation::EngineFailure { .. },
        ) => true,
        (
            DifferentialProtocol::V5SelectedObjectProbePrintTranscript,
            ExecutionObservation::Normal { .. }
            | ExecutionObservation::Error { .. }
            | ExecutionObservation::PrimitiveCompletion { .. }
            | ExecutionObservation::UnsupportedCompletion { .. },
        ) => false,
        (
            DifferentialProtocol::V1SelfCheckingNoOutput
            | DifferentialProtocol::V2PrimitiveCompletionNoOutput
            | DifferentialProtocol::V3PrimitiveCompletionPrintTranscript
            | DifferentialProtocol::V4EmbeddedGraphPrimitivePrintTranscript
            | DifferentialProtocol::V6Test262HostPrimitivePrintTranscript,
            ExecutionObservation::SelectedObjectProbe { .. }
            | ExecutionObservation::ObservationRejected { .. },
        ) => false,
        (
            DifferentialProtocol::V1SelfCheckingNoOutput,
            ExecutionObservation::Normal { .. } | ExecutionObservation::Error { .. },
        ) => true,
        (
            DifferentialProtocol::V2PrimitiveCompletionNoOutput
            | DifferentialProtocol::V3PrimitiveCompletionPrintTranscript
            | DifferentialProtocol::V4EmbeddedGraphPrimitivePrintTranscript
            | DifferentialProtocol::V6Test262HostPrimitivePrintTranscript,
            ExecutionObservation::PrimitiveCompletion { completion, .. },
        ) => match completion.value() {
            PrimitiveValueObservation::Number { bits } => {
                bits.len() == 16
                    && bits
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                    && u64::from_str_radix(bits, 16)
                        .is_ok_and(|raw| lila_engine::ObservedNumber::from_bits(raw).bits() == raw)
            }
            PrimitiveValueObservation::BigInt { decimal } => {
                lila_engine::ObservedBigInt::parse_canonical_decimal(
                    decimal.clone().into_boxed_str(),
                )
                .is_ok()
            }
            PrimitiveValueObservation::Undefined
            | PrimitiveValueObservation::Null
            | PrimitiveValueObservation::Boolean { .. }
            | PrimitiveValueObservation::String { .. } => true,
        },
        (
            DifferentialProtocol::V2PrimitiveCompletionNoOutput
            | DifferentialProtocol::V3PrimitiveCompletionPrintTranscript
            | DifferentialProtocol::V4EmbeddedGraphPrimitivePrintTranscript
            | DifferentialProtocol::V6Test262HostPrimitivePrintTranscript,
            ExecutionObservation::UnsupportedCompletion { .. }
            | ExecutionObservation::EngineFailure { .. },
        ) => true,
        (
            DifferentialProtocol::V1SelfCheckingNoOutput,
            ExecutionObservation::PrimitiveCompletion { .. }
            | ExecutionObservation::UnsupportedCompletion { .. }
            | ExecutionObservation::EngineFailure { .. },
        ) => false,
        (
            DifferentialProtocol::V2PrimitiveCompletionNoOutput
            | DifferentialProtocol::V3PrimitiveCompletionPrintTranscript
            | DifferentialProtocol::V4EmbeddedGraphPrimitivePrintTranscript
            | DifferentialProtocol::V6Test262HostPrimitivePrintTranscript,
            ExecutionObservation::Normal { .. } | ExecutionObservation::Error { .. },
        ) => false,
        (_, ExecutionObservation::WorkerFailure { .. }) => false,
    }
}

#[cfg(all(test, feature = "spec-exec-oracle"))]
mod tests {
    use super::*;

    #[test]
    fn robustness_frames_bind_exact_bytes_target_and_each_actual_stage() {
        use super::super::robustness::worker::Binding;
        use super::super::robustness::{
            RobustnessInput, RobustnessRejectionPhase, RobustnessResult, RobustnessStage,
            RobustnessTarget,
        };
        let (runner, _, _) = fixture();
        let input = RobustnessInput::new(
            "robustness/protocol",
            RobustnessTarget::Compiler {
                goal: DifferentialGoal::Script,
            },
            1000,
            b"let = ;".to_vec(),
        )
        .unwrap();
        let binding = Binding::new(&input, "unique-request".into());
        let frames = || {
            vec![
                WorkerFrame::RobustnessHeader {
                    version: PROTOCOL_VERSION,
                    binding: binding.clone(),
                    compiler_identity: runner.parent_identity.clone(),
                },
                WorkerFrame::RobustnessStage {
                    stage: RobustnessStage::Decode,
                },
                WorkerFrame::RobustnessStage {
                    stage: RobustnessStage::Preparation,
                },
                WorkerFrame::RobustnessTerminal {
                    result: RobustnessResult::Rejected {
                        stage: RobustnessStage::Preparation,
                        phase: RobustnessRejectionPhase::Parse,
                        message: "original parser rejection".into(),
                    },
                },
            ]
        };
        let complete =
            runner.decode_robustness_journal(&bytes(&frames()), &binding, input.target());
        assert!(complete.result.is_some());
        assert!(complete.invalid.is_none());
        assert_eq!(complete.stages.len(), 2);
        assert_eq!(complete.interrupted_stage(&bytes(&frames()), true), None);
        let committed = bytes(&frames()[..3]);
        let interrupted = runner.decode_robustness_journal(&committed, &binding, input.target());
        assert_eq!(
            interrupted.interrupted_stage(&committed, true),
            Some(RobustnessStage::Preparation)
        );
        assert_eq!(interrupted.interrupted_stage(&committed, false), None);
        let foreign = RobustnessInput::new(
            "robustness/protocol",
            input.target(),
            1000,
            b"other bytes".to_vec(),
        )
        .unwrap();
        assert!(runner
            .decode_robustness_journal(
                &bytes(&frames()),
                &Binding::new(&foreign, "unique-request".into()),
                input.target()
            )
            .invalid
            .is_some());
        let mut wrong_stage = frames();
        wrong_stage[2] = WorkerFrame::RobustnessStage {
            stage: RobustnessStage::Validation,
        };
        assert!(runner
            .decode_robustness_journal(&bytes(&wrong_stage), &binding, input.target())
            .invalid
            .is_some());
        let mut false_accepted = frames();
        false_accepted[3] = WorkerFrame::RobustnessTerminal {
            result: RobustnessResult::Accepted {
                stage: RobustnessStage::Validation,
                artifact_bytes: Some(8),
            },
        };
        assert!(runner
            .decode_robustness_journal(&bytes(&false_accepted), &binding, input.target())
            .invalid
            .is_some());
        let mut torn = bytes(&frames()[..3]);
        torn.extend_from_slice(b"{\"kind\":");
        let prefix = runner.decode_robustness_journal(&torn, &binding, input.target());
        assert_eq!(prefix.stages.len(), 2);
        assert!(prefix.result.is_none());
        assert_eq!(prefix.interrupted_stage(&torn, true), None);
        let mut trailing = bytes(&frames());
        trailing.extend_from_slice(b"torn");
        assert!(runner
            .decode_robustness_journal(&trailing, &binding, input.target())
            .invalid
            .is_some());
        let (legacy_runner, legacy_input, legacy_binding) = fixture();
        assert!(matches!(
            legacy_runner
                .decode_journal(&bytes(&frames()), &legacy_binding, &legacy_input)
                .tail,
            JournalTail::Invalid(_)
        ));
    }

    fn provenance(source: &str, image: &str) -> super::super::super::CompilerProvenance {
        serde_json::from_value(serde_json::json!({
            "source_fingerprint_scheme": lila_engine::CompilerIdentity::source_fingerprint_scheme(),
            "source_fingerprint": source.repeat(64),
            "source_revision": { "kind": "unversioned-archive" },
            "executable_sha256": image.repeat(64),
        }))
        .unwrap()
    }

    fn fixture() -> (
        DifferentialWorkerRunner,
        DifferentialReplayInput,
        WorkerBinding,
    ) {
        let identity = provenance("1", "2");
        let runner = DifferentialWorkerRunner {
            executable: "selected-worker".into(),
            selected_image: identity.identity().executable_sha256(),
            parent_identity: identity,
        };
        let input = DifferentialReplayInput::new_script(
            "worker/protocol",
            DifferentialProtocol::V3PrimitiveCompletionPrintTranscript,
            "worker/protocol.js",
            200,
            "print('first'); 1;",
        )
        .unwrap();
        let binding = WorkerBinding::new(
            &input,
            DifferentialBackend::SpecExec,
            "unique-request".into(),
        );
        (runner, input, binding)
    }

    fn bytes(frames: &[WorkerFrame]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for frame in frames {
            serde_json::to_writer(&mut bytes, frame).unwrap();
            bytes.push(b'\n');
        }
        bytes
    }

    fn frames(runner: &DifferentialWorkerRunner, binding: &WorkerBinding) -> Vec<WorkerFrame> {
        vec![
            WorkerFrame::Header {
                version: PROTOCOL_VERSION,
                binding: binding.clone(),
                compiler_identity: runner.parent_identity.clone(),
            },
            WorkerFrame::Admitted {
                binding: binding.clone(),
            },
            WorkerFrame::PrintLine {
                sequence: 0,
                text: "first".into(),
            },
            WorkerFrame::Terminal {
                print_count: 1,
                execution: ExecutionObservation::PrimitiveCompletion {
                    completion: PrimitiveCompletionObservation::Normal {
                        value: PrimitiveValueObservation::Number {
                            bits: "3ff0000000000000".into(),
                        },
                    },
                    backend_note: String::new(),
                },
            },
        ]
    }

    #[test]
    fn completed_journal_requires_selected_image_source_case_and_backend() {
        let (runner, input, binding) = fixture();
        let complete = runner.decode_journal(&bytes(&frames(&runner, &binding)), &binding, &input);
        assert!(matches!(complete.tail, JournalTail::Completed(_)));
        assert_eq!(complete.events, ["first"]);
        assert_eq!(complete.identity, Some(runner.parent_identity.clone()));
        for index in 0..8 {
            let mut changed = binding.clone();
            match index {
                0 => changed.token.push('x'),
                1 => changed.backend = DifferentialBackend::WasmAot,
                2 => changed.case_id.push('x'),
                3 => changed.case_fingerprint.push('x'),
                4 => changed.schema_version = 2,
                5 => changed.goal = DifferentialGoal::Module,
                6 => changed.filename.push('x'),
                7 => changed.observation_contract = ObservationContract::SelfCheckingNoOutput,
                _ => unreachable!(),
            }
            assert!(matches!(
                runner
                    .decode_journal(&bytes(&frames(&runner, &changed)), &binding, &input)
                    .tail,
                JournalTail::Invalid(_)
            ));
        }
        for identity in [provenance("3", "2"), provenance("1", "4")] {
            let mut wrong = frames(&runner, &binding);
            wrong[0] = WorkerFrame::Header {
                version: PROTOCOL_VERSION,
                binding: binding.clone(),
                compiler_identity: identity,
            };
            assert!(matches!(
                runner.decode_journal(&bytes(&wrong), &binding, &input).tail,
                JournalTail::Invalid(_)
            ));
        }
    }

    #[test]
    fn phase_count_and_terminal_checks_retain_only_validated_prints() {
        let (runner, input, binding) = fixture();
        let mut wrong_sequence = frames(&runner, &binding);
        wrong_sequence[2] = WorkerFrame::PrintLine {
            sequence: 1,
            text: "uncommitted".into(),
        };
        let decoded = runner.decode_journal(&bytes(&wrong_sequence), &binding, &input);
        assert!(matches!(decoded.tail, JournalTail::Invalid(_)));
        assert!(decoded.events.is_empty());
        let mut wrong_count = frames(&runner, &binding);
        wrong_count[3] = WorkerFrame::Terminal {
            print_count: 0,
            execution: ExecutionObservation::Normal {
                backend_note: String::new(),
            },
        };
        assert!(matches!(
            runner
                .decode_journal(&bytes(&wrong_count), &binding, &input)
                .tail,
            JournalTail::Invalid(_)
        ));
        let mut out_of_order = frames(&runner, &binding);
        out_of_order.swap(1, 2);
        assert!(matches!(
            runner
                .decode_journal(&bytes(&out_of_order), &binding, &input)
                .tail,
            JournalTail::Invalid(_)
        ));
        let mut duplicate = frames(&runner, &binding);
        duplicate.push(WorkerFrame::Terminal {
            print_count: 1,
            execution: ExecutionObservation::Normal {
                backend_note: String::new(),
            },
        });
        assert!(matches!(
            runner
                .decode_journal(&bytes(&duplicate), &binding, &input)
                .tail,
            JournalTail::Invalid(_)
        ));
    }

    #[test]
    fn torn_and_foreign_frames_do_not_become_committed_output() {
        let (runner, input, binding) = fixture();
        let complete = frames(&runner, &binding);
        let mut torn = bytes(&complete[..3]);
        torn.extend_from_slice(b"{\"kind\":\"print_line\",\"text\":\"torn");
        let decoded = runner.decode_journal(&torn, &binding, &input);
        assert_eq!(decoded.events, ["first"]);
        assert!(matches!(decoded.tail, JournalTail::Missing));
        let mut trailing = bytes(&complete);
        trailing.push(b'x');
        assert!(matches!(
            runner.decode_journal(&trailing, &binding, &input).tail,
            JournalTail::Invalid(_)
        ));
        let mut empty = bytes(&complete);
        empty.push(b'\n');
        assert!(matches!(
            runner.decode_journal(&empty, &binding, &input).tail,
            JournalTail::Invalid(_)
        ));
        let unknown = b"{\"kind\":\"other\"}\n";
        assert!(matches!(
            runner.decode_journal(unknown, &binding, &input).tail,
            JournalTail::Invalid(_)
        ));
    }

    #[test]
    fn explicit_admission_rejection_is_distinct_from_missing_terminal() {
        let (runner, input, binding) = fixture();
        let rejection = vec![
            WorkerFrame::Header {
                version: PROTOCOL_VERSION,
                binding: binding.clone(),
                compiler_identity: runner.parent_identity.clone(),
            },
            WorkerFrame::AdmissionRejected {
                message: "outer source closure is indeterminate".into(),
            },
        ];
        assert!(matches!(
            runner
                .decode_journal(&bytes(&rejection), &binding, &input)
                .tail,
            JournalTail::AdmissionRejected(_)
        ));
        assert!(matches!(
            runner
                .decode_journal(&bytes(&rejection[..1]), &binding, &input)
                .tail,
            JournalTail::Missing
        ));
    }

    #[test]
    fn primitive_wire_requires_canonical_number_and_bigint_domains() {
        let (runner, input, binding) = fixture();
        for value in [
            PrimitiveValueObservation::Number {
                bits: "7ff0000000000001".into(),
            },
            PrimitiveValueObservation::Number {
                bits: "3FF0000000000000".into(),
            },
            PrimitiveValueObservation::BigInt {
                decimal: "01".into(),
            },
            PrimitiveValueObservation::BigInt {
                decimal: "-0".into(),
            },
        ] {
            let mut invalid = frames(&runner, &binding);
            invalid[3] = WorkerFrame::Terminal {
                print_count: 1,
                execution: ExecutionObservation::PrimitiveCompletion {
                    completion: PrimitiveCompletionObservation::Normal { value },
                    backend_note: String::new(),
                },
            };
            assert!(matches!(
                runner
                    .decode_journal(&bytes(&invalid), &binding, &input)
                    .tail,
                JournalTail::Invalid(_)
            ));
        }
    }

    #[test]
    fn rooted_journal_requires_the_requested_limits_and_native_result_domain() {
        let (runner, _, _) = fixture();
        let input = DifferentialReplayInput::new_snapshot_script(
            "t25/rooted/journal",
            "rooted-journal.js",
            5_000,
            "0",
            SnapshotLimits::default(),
        )
        .unwrap();
        let binding =
            WorkerBinding::new(&input, DifferentialBackend::SpecExec, "rooted-token".into());
        for (limits, accepted) in [
            (SnapshotLimits::default(), true),
            (SnapshotLimits::HARD_MAX, false),
        ] {
            let graph: RootedSnapshotGraph = serde_json::from_value(serde_json::json!({
                "version":1,"limits":limits,"root":{"type":"undefined"},"nodes":[],"symbols":[],"realm_count":1
            })).unwrap();
            let mut journal = frames(&runner, &binding);
            *journal.last_mut().unwrap() = WorkerFrame::Terminal {
                print_count: 1,
                execution: ExecutionObservation::RootedCompletionGraph {
                    completion: SnapshotCompletion {
                        kind: SnapshotCompletionKind::Normal,
                        outcome: SnapshotOutcome::Captured { graph },
                    },
                    backend_note: String::new(),
                },
            };
            let decoded = runner.decode_journal(&bytes(&journal), &binding, &input);
            assert_eq!(matches!(decoded.tail, JournalTail::Completed(_)), accepted);
        }
        // The old primitive terminal is structurally valid JSON but cannot
        // satisfy v7 by silently substituting the lossy completion path.
        assert!(matches!(
            runner
                .decode_journal(&bytes(&frames(&runner, &binding)), &binding, &input)
                .tail,
            JournalTail::Invalid(_)
        ));
    }

    #[test]
    fn worker_failures_on_both_sides_never_match_or_acquire_a_signature() {
        let (_, input, _) = fixture();
        let failure = |backend| {
            worker_failure(
                backend,
                empty_journal(),
                DifferentialWorkerFailure::Exit {
                    code: None,
                    signal: Some(9),
                },
                None,
            )
            .into_observation()
        };
        let report = compare_observations(
            &input,
            failure(DifferentialBackend::WasmAot),
            failure(DifferentialBackend::SpecExec),
        );
        assert_eq!(report.verdict(), DifferentialVerdict::WorkerFailure);
        assert!(report.mismatch_signature().is_none());
    }

    #[test]
    fn request_budget_does_not_allocate_or_write_an_overflowing_chunk() {
        let mut output = Vec::new();
        let mut writer = LimitedWriter {
            output: &mut output,
            written: MAX_REQUEST_BYTES - 1,
        };
        assert!(writer.write_all(b"xx").is_err());
        assert!(output.is_empty());
    }

    #[test]
    fn staging_is_private_and_retires_request_and_journal_together() {
        let stage = Stage::new().unwrap();
        let directory = stage.directory.clone();
        fs::write(directory.join("request.json"), b"request").unwrap();
        fs::write(directory.join("journal.jsonl"), b"journal").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        assert!(stage.finish().is_none());
        assert!(!directory.exists());
    }

    #[cfg(unix)]
    #[test]
    fn retirement_terminates_and_reaps_a_live_worker_group() {
        let child = Command::new("/bin/sh")
            .args(["-c", "sleep 30 & wait"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .unwrap();
        let group = i32::try_from(child.id()).unwrap();
        let mut live = LiveWorker {
            child,
            group: Some(group),
            retired: false,
        };
        assert!(live.child.try_wait().unwrap().is_none());
        assert!(live.retire().is_none());
        assert!(live.child.try_wait().unwrap().is_some());
        assert!(live.retire().is_none());
    }
}
