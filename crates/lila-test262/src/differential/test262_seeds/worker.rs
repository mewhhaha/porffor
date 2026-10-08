//! A separate request domain uses the original Test262 worker evaluator,
//! including negative expectations, async completion, Module and agent hosts.
use super::*;
use crate::differential::worker::{Journal, WorkerFrame, PROTOCOL_VERSION};
use crate::differential::{DifferentialBackend, DifferentialWorkerFailure};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::differential) struct Binding {
    token: String,
    backend: DifferentialBackend,
    execution_id: TestExecutionId,
    seed_sha256: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn bytes(frames: &[WorkerFrame]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for frame in frames {
            serde_json::to_writer(&mut bytes, frame).unwrap();
            bytes.push(b'\n');
        }
        bytes
    }
    #[test]
    fn selected_worker_journal_binds_mode_backend_image_and_committed_admission() {
        let runner = DifferentialWorkerRunner::new(std::env::current_exe().unwrap()).unwrap();
        let identity = CompilerProvenance::current().unwrap();
        let binding = Binding {
            token: "test262-control".into(),
            backend: DifferentialBackend::WasmAot,
            execution_id: TestExecutionId::parse_wire_key("strict-script:case.js").unwrap(),
            seed_sha256: "7".repeat(64),
        };
        let header = || WorkerFrame::Test262Header {
            version: PROTOCOL_VERSION,
            binding: binding.clone(),
            compiler_identity: identity.clone(),
        };
        let admitted = || WorkerFrame::Test262Admitted {
            binding: binding.clone(),
        };
        let terminal = || WorkerFrame::Test262Terminal {
            result: Test262ReplayResult::Passed { duration_ms: 2 },
        };
        let complete = bytes(&[header(), admitted(), terminal()]);
        assert!(runner.decode_test262(&complete, &binding).1.is_ok());
        let mut foreign = binding.clone();
        foreign.backend = DifferentialBackend::SpecExec;
        assert!(runner.decode_test262(&complete, &foreign).1.is_err());
        foreign = binding.clone();
        foreign.execution_id = TestExecutionId::parse_wire_key("sloppy-script:case.js").unwrap();
        assert!(runner.decode_test262(&complete, &foreign).1.is_err());
        foreign = binding.clone();
        foreign.seed_sha256 = "8".repeat(64);
        assert!(runner.decode_test262(&complete, &foreign).1.is_err());
        assert!(runner
            .decode_test262(&bytes(&[header(), terminal()]), &binding)
            .1
            .is_err());
        assert!(runner
            .decode_test262(
                &bytes(&[header(), admitted(), terminal(), terminal()]),
                &binding
            )
            .1
            .is_err());
        assert!(runner
            .decode_test262(&complete[..complete.len() - 1], &binding)
            .1
            .is_err());
        let rejected = WorkerFrame::Test262Terminal {
            result: Test262ReplayResult::AdmissionRejected {
                message: "changed harness".into(),
            },
        };
        assert!(runner
            .decode_test262(&bytes(&[header(), rejected]), &binding)
            .1
            .is_ok());
    }
}
impl Binding {
    fn new(seed: &Test262ReplaySeed, backend: DifferentialBackend, token: String) -> Self {
        Self {
            token,
            backend,
            execution_id: seed.execution_id().clone(),
            seed_sha256: seed.fingerprint(),
        }
    }
}
#[derive(Serialize)]
struct RequestProjection<'a> {
    version: u32,
    test262_binding: &'a Binding,
    test262_seed: &'a Test262ReplaySeed,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::differential) struct RequestWire {
    version: u32,
    test262_binding: Binding,
    test262_seed: serde_json::Value,
}

