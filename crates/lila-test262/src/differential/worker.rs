//! The sole process entry that admits source and runs a differential backend.

use super::*;
use lila_engine::{Engine, HostOutputEvent, RealmBuilder, RunOptions};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

pub(super) const PROTOCOL_VERSION: u32 = 1;
pub(super) const BUDGET_EXIT: i32 = 79;
pub(super) const JOURNAL_EXIT: i32 = 80;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WorkerBinding {
    pub(super) token: String,
    pub(super) backend: DifferentialBackend,
    pub(super) case_id: String,
    pub(super) case_fingerprint: String,
    pub(super) schema_version: u32,
    pub(super) observation_contract: ObservationContract,
    pub(super) goal: DifferentialGoal,
    pub(super) filename: String,
}

impl WorkerBinding {
    pub(super) fn new(
        input: &DifferentialReplayInput,
        backend: DifferentialBackend,
        token: String,
    ) -> Self {
        Self {
            token,
            backend,
            case_id: input.id().as_str().into(),
            case_fingerprint: input_fingerprint(input).0,
            schema_version: input.protocol().schema_version(),
            observation_contract: input.observation_contract(),
            goal: input.goal(),
            filename: input.filename().into(),
        }
    }
}

#[derive(Serialize)]
pub(super) struct RequestProjection<'a> {
    pub(super) version: u32,
    pub(super) binding: &'a WorkerBinding,
    pub(super) corpus: &'a DifferentialReplayInput,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestWire {
    version: u32,
    binding: WorkerBinding,
    corpus: serde_json::Value,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum RequestEnvelope {
    Differential(RequestWire),
    Robustness(super::robustness::worker::RequestWire),
    Test262(super::test262_seeds::worker::RequestWire),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum WorkerFrame {
    Test262Header {
        version: u32,
        binding: super::test262_seeds::worker::Binding,
        compiler_identity: super::super::CompilerProvenance,
    },
    Test262Admitted {
        binding: super::test262_seeds::worker::Binding,
    },
    Test262Terminal {
        result: super::test262_seeds::Test262ReplayResult,
    },
    RobustnessHeader {
        version: u32,
        binding: super::robustness::worker::Binding,
        compiler_identity: super::super::CompilerProvenance,
    },
    RobustnessStage {
        stage: super::robustness::RobustnessStage,
    },
    RobustnessTerminal {
        result: super::robustness::RobustnessResult,
    },
    Header {
        version: u32,
        binding: WorkerBinding,
        compiler_identity: super::super::CompilerProvenance,
    },
    Admitted {
        binding: WorkerBinding,
    },
    PrintLine {
        sequence: u64,
        text: String,
    },
    Terminal {
        print_count: u64,
        execution: ExecutionObservation,
    },
    AdmissionRejected {
        message: String,
    },
}

struct FrameBuffer {
    bytes: Vec<u8>,
    exceeded: bool,
}
impl Write for FrameBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > worker_process::MAX_FRAME_BYTES.saturating_sub(self.bytes.len()) {
            self.exceeded = true;
            return Err(std::io::Error::other("journal frame budget exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(super) struct Journal {
    file: fs::File,
    bytes_written: usize,
    events: Vec<String>,
}

impl Journal {
    pub(super) fn new(path: &Path) -> Result<Self, DifferentialError> {
        Ok(Self {
            file: fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?,
            bytes_written: 0,
            events: Vec::new(),
        })
    }
    pub(super) fn write(&mut self, frame: &WorkerFrame) {
        let mut buffer = FrameBuffer {
            bytes: Vec::new(),
            exceeded: false,
        };
        if serde_json::to_writer(&mut buffer, frame).is_err() {
            std::process::exit(if buffer.exceeded {
                BUDGET_EXIT
            } else {
                JOURNAL_EXIT
            });
        }
        let bytes = buffer.bytes;
        if bytes.len() > worker_process::MAX_FRAME_BYTES
            || self
                .bytes_written
                .saturating_add(bytes.len())
                .saturating_add(1)
                > worker_process::MAX_JOURNAL_BYTES
        {
            // Engine also captures events. Terminate immediately before it can
            // continue allocating after this hook's observation budget is lost.
            std::process::exit(BUDGET_EXIT);
        }
        if self
            .file
            .write_all(&bytes)
            .and_then(|()| self.file.write_all(b"\n"))
            .and_then(|()| self.file.flush())
            .is_err()
        {
            std::process::exit(JOURNAL_EXIT);
        }
        self.bytes_written += bytes.len() + 1;
    }

    fn print(&mut self, text: &str) {
        if text.len() > worker_process::MAX_FRAME_BYTES {
            std::process::exit(BUDGET_EXIT);
        }
        self.write(&WorkerFrame::PrintLine {
            sequence: self.events.len() as u64,
            text: text.into(),
        });
        self.events.push(text.into());
    }
}

#[derive(Debug)]
struct JournalOutput {
    journal: Arc<Mutex<Journal>>,
}

impl fmt::Debug for Journal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Journal")
            .field("bytes_written", &self.bytes_written)
            .finish_non_exhaustive()
    }
}

impl lila_engine::HostHooks for JournalOutput {
    fn print_line(&self, text: &str) {
        match self.journal.lock() {
            Ok(mut journal) => journal.print(text),
            Err(_) => std::process::exit(JOURNAL_EXIT),
        }
    }
}

/// Hidden developer-worker entry. The caller must dispatch it before building
/// any Realm/Engine. There is no parent-accessible raw backend executor.
pub fn run_differential_worker(
    request_path: &Path,
    journal_path: &Path,
    _oracle: SpecExecOracle,
) -> Result<(), DifferentialError> {
    lila_engine::configure_compilation_jobs(1).map_err(DifferentialError::WorkerConfiguration)?;
    let file = fs::File::open(request_path)
        .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
    let mut bytes = Vec::new();
    file.take(worker_process::MAX_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
    if bytes.len() > worker_process::MAX_REQUEST_BYTES {
        return Err(DifferentialError::WorkerConfiguration(
            "worker request budget exceeded".into(),
        ));
    }
    let request: RequestEnvelope = serde_json::from_slice(&bytes)
        .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
    let request = match request {
        RequestEnvelope::Differential(request) => request,
        RequestEnvelope::Robustness(request) => {
            return super::robustness::worker::run(request, journal_path)
        }
        RequestEnvelope::Test262(request) => {
            return super::test262_seeds::worker::run(request, journal_path)
        }
    };
    if request.version != PROTOCOL_VERSION {
        return Err(DifferentialError::WorkerConfiguration(
            "unknown worker protocol".into(),
        ));
    }
    let mut journal = Journal::new(journal_path)?;
    journal.write(&WorkerFrame::Header {
        version: PROTOCOL_VERSION,
        binding: request.binding.clone(),
        compiler_identity: super::super::CompilerProvenance::current()
            .map_err(DifferentialError::WorkerConfiguration)?,
    });
    let mut corpus = Vec::new();
    worker_process::write_bounded_json(&request.corpus, &mut corpus)
        .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?;
    let native = DifferentialReplayInput::from_json(
        std::str::from_utf8(&corpus)
            .map_err(|error| DifferentialError::WorkerConfiguration(error.to_string()))?,
    )?;
    if WorkerBinding::new(
        &native,
        request.binding.backend,
        request.binding.token.clone(),
    ) != request.binding
    {
        return Err(DifferentialError::WorkerConfiguration(
            "worker input identity differs from request".into(),
        ));
    }
    let case = match native.admit() {
        Ok(case) => case,
        Err(error) => {
            journal.write(&WorkerFrame::AdmissionRejected {
                message: error.to_string(),
            });
            return Ok(());
        }
    };
    journal.write(&WorkerFrame::Admitted {
        binding: request.binding.clone(),
    });
    let journal = Arc::new(Mutex::new(journal));
    let execution = execute_case(&case, request.binding.backend, Arc::clone(&journal));
    let projected = project_backend_execution(case.protocol(), execution);
    let mut journal = journal
        .lock()
        .unwrap_or_else(|_| std::process::exit(JOURNAL_EXIT));
    let print_count = journal.events.len() as u64;
    journal.write(&WorkerFrame::Terminal {
        print_count,
        execution: projected.execution,
    });
    Ok(())
}

fn execute_case(
    case: &DifferentialCase,
    backend: DifferentialBackend,
    journal: Arc<Mutex<Journal>>,
) -> BackendExecution {
    let engine = Engine::new(
        RealmBuilder::new()
            .with_host_hooks(Box::new(JournalOutput {
                journal: Arc::clone(&journal),
            }))
            .build(),
    );
    let compile = compile_options_for_case(case);
    let run = RunOptions {
        backend: backend.execution_backend(),
        test_path: Some(case.filename.clone()),
        can_block: false,
        timeout_ms: match backend {
            DifferentialBackend::WasmAot => Some(case.timeout_ms.get()),
            DifferentialBackend::SpecExec => None,
        },
        ..RunOptions::default()
    };
    let ordinary_outcome = |outcome: lila_engine::ObservedRunOutcome| {
        (
            outcome.backend_used,
            outcome.output_events,
            BackendExecutionResult::Completion {
                completion: outcome.completion,
                backend_note: outcome.note,
            },
        )
    };
    let outcome = match &case.program {
        DifferentialProgram::DependencySealedScript(source) => {
            if case.protocol() == DifferentialProtocol::V5SelectedObjectProbePrintTranscript {
                engine
                    .observe_script(&object_probe::execution_source(source), compile, run)
                    .map(ordinary_outcome)
            } else {
                engine
                    .observe_script(source, compile, run)
                    .map(ordinary_outcome)
            }
        }
        DifferentialProgram::EmbeddedGraph(graph) => match graph.entry().goal() {
            EmbeddedModuleGoal::Script => engine
                .observe_script(graph.entry().source(), compile, run)
                .map(ordinary_outcome),
            EmbeddedModuleGoal::Module => engine
                .observe_module(graph.entry().source(), compile, run)
                .map(ordinary_outcome),
        },
        DifferentialProgram::RootedSnapshot(program) => {
            let limits = case
                .snapshot_limits()
                .expect("snapshot program owns checked limits");
            match program.goal() {
                DifferentialGoal::Script => {
                    engine.observe_script_graph(program.source(), compile, run, limits)
                }
                DifferentialGoal::Module => {
                    engine.observe_module_graph(program.source(), compile, run, limits)
                }
            }
            .map(|outcome| {
                (
                    outcome.backend_used,
                    outcome.output_events,
                    BackendExecutionResult::RootedCompletionGraph {
                        completion: outcome.completion,
                        backend_note: outcome.note,
                    },
                )
            })
        }
    };
    let result = match outcome {
        Ok((backend_used, output_events, completion)) => {
            let events: Vec<String> = output_events
                .into_iter()
                .map(|event| match event {
                    HostOutputEvent::PrintLine(text) => text,
                })
                .collect();
            let journal = journal
                .lock()
                .unwrap_or_else(|_| std::process::exit(JOURNAL_EXIT));
            if backend_used != backend.execution_backend() || events != journal.events {
                BackendExecutionResult::EngineFailure {
                    phase: FailurePhase::RunnerInvariant,
                    message:
                        "backend identity or captured print transcript differs from worker hook"
                            .into(),
                }
            } else {
                completion
            }
        }
        Err(error) => observe_engine_error(backend, &error),
    };
    BackendExecution {
        backend,
        output_events: OutputEventsObservation::Captured { events: Vec::new() },
        result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frame_serialization_stops_before_overflowing_its_buffer() {
        let mut buffer = FrameBuffer {
            bytes: vec![0; worker_process::MAX_FRAME_BYTES - 1],
            exceeded: false,
        };
        assert!(buffer.write_all(b"xx").is_err());
        assert!(buffer.exceeded);
        assert_eq!(buffer.bytes.len(), worker_process::MAX_FRAME_BYTES - 1);
    }
}
