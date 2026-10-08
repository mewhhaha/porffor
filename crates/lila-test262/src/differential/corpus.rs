//! Serial replay of every entry in one bounded native corpus inventory.

use super::{DifferentialError, DifferentialWorkerRunner, SpecExecOracle};
use crate::CompilerProvenance;
use serde::Serialize;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub const MAX_CORPUS_ENTRIES: usize = 128;
const MAX_INVENTORY_BYTES: u64 = 64 * 1024 * 1024;
const MAX_DIRECTORY_DEPTH: usize = 8;

#[derive(Debug)]
enum EntrySource {
    Compiled(&'static str),
    File(PathBuf),
}

#[derive(Debug)]
struct Entry {
    name: String,
    source: EntrySource,
}

/// Only complete, deterministic inventories can reach replay. A rejected
/// directory never becomes a smaller successful corpus.
#[derive(Debug)]
pub struct DifferentialCorpus {
    origin: String,
    entries: Vec<Entry>,
}

impl DifferentialCorpus {
    pub fn compiled() -> Self {
        macro_rules! entry {
            ($name:literal) => {
                Entry {
                    name: $name.into(),
                    source: EntrySource::Compiled(include_str!(concat!(
                        "../../tests/differential/",
                        $name
                    ))),
                }
            };
        }
        Self {
            origin: "compiled differential corpus".into(),
            entries: vec![
                entry!("v1/t25-foundation-arithmetic-self-check.json"),
                entry!("v1/t25-generated-integer-arithmetic-v1-seed-1.json"),
                entry!("v2/t25-foundation-primitive-number.json"),
                entry!("v3/t25-foundation-primitive-number-and-print.json"),
                entry!("v4/t25-computed-exact-attributes.json"),
                entry!("v4/t25-defer-order.json"),
                entry!("v4/t25-dynamic-parse-failure.json"),
                entry!("v4/t25-module-cycles-and-metadata.json"),
                entry!("v4/t25-script-self-import.json"),
                entry!("v4/t25-source-phase.json"),
                entry!("v4/t25-undeclared-and-unused.json"),
                entry!("v5/t25-object-graph-descriptors-and-realm.json"),
            ],
        }
    }

    pub fn from_directory(root: &Path) -> Result<Self, CorpusReplayError> {
        let origin = root
            .to_str()
            .ok_or_else(|| inventory("corpus root must be UTF-8"))?
            .to_owned();
        let metadata = fs::symlink_metadata(root).map_err(|error| inventory(error.to_string()))?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(inventory("corpus root must be an actual directory"));
        }
        let mut entries = Vec::new();
        let mut bytes = 0;
        discover(root, root, 0, &mut entries, &mut bytes)?;
        if entries.is_empty() {
            return Err(inventory("corpus contains no JSON entries"));
        }
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(Self { origin, entries })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn names(&self) -> impl ExactSizeIterator<Item = &str> {
        self.entries.iter().map(|entry| entry.name.as_str())
    }
}

fn inventory(message: impl Into<String>) -> CorpusReplayError {
    CorpusReplayError::Inventory(message.into())
}

fn discover(
    root: &Path,
    directory: &Path,
    depth: usize,
    entries: &mut Vec<Entry>,
    bytes: &mut u64,
) -> Result<(), CorpusReplayError> {
    if depth > MAX_DIRECTORY_DEPTH {
        return Err(inventory("corpus directory depth exceeds eight"));
    }
    for child in fs::read_dir(directory).map_err(|error| inventory(error.to_string()))? {
        let child = child.map_err(|error| inventory(error.to_string()))?;
        let path = child.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| inventory(error.to_string()))?;
        if metadata.file_type().is_symlink() {
            return Err(inventory(format!(
                "symlink is not a corpus entry: {}",
                path.display()
            )));
        }
        if metadata.is_dir() {
            discover(root, &path, depth + 1, entries, bytes)?;
        } else if metadata.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "json")
        {
            let relative = path
                .strip_prefix(root)
                .map_err(|error| inventory(error.to_string()))?;
            let name = relative
                .to_str()
                .ok_or_else(|| inventory("corpus entry must be UTF-8"))?
                .to_owned();
            if entries.len() == MAX_CORPUS_ENTRIES {
                return Err(inventory("corpus exceeds 128 entries"));
            }
            *bytes = bytes
                .checked_add(metadata.len())
                .ok_or_else(|| inventory("corpus inventory size overflow"))?;
            if *bytes > MAX_INVENTORY_BYTES {
                return Err(inventory("corpus inventory exceeds 64 MiB"));
            }
            entries.push(Entry {
                name,
                source: EntrySource::File(path),
            });
        } else if !metadata.is_file() {
            return Err(inventory(format!(
                "unsupported corpus entry: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum CorpusReplayError {
    Inventory(String),
    ExecutionUnavailable(String),
    Output { path: PathBuf, message: String },
}
impl fmt::Display for CorpusReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inventory(message) => write!(f, "corpus inventory: {message}"),
            Self::ExecutionUnavailable(message) => f.write_str(message),
            Self::Output { path, message } => {
                write!(f, "corpus output {}: {message}", path.display())
            }
        }
    }
}
impl std::error::Error for CorpusReplayError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusVerdict {
    Incomplete,
    AllMatched,
    ContainsFailures,
}

#[derive(Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum CaseState {
    Pending,
    Matched { report: String },
    Failed { report: String },
}

