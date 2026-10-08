use super::super::worker::{Journal, WorkerFrame, PROTOCOL_VERSION};
use super::*;
use lila_engine::{
    CompileOptions, CompilerInspectionStage, Engine, ModuleLoadingPolicy, RealmBuilder,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::differential) struct Binding {
    token: String,
    id: String,
    fingerprint: String,
    target: RobustnessTarget,
    length: u64,
}
impl Binding {
    pub(in crate::differential) fn new(input: &RobustnessInput, token: String) -> Self {
        Self {
            token,
            id: input.id().as_str().into(),
            fingerprint: input.fingerprint(),
            target: input.target(),
            length: input.bytes().len() as u64,
        }
    }
}
#[derive(Serialize)]
pub(in crate::differential) struct RequestProjection<'a> {
    pub(in crate::differential) version: u32,
    pub(in crate::differential) robustness_binding: &'a Binding,
    pub(in crate::differential) robustness: &'a RobustnessInput,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::differential) struct RequestWire {
    version: u32,
    robustness_binding: Binding,
    robustness: serde_json::Value,
}

pub(in crate::differential) fn run(
    request: RequestWire,
    path: &Path,
) -> Result<(), DifferentialError> {
    if request.version != PROTOCOL_VERSION {
        return Err(DifferentialError::WorkerConfiguration(
            "unknown robustness worker protocol".into(),
        ));
    }
    let mut journal = Journal::new(path)?;
    journal.write(&WorkerFrame::RobustnessHeader {
        version: PROTOCOL_VERSION,
        binding: request.robustness_binding.clone(),
        compiler_identity: crate::CompilerProvenance::current()
            .map_err(DifferentialError::WorkerConfiguration)?,
    });
    let json = serde_json::to_string(&request.robustness)
        .map_err(|error| DifferentialError::DecodeCorpus(error.to_string()))?;
    let input = RobustnessInput::from_json(&json)?;
    if Binding::new(&input, request.robustness_binding.token.clone()) != request.robustness_binding
    {
        return Err(DifferentialError::WorkerConfiguration(
            "robustness worker input identity differs".into(),
        ));
    }
    journal.write(&WorkerFrame::RobustnessStage {
        stage: RobustnessStage::Decode,
    });
    let result = match std::str::from_utf8(input.bytes()) {
        Err(error) => RobustnessResult::Rejected {
            stage: RobustnessStage::Decode,
            phase: RobustnessRejectionPhase::Encoding,
            message: error.to_string(),
        },
        Ok(text) => execute(
            &input,
            text,
            &mut journal,
            path.parent().ok_or_else(|| {
                DifferentialError::WorkerConfiguration(
                    "worker journal has no owned staging parent".into(),
                )
            })?,
        ),
    };
    journal.write(&WorkerFrame::RobustnessTerminal { result });
    Ok(())
}
fn enter(journal: &mut Journal, stage: RobustnessStage) {
    journal.write(&WorkerFrame::RobustnessStage { stage });
}
fn native(stage: RobustnessStage, result: Result<(), String>) -> RobustnessResult {
    match result {
        Ok(()) => RobustnessResult::Accepted {
            stage,
            artifact_bytes: None,
        },
        Err(message) => RobustnessResult::Rejected {
            stage,
            phase: RobustnessRejectionPhase::NativeBoundary,
            message,
        },
    }
}
fn execute(
    input: &RobustnessInput,
    text: &str,
    journal: &mut Journal,
    sandbox_parent: &Path,
) -> RobustnessResult {
    match input.target() {
        RobustnessTarget::Compiler { goal } => compile(
            input,
            CompilerInput::Source { source: text, goal },
            false,
            journal,
        ),
        RobustnessTarget::IrAdmission {} => {
            compile(input, CompilerInput::Ir(input.bytes()), false, journal)
        }
        RobustnessTarget::Prelude {} => prelude_probe(text, sandbox_parent, journal),
        RobustnessTarget::FilesystemResolver {} => filesystem_probe(text, sandbox_parent, journal),
        RobustnessTarget::Builtin { parser } => {
            if parser.is_uri() {
                enter(journal, RobustnessStage::BuiltinInput);
                let source = match super::uri::UriInput::from_json(text)
                    .and_then(|input| input.source(parser))
                {
                    Ok(source) => source,
                    Err(message) => {
                        return RobustnessResult::Rejected {
                            stage: RobustnessStage::BuiltinInput,
                            phase: RobustnessRejectionPhase::NativeBoundary,
                            message,
                        }
                    }
                };
                return compile(
                    input,
                    CompilerInput::Source {
                        source: &source,
                        goal: DifferentialGoal::Script,
                    },
                    true,
                    journal,
                );
            }
            // Raw data is one String literal, never source splicing or eval.
            let literal = match serde_json::to_string(text) {
                Ok(literal) => literal,
                Err(error) => {
                    return RobustnessResult::Failure {
                        stage: RobustnessStage::Decode,
                        message: error.to_string(),
                    }
                }
            };
            let expression = match parser {
                BuiltinParserTarget::Json => format!("JSON.parse({literal})"),
                BuiltinParserTarget::RegExp => format!("new RegExp({literal})"),
                BuiltinParserTarget::Number => format!("Number({literal})"),
                BuiltinParserTarget::BigInt => format!("BigInt({literal})"),
                BuiltinParserTarget::TemporalDate => format!("Temporal.PlainDate.from({literal})"),
                BuiltinParserTarget::EncodeUri
                | BuiltinParserTarget::EncodeUriComponent
                | BuiltinParserTarget::DecodeUri
                | BuiltinParserTarget::DecodeUriComponent => {
                    unreachable!("URI data passed its checked native decoder")
                }
            };
            compile(
                input,
                CompilerInput::Source {
                    source: &format!("void ({expression});"),
                    goal: DifferentialGoal::Script,
                },
                true,
                journal,
            )
        }
        RobustnessTarget::Frontmatter {} => {
            enter(journal, RobustnessStage::Frontmatter);
            native(
                RobustnessStage::Frontmatter,
                crate::parse_test_executions(
                    "robustness.js".into(),
                    PathBuf::from("robustness.js"),
                    text.into(),
                )
                .map(|_| ()),
            )
        }
        RobustnessTarget::Snapshot {} => {
            enter(journal, RobustnessStage::Snapshot);
            native(
                RobustnessStage::Snapshot,
                crate::decode_snapshot_bytes(input.bytes(), Path::new("robustness.snapshot.json"))
                    .map(|_| ()),
            )
        }
        RobustnessTarget::Corpus {} => {
            enter(journal, RobustnessStage::Corpus);
            native(
                RobustnessStage::Corpus,
                DifferentialReplayInput::from_json(text)
                    .map(|_| ())
                    .map_err(|error| error.to_string()),
            )
        }
        RobustnessTarget::ModuleGraph {} => {
            enter(journal, RobustnessStage::ModuleGraph);
            // Exact v4 native resolver/catalog admission, with no filesystem IO.
            native(
                RobustnessStage::ModuleGraph,
                serde_json::from_str::<super::super::embedded_graph::EmbeddedCaseWire>(text)
                    .map_err(|error| error.to_string())
                    .and_then(|wire| {
                        wire.into_input()
                            .map(|_| ())
                            .map_err(|error| error.to_string())
                    }),
            )
        }
        RobustnessTarget::ReportObservation {} => {
            enter(journal, RobustnessStage::ReportObservation);
            // A BackendObservation is produced only by the original completed
            // worker consumer. Exercise that consumer's actual untrusted frame
            // decoder; arbitrary JSON cannot construct its provenance result.
            native(RobustnessStage::ReportObservation,
                serde_json::from_str::<WorkerFrame>(text)
                    .map_err(|error| error.to_string()).and_then(|frame| {
                        let WorkerFrame::Terminal { execution, .. } = frame else {
                            return Err("report-observation requires a native terminal frame".into());
                        };
                        if worker_process::valid_terminal(DifferentialProtocol::V3PrimitiveCompletionPrintTranscript, &execution) {
                            Ok(())
                        } else { Err("observation is outside the original schema-v3 worker terminal domain".into()) }
                    }))
        }
    }
}
fn prelude_probe(text: &str, parent: &Path, journal: &mut Journal) -> RobustnessResult {
    use super::prelude::PreludeLoadError;
    enter(journal, RobustnessStage::PreludeInput);
    let input = match super::prelude::PreludeInput::from_json(text) {
        Ok(input) => input,
        Err(message) => return native(RobustnessStage::PreludeInput, Err(message)),
    };
    enter(journal, RobustnessStage::PreludeLoad);
    let loaded = match input.load(parent) {
        Ok(loaded) => loaded,
        Err(PreludeLoadError::Rejected(message)) => {
            return native(RobustnessStage::PreludeLoad, Err(message))
        }
        Err(PreludeLoadError::Fixture(message)) => {
            return RobustnessResult::Failure {
                stage: RobustnessStage::PreludeLoad,
                message,
            }
        }
    };
    enter(journal, RobustnessStage::PreludeMaterialization);
    let result = native(
        RobustnessStage::PreludeMaterialization,
        loaded.materialize().map(|_| ()),
    );
    match loaded.finish() {
        Ok(()) => result,
        Err(message) => RobustnessResult::Failure {
            stage: RobustnessStage::PreludeMaterialization,
            message,
        },
    }
}
fn filesystem_probe(text: &str, parent: &Path, journal: &mut Journal) -> RobustnessResult {
    use super::filesystem::{FilesystemInput, ProbeError, SetupError};
    enter(journal, RobustnessStage::FilesystemInput);
    let input = match FilesystemInput::from_json(text) {
        Ok(input) => input,
        Err(message) => return native(RobustnessStage::FilesystemInput, Err(message)),
    };
    enter(journal, RobustnessStage::FilesystemSetup);
    let probe = match input.setup(parent) {
        Ok(probe) => probe,
        Err(SetupError::Failure(message)) => {
            return RobustnessResult::Failure {
                stage: RobustnessStage::FilesystemSetup,
                message,
            }
        }
        Err(SetupError::Unavailable(message)) => {
            return RobustnessResult::Unsupported {
                stage: RobustnessStage::FilesystemSetup,
                message,
            }
        }
    };
    enter(journal, RobustnessStage::ModuleResolution);
    let (stage, result) = match probe.resolve() {
        Err(error) => (RobustnessStage::ModuleResolution, Err(error)),
        Ok(key) => {
            enter(journal, RobustnessStage::ModuleLoading);
            (RobustnessStage::ModuleLoading, probe.load(&key))
        }
    };
    let result = match result {
        Ok(()) => native(stage, Ok(())),
        Err(ProbeError::Rejected(message)) => native(stage, Err(message)),
        Err(ProbeError::Failure(message)) => RobustnessResult::Failure { stage, message },
    };
    match probe.finish() {
        Ok(()) => result,
        Err(message) => RobustnessResult::Failure { stage, message },
    }
}
enum CompilerInput<'a> {
    Source {
        source: &'a str,
        goal: DifferentialGoal,
    },
    Ir(&'a [u8]),
}

