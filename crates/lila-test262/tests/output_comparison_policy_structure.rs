use std::fs;
use std::path::Path;

const OWNER_SOURCE: &str = include_str!("../src/differential.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/differential-output-comparison-policy.md");
const TASK: &str = include_str!("../../../tasks/25-differential-fuzzing-performance.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn output_comparison_policy_is_the_exact_private_no_capability_domain() {
    let declaration = bounded(
        OWNER_SOURCE,
        "impl From<ObservationContract> for DifferentialProtocol {",
        "/// The complete program admitted by the current corpus protocols.",
    );
    assert_eq!(
        normalized(declaration),
        normalized(
            r#"
    fn from(contract: ObservationContract) -> Self {
        match contract {
            ObservationContract::SelfCheckingNoOutput => Self::V1SelfCheckingNoOutput,
            ObservationContract::PrimitiveCompletionNoOutput => Self::V2PrimitiveCompletionNoOutput,
            ObservationContract::PrimitiveCompletionPrintTranscript => {
                Self::V3PrimitiveCompletionPrintTranscript
            }
        }
    }
}

/// The complete output-observation policy selected by a protocol.
///
/// There is no caller-provided boolean/default. A new protocol row must choose
/// a policy exhaustively; a new policy row must define comparison exhaustively.
#[cfg(any(test, feature = "spec-exec-oracle"))]
enum OutputComparisonPolicy {
    RequireCapturedEmpty,
    CompareCapturedPrintTranscript,
}

"#,
        )
    );
    assert!(!OWNER_SOURCE.contains("pub enum OutputComparisonPolicy"));
    assert!(!OWNER_SOURCE.contains("pub(crate) enum OutputComparisonPolicy"));
    assert!(!OWNER_SOURCE.contains("impl OutputComparisonPolicy"));
    assert!(!OWNER_SOURCE.contains("for OutputComparisonPolicy"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "OutputComparisonPolicy"),
        7,
        "one declaration, two typed boundaries, two producers and two consumer rows own the policy"
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("OutputComparisonPolicy::RequireCapturedEmpty")
            .count(),
        2
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("OutputComparisonPolicy::CompareCapturedPrintTranscript")
            .count(),
        2
    );
}

#[test]
fn every_protocol_projects_one_exact_output_policy() {
    let projection = bounded(
        OWNER_SOURCE,
        "    const fn output_policy(self) -> OutputComparisonPolicy {",
        "\n}\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]\n#[serde(rename_all = \"snake_case\")]\npub enum DifferentialVerdict",
    );
    assert_eq!(
        normalized(projection),
        normalized(
            r#"
        match self {
            Self::V1SelfCheckingNoOutput | Self::V2PrimitiveCompletionNoOutput => {
                OutputComparisonPolicy::RequireCapturedEmpty
            }
            Self::V3PrimitiveCompletionPrintTranscript => {
                OutputComparisonPolicy::CompareCapturedPrintTranscript
            }
        }
    }
"#,
        )
    );
    assert!(!projection.contains("_ =>"));
    assert!(!projection.contains("Default"));
}

#[test]
fn replay_checks_the_exact_output_policy_before_projecting_backend_observations() {
    let comparison_prefix = bounded(
        OWNER_SOURCE,
        "fn compare_executions(",
        "    let verdict = if !output_policy_satisfied {",
    );
    assert_eq!(
        normalized(comparison_prefix),
        normalized(
            r#"
    case: &DifferentialCase,
    wasm_execution: BackendExecution,
    spec_execution: BackendExecution,
) -> DifferentialReport {
    let protocol = case.protocol();
    let output_policy_satisfied = obeys_output_policy(
        protocol.output_policy(),
        &wasm_execution.output_events,
        &spec_execution.output_events,
    );
    let wasm_disposition = wasm_execution.result.disposition();
    let spec_disposition = spec_execution.result.disposition();
    let wasm_aot = project_backend_execution(protocol, wasm_execution);
    let spec_exec = project_backend_execution(protocol, spec_execution);

"#,
        )
    );

    let consumer = bounded(
        OWNER_SOURCE,
        "fn obeys_output_policy(",
        "#[cfg(any(test, feature = \"spec-exec-oracle\"))]\nfn project_backend_execution(",
    );
    assert_eq!(
        normalized(consumer),
        normalized(
            r#"
    policy: OutputComparisonPolicy,
    wasm: &OutputEventsObservation,
    spec_exec: &OutputEventsObservation,
) -> bool {
    match policy {
        OutputComparisonPolicy::RequireCapturedEmpty => {
            matches!(wasm, OutputEventsObservation::Captured { events } if events.is_empty())
                && matches!(spec_exec, OutputEventsObservation::Captured { events } if events.is_empty())
        }
        OutputComparisonPolicy::CompareCapturedPrintTranscript => {
            matches!(wasm, OutputEventsObservation::Captured { .. })
                && matches!(spec_exec, OutputEventsObservation::Captured { .. })
        }
    }
}

"#,
        )
    );
    assert!(!consumer.contains("_ =>"));
}

#[test]
fn focused_policy_evidence_is_named_in_source_and_durable_docs() {
    for witness in [
        "fn either_backend_output_makes_a_no_output_case_red()",
        "fn v3_matches_primitive_completion_and_exact_ordered_print_transcript()",
    ] {
        assert_eq!(OWNER_SOURCE.matches(witness).count(), 1);
    }
    for evidence in [
        "OutputComparisonPolicy",
        "either_backend_output_makes_a_no_output_case_red",
        "v3_matches_primitive_completion_and_exact_ordered_print_transcript",
    ] {
        assert!(CONTRACT.contains(evidence));
        assert!(TASK.contains(evidence));
    }
}