#[derive(Debug, Serialize)]
struct CaseSummary {
    ordinal: usize,
    entry: String,
    #[serde(flatten)]
    state: CaseState,
}

/// Created by the serial replay owner after writing each retained case report.
#[derive(Debug, Serialize)]
pub struct CorpusRunReport {
    schema_version: u32,
    controller_identity: CompilerProvenance,
    origin: String,
    total: usize,
    completed: usize,
    matched: usize,
    failed: usize,
    verdict: CorpusVerdict,
    semantic_equivalence: super::SemanticEquivalence,
    cases: Vec<CaseSummary>,
}
impl CorpusRunReport {
    pub const fn total(&self) -> usize {
        self.total
    }
    pub const fn completed(&self) -> usize {
        self.completed
    }
    pub const fn matched(&self) -> usize {
        self.matched
    }
    pub const fn failed(&self) -> usize {
        self.failed
    }
    pub const fn verdict(&self) -> CorpusVerdict {
        self.verdict
    }
    pub const fn is_green(&self) -> bool {
        matches!(self.verdict, CorpusVerdict::AllMatched)
    }
}

#[cfg(not(feature = "spec-exec-oracle"))]
pub fn replay_corpus(
    _corpus: DifferentialCorpus,
    _oracle: SpecExecOracle,
    _runner: &DifferentialWorkerRunner,
    _output: &Path,
) -> Result<CorpusRunReport, CorpusReplayError> {
    Err(CorpusReplayError::ExecutionUnavailable(
        DifferentialError::OracleNotLinked.to_string(),
    ))
}

#[cfg(feature = "spec-exec-oracle")]
pub fn replay_corpus(
    corpus: DifferentialCorpus,
    oracle: SpecExecOracle,
    runner: &DifferentialWorkerRunner,
    output: &Path,
) -> Result<CorpusRunReport, CorpusReplayError> {
    replay::run(corpus, oracle, runner, output)
}

#[cfg(feature = "spec-exec-oracle")]
mod replay {
    use super::super::{
        compare_observations, BackendObservation, DifferentialBackend, DifferentialReplayInput,
        DifferentialReport,
    };
    use super::*;
    use std::collections::BTreeSet;
    use std::io::{Read, Write};