fn compile(
    input: &RobustnessInput,
    candidate: CompilerInput<'_>,
    execute: bool,
    journal: &mut Journal,
) -> RobustnessResult {
    let engine = Engine::new(RealmBuilder::new().build());
    let options = CompileOptions {
        filename: Some("lila-robustness.js".into()),
        module_loading_policy: ModuleLoadingPolicy::RejectAll,
        ..CompileOptions::default()
    };
    let artifact = match candidate {
        CompilerInput::Source { source, goal } => {
            let goal = match goal {
                DifferentialGoal::Script => lila_front::ParseGoal::Script,
                DifferentialGoal::Module => lila_front::ParseGoal::Module,
            };
            engine.inspect_compilation(source, goal, options, |stage| {
                enter(journal, stage_from_engine(stage))
            })
        }
        CompilerInput::Ir(bytes) => engine.inspect_ir_compilation(bytes, options, |stage| {
            enter(journal, stage_from_engine(stage))
        }),
    };
    let artifact = match artifact {
        Ok(artifact) => artifact,
        Err(failure) => {
            let stage = stage_from_engine(failure.stage());
            let error = failure.error();
            let message = error.to_string();
            if matches!(
                stage,
                RobustnessStage::IrInput | RobustnessStage::IrAdmission
            ) {
                return RobustnessResult::Rejected {
                    stage,
                    phase: RobustnessRejectionPhase::NativeBoundary,
                    message,
                };
            }
            if stage == RobustnessStage::Validation {
                return RobustnessResult::InvalidWasm { message };
            }
            if let Some(diagnostic) = error.parse_diagnostic() {
                return match diagnostic.kind() {
                    lila_front::ParseDiagnosticKind::MalformedJavaScript => {
                        RobustnessResult::Rejected {
                            stage,
                            phase: match diagnostic.phase() {
                                lila_front::ParseDiagnosticPhase::Parse => {
                                    RobustnessRejectionPhase::Parse
                                }
                                lila_front::ParseDiagnosticPhase::Early => {
                                    RobustnessRejectionPhase::EarlyError
                                }
                            },
                            message,
                        }
                    }
                    lila_front::ParseDiagnosticKind::UnsupportedParserFeature => {
                        RobustnessResult::Unsupported { stage, message }
                    }
                };
            }
            if let Some(diagnostic) = error
                .ir_diagnostic()
                .filter(|diagnostic| diagnostic.code().is_some())
            {
                return match diagnostic.phase() {
                    lila_ir::IrDiagnosticPhase::Early => RobustnessResult::Rejected {
                        stage,
                        phase: RobustnessRejectionPhase::EarlyError,
                        message,
                    },
                    lila_ir::IrDiagnosticPhase::Resolution => RobustnessResult::Rejected {
                        stage,
                        phase: RobustnessRejectionPhase::Resolution,
                        message,
                    },
                    lila_ir::IrDiagnosticPhase::Lowering => {
                        RobustnessResult::Unsupported { stage, message }
                    }
                };
            }
            if error.ir_diagnostic().is_some() || stage == RobustnessStage::RuntimeSetup {
                return RobustnessResult::Unsupported { stage, message };
            }
            return RobustnessResult::Failure { stage, message };
        }
    };
    if !execute {
        return RobustnessResult::Accepted {
            stage: RobustnessStage::Validation,
            artifact_bytes: Some(artifact.bytes.len() as u64),
        };
    }
    enter(journal, RobustnessStage::BuiltinExecution);
    match engine.profile_wasm_execution(&artifact, Some(input.timeout_ms()), false) {
        Ok(profile) => RobustnessResult::Executed {
            completion: match &profile.outcome.completion {
                lila_engine::ObservedCompletion::Normal(_) => RobustnessCompletionKind::Normal,
                lila_engine::ObservedCompletion::Throw(_) => RobustnessCompletionKind::Throw,
            },
            // Structured completion Debug never exposes heap addresses or user coercion.
            value: format!("{:?}", profile.outcome.completion),
        },
        Err(error) => RobustnessResult::Failure {
            stage: RobustnessStage::BuiltinExecution,
            message: error.to_string(),
        },
    }
}
fn stage_from_engine(stage: CompilerInspectionStage) -> RobustnessStage {
    match stage {
        CompilerInspectionStage::IrInput => RobustnessStage::IrInput,
        CompilerInspectionStage::Preparation => RobustnessStage::Preparation,
        CompilerInspectionStage::Lowering => RobustnessStage::Lowering,
        CompilerInspectionStage::IrAdmission => RobustnessStage::IrAdmission,
        CompilerInspectionStage::Emission => RobustnessStage::Emission,
        CompilerInspectionStage::RuntimeSetup => RobustnessStage::RuntimeSetup,
        CompilerInspectionStage::Validation => RobustnessStage::Validation,
    }
}