pub(in crate::differential) fn run(
    request: RequestWire,
    journal_path: &Path,
) -> Result<(), DifferentialError> {
    if request.version != PROTOCOL_VERSION {
        return Err(invalid("unknown Test262 worker protocol"));
    }
    let mut journal = Journal::new(journal_path)?;
    journal.write(&WorkerFrame::Test262Header {
        version: PROTOCOL_VERSION,
        binding: request.test262_binding.clone(),
        compiler_identity: CompilerProvenance::current()
            .map_err(DifferentialError::WorkerConfiguration)?,
    });
    let json = serde_json::to_string(&request.test262_seed).map_err(|e| invalid(e.to_string()))?;
    let seed = match Test262ReplaySeed::from_json(&json) {
        Ok(seed) => seed,
        Err(error) => {
            journal.write(&WorkerFrame::Test262Terminal {
                result: Test262ReplayResult::AdmissionRejected {
                    message: error.to_string(),
                },
            });
            return Ok(());
        }
    };
    if Binding::new(
        &seed,
        request.test262_binding.backend,
        request.test262_binding.token.clone(),
    ) != request.test262_binding
    {
        return Err(invalid(
            "Test262 worker binding differs from the admitted seed",
        ));
    }
    journal.write(&WorkerFrame::Test262Admitted {
        binding: request.test262_binding.clone(),
    });
    let backend = request.test262_binding.backend.execution_backend();
    let result = execute(seed, backend);
    journal.write(&WorkerFrame::Test262Terminal { result });
    Ok(())
}

fn execute(seed: Test262ReplaySeed, backend: ExecutionBackend) -> Test262ReplayResult {
    let run = move || -> Result<Test262ReplayResult, DifferentialError> {
        let config = backend_config(
            &SuiteConfig {
                suite_root: seed.wire.suite_root.clone(),
                snapshot_dir: seed.wire.snapshot_dir.clone(),
                timeout_ms: seed.wire.timeout_ms,
                ..SuiteConfig::default()
            },
            backend,
        );
        let manifest = crate::discover_suite(&config, Some(&seed.execution_id().wire_key()))
            .map_err(invalid)?;
        let case = manifest
            .cases
            .into_iter()
            .next()
            .ok_or_else(|| invalid("selected Test262 execution vanished"))?;
        let preludes = crate::load_preludes(&config).map_err(invalid)?;
        let original =
            crate::run_one_case_on_persistent_worker(&case, &preludes, config.timeout_ms, backend);
        if original.test_id != *seed.execution_id() {
            return Err(invalid("Test262 evaluator returned a foreign execution id"));
        }
        if suite_digest(&config.suite_root)? != seed.wire.suite_sha256 {
            return Err(invalid("suite dependencies changed during Test262 replay"));
        }
        Ok(match original.status {
            crate::TestStatus::Passed => Test262ReplayResult::Passed {
                duration_ms: original.duration_ms,
            },
            crate::TestStatus::Failed(failure) => Test262ReplayResult::Failed {
                kind: failure.kind,
                outcome: failure.outcome,
                origin: failure.origin,
                detail: failure.detail,
                detail_hash: failure.detail_hash,
                duration_ms: original.duration_ms,
            },
        })
    };
    match std::thread::Builder::new()
        .name("test262-differential-seed".into())
        .stack_size(crate::TEST262_WORKER_STACK_SIZE)
        .spawn(run)
    {
        Ok(thread) => match thread.join() {
            Ok(Ok(result)) => result,
            Ok(Err(error)) => failure(error.to_string()),
            Err(_) => failure("Test262 evaluator panicked".into()),
        },
        Err(error) => failure(error.to_string()),
    }
}
fn failure(message: String) -> Test262ReplayResult {
    Test262ReplayResult::WorkerFailure {
        failure: DifferentialWorkerFailure::Process { message },
        cleanup_error: None,
    }
}