    #[derive(Serialize)]
    #[serde(tag = "status", rename_all = "snake_case")]
    enum AttemptOutcome {
        Observed { observation: BackendObservation },
        Rejected { message: String },
    }
    #[derive(Serialize)]
    #[serde(tag = "status", rename_all = "snake_case")]
    enum CaseOutcome {
        NativeInputRejected {
            message: String,
        },
        Compared {
            report: DifferentialReport,
        },
        ReplayRejected {
            wasm_aot: AttemptOutcome,
            spec_exec: AttemptOutcome,
        },
    }
    impl CaseOutcome {
        fn is_green(&self) -> bool {
            match self {
                Self::Compared { report } => report.is_green(),
                Self::NativeInputRejected { .. } | Self::ReplayRejected { .. } => false,
            }
        }
    }
    #[derive(Serialize)]
    struct CaseRecord {
        schema_version: u32,
        ordinal: usize,
        entry: String,
        input_artifact: Option<String>,
        #[serde(flatten)]
        outcome: CaseOutcome,
    }
    fn output_error(path: &Path, error: impl fmt::Display) -> CorpusReplayError {
        CorpusReplayError::Output {
            path: path.into(),
            message: error.to_string(),
        }
    }
    fn write_new(path: &Path, bytes: &[u8]) -> Result<(), CorpusReplayError> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| output_error(path, error))?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| output_error(path, error))
    }
    fn write_json(path: &Path, value: &impl Serialize) -> Result<(), CorpusReplayError> {
        let bytes = serde_json::to_vec_pretty(value).map_err(|error| output_error(path, error))?;
        write_new(path, &bytes)
    }
    fn commit_aggregate(output: &Path, report: &CorpusRunReport) -> Result<(), CorpusReplayError> {
        let temporary = output.join("aggregate.tmp");
        write_json(&temporary, report)?;
        let destination = output.join("aggregate.json");
        fs::rename(&temporary, &destination).map_err(|error| output_error(&destination, error))
    }
    fn read_entry(entry: &Entry) -> Result<Vec<u8>, String> {
        match &entry.source {
            EntrySource::Compiled(source) => Ok(source.as_bytes().to_vec()),
            EntrySource::File(path) => {
                let file = fs::File::open(path).map_err(|error| error.to_string())?;
                let mut bytes = Vec::new();
                file.take(super::super::worker_process::MAX_REQUEST_BYTES as u64 + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|error| error.to_string())?;
                if bytes.len() > super::super::worker_process::MAX_REQUEST_BYTES {
                    return Err("corpus entry exceeds the 16 MiB worker request budget".into());
                }
                Ok(bytes)
            }
        }
    }
    fn replay_entry(
        input: &DifferentialReplayInput,
        oracle: SpecExecOracle,
        runner: &DifferentialWorkerRunner,
    ) -> CaseOutcome {
        // Both attempts are retained even when one admission/lifecycle returns
        // an error rather than an observation. No `?` discards the first one.
        let wasm = runner
            .run(input, DifferentialBackend::WasmAot, oracle)
            .map(|attempt| attempt.into_observation());
        let spec = runner
            .run(input, DifferentialBackend::SpecExec, oracle)
            .map(|attempt| attempt.into_observation());
        match (wasm, spec) {
            (Ok(wasm), Ok(spec)) => CaseOutcome::Compared {
                report: compare_observations(input, wasm, spec),
            },
            (wasm, spec) => {
                fn retained(
                    result: Result<BackendObservation, DifferentialError>,
                ) -> AttemptOutcome {
                    match result {
                        Ok(observation) => AttemptOutcome::Observed { observation },
                        Err(error) => AttemptOutcome::Rejected {
                            message: error.to_string(),
                        },
                    }
                }
                CaseOutcome::ReplayRejected {
                    wasm_aot: retained(wasm),
                    spec_exec: retained(spec),
                }
            }
        }
    }
    pub(super) fn run(
        corpus: DifferentialCorpus,
        oracle: SpecExecOracle,
        runner: &DifferentialWorkerRunner,
        output: &Path,
    ) -> Result<CorpusRunReport, CorpusReplayError> {
        let controller_identity =
            CompilerProvenance::current().map_err(CorpusReplayError::ExecutionUnavailable)?;
        fs::create_dir(output).map_err(|error| output_error(output, error))?;
        let cases = corpus
            .names()
            .enumerate()
            .map(|(ordinal, entry)| CaseSummary {
                ordinal,
                entry: entry.into(),
                state: CaseState::Pending,
            })
            .collect();
        let mut report = CorpusRunReport {
            schema_version: 1,
            controller_identity,
            origin: corpus.origin,
            total: corpus.entries.len(),
            completed: 0,
            matched: 0,
            failed: 0,
            verdict: CorpusVerdict::Incomplete,
            semantic_equivalence: super::super::SemanticEquivalence::NotEstablished,
            cases,
        };
        commit_aggregate(output, &report)?;
        let mut identifiers = BTreeSet::new();
        for (ordinal, entry) in corpus.entries.into_iter().enumerate() {
            let input_name = format!("case-{ordinal:03}.input.json");
            let (input_artifact, outcome) = match read_entry(&entry) {
                Err(message) => (None, CaseOutcome::NativeInputRejected { message }),
                Ok(bytes) => {
                    write_new(&output.join(&input_name), &bytes)?;
                    let decoded = std::str::from_utf8(&bytes)
                        .map_err(|error| error.to_string())
                        .and_then(|json| {
                            DifferentialReplayInput::from_json(json)
                                .map_err(|error| error.to_string())
                        });
                    let outcome = match decoded {
                        Err(message) => CaseOutcome::NativeInputRejected { message },
                        Ok(input) if !identifiers.insert(input.id().as_str().to_owned()) => {
                            CaseOutcome::NativeInputRejected {
                                message: format!("duplicate case id: {}", input.id().as_str()),
                            }
                        }
                        Ok(input) => replay_entry(&input, oracle, runner),
                    };
                    (Some(input_name), outcome)
                }
            };
            let green = outcome.is_green();
            let report_name = format!("case-{ordinal:03}.report.json");
            write_json(
                &output.join(&report_name),
                &CaseRecord {
                    schema_version: 1,
                    ordinal,
                    entry: entry.name,
                    input_artifact,
                    outcome,
                },
            )?;
            report.cases[ordinal].state = if green {
                report.matched += 1;
                CaseState::Matched {
                    report: report_name,
                }
            } else {
                report.failed += 1;
                CaseState::Failed {
                    report: report_name,
                }
            };
            report.completed += 1;
            if report.completed == report.total {
                report.verdict = if report.failed == 0 {
                    CorpusVerdict::AllMatched
                } else {
                    CorpusVerdict::ContainsFailures
                };
            }
            commit_aggregate(output, &report)?;
        }
        Ok(report)
    }
}