impl DifferentialWorkerRunner {
    pub(in crate::differential) fn run_test262(
        &self,
        seed: &Test262ReplaySeed,
        backend: DifferentialBackend,
        _oracle: SpecExecOracle,
    ) -> Result<Test262BackendReplay, DifferentialError> {
        #[cfg(not(unix))]
        {
            let _ = (seed, backend);
            Ok(Test262BackendReplay {
                compiler_identity: None,
                result: Test262ReplayResult::WorkerFailure {
                    failure: DifferentialWorkerFailure::UnsupportedPlatform,
                    cleanup_error: None,
                },
                journal_bytes_hex: String::new(),
                stderr_bytes_hex: String::new(),
            })
        }
        #[cfg(unix)]
        {
            let transport = self.run_transport(seed.wire.timeout_ms, |token, file| {
                let binding = Binding::new(seed, backend, token.into());
                crate::differential::worker_process::write_bounded_json(
                    &RequestProjection {
                        version: PROTOCOL_VERSION,
                        test262_binding: &binding,
                        test262_seed: seed,
                    },
                    file,
                )
            })?;
            let binding = Binding::new(seed, backend, transport.token);
            let (identity, decoded) = self.decode_test262(&transport.bytes, &binding);
            let result = if let Some(failure) = transport.failure {
                Test262ReplayResult::WorkerFailure {
                    failure,
                    cleanup_error: transport.cleanup,
                }
            } else if let Some(cleanup) = transport.cleanup {
                Test262ReplayResult::WorkerFailure {
                    failure: DifferentialWorkerFailure::Process {
                        message: "Test262 worker cleanup failed".into(),
                    },
                    cleanup_error: Some(cleanup),
                }
            } else {
                decoded.unwrap_or_else(|message| Test262ReplayResult::WorkerFailure {
                    failure: DifferentialWorkerFailure::Protocol { message },
                    cleanup_error: None,
                })
            };
            Ok(Test262BackendReplay {
                compiler_identity: identity,
                result,
                journal_bytes_hex: transport.bytes.iter().map(|b| format!("{b:02x}")).collect(),
                stderr_bytes_hex: transport
                    .stderr
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect(),
            })
        }
    }

    fn decode_test262(
        &self,
        bytes: &[u8],
        binding: &Binding,
    ) -> (
        Option<CompilerProvenance>,
        Result<Test262ReplayResult, String>,
    ) {
        let mut identity = None;
        let mut admitted = false;
        let mut result = None;
        if !bytes.ends_with(b"\n") {
            return (
                identity,
                Err("Test262 journal is missing its committed terminal frame".into()),
            );
        }
        for line in bytes
            .strip_suffix(b"\n")
            .unwrap_or(bytes)
            .split(|b| *b == b'\n')
        {
            if line.is_empty() || line.len() > crate::differential::worker_process::MAX_FRAME_BYTES
            {
                return (
                    identity,
                    Err("empty or oversized Test262 journal frame".into()),
                );
            }
            let frame = match serde_json::from_slice::<WorkerFrame>(line) {
                Ok(frame) => frame,
                Err(error) => return (identity, Err(error.to_string())),
            };
            match frame {
                WorkerFrame::Test262Header {
                    version,
                    binding: actual,
                    compiler_identity,
                } if identity.is_none()
                    && !admitted
                    && result.is_none()
                    && version == PROTOCOL_VERSION
                    && actual == *binding
                    && self.identity_matches(&compiler_identity) =>
                {
                    identity = Some(compiler_identity)
                }
                WorkerFrame::Test262Admitted { binding: actual }
                    if identity.is_some()
                        && !admitted
                        && result.is_none()
                        && actual == *binding =>
                {
                    admitted = true
                }
                WorkerFrame::Test262Terminal { result: actual }
                    if identity.is_some() && result.is_none() =>
                {
                    if admitted == matches!(&actual, Test262ReplayResult::AdmissionRejected { .. })
                    {
                        return (
                            identity,
                            Err("Test262 terminal disagrees with admission order".into()),
                        );
                    }
                    if matches!(
                        &actual,
                        Test262ReplayResult::Failed {
                            outcome: OutcomeKind::Success,
                            ..
                        }
                    ) {
                        return (
                            identity,
                            Err("a failed Test262 execution cannot carry Success".into()),
                        );
                    }
                    result = Some(actual);
                }
                _ => {
                    return (
                        identity,
                        Err("foreign Test262 identity, frame order or terminal domain".into()),
                    )
                }
            }
        }
        (
            identity,
            result.ok_or_else(|| "Test262 worker exited without a terminal observation".into()),
        )
    }
}
