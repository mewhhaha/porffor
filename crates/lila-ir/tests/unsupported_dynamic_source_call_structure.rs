use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../src/lowering/dynamic_source.rs");
const CALL_CANDIDATE_SOURCE: &str = include_str!("../src/lowering/call_candidate_analysis.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/dynamic-source-capability.md");
const TASK: &str = include_str!("../../../tasks/13-dynamic-source-evaluation.md");

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
fn unsupported_dynamic_source_call_owns_the_exact_private_accounting_pair() {
    let declaration = normalized(bounded(
        SOURCE,
        "/// One-shot ownership of an unsupported dynamic-source invocation.",
        "/// Proof that the intrinsic `%eval%` call returns before parsing source.",
    ));
    assert_eq!(
        declaration,
        normalized(
            r#"
///
/// The fields stay private so the builtin-accounting identity and diagnostic
/// gap cannot be paired independently after target resolution.
pub(super) struct UnsupportedDynamicSourceCall {
    standard_builtin: Option<StandardBuiltinId>,
    gap: DynamicSourceGap,
}

"#,
        )
    );
    for forbidden in [
        "derive(",
        "pub standard_builtin",
        "pub gap",
        "impl Clone",
        "impl Copy",
    ] {
        assert!(!declaration.contains(&normalized(forbidden)));
    }
    assert_eq!(SOURCE.matches("UnsupportedDynamicSourceCall").count(), 5);
}

#[test]
fn resolution_produces_and_only_the_recorder_decomposes_the_accounting_pair() {
    let producer = normalized(bounded(
        SOURCE,
        "        let proof = source_args",
        "    pub(super) fn record_unsupported_dynamic_source(",
    ));
    assert_eq!(producer.matches("UnsupportedDynamicSourceCall{").count(), 1);
    assert_eq!(
        producer
            .matches("standard_builtin:StandardBuiltinId::from_function_id(function_id)")
            .count(),
        1
    );
    assert_eq!(
        producer
            .matches("gap:gap_for_source_proof(kind,proof)")
            .count(),
        1
    );

    let recorder = normalized(bounded(
        SOURCE,
        "    pub(super) fn record_unsupported_dynamic_source(",
        "    pub(super) fn lower_dynamic_source_construct(",
    ));
    assert_eq!(
        recorder,
        normalized(
            r#"
        &mut self,
        unsupported: UnsupportedDynamicSourceCall,
    ) {
        let UnsupportedDynamicSourceCall {
            standard_builtin,
            gap,
        } = unsupported;
        if let Some(builtin) = standard_builtin {
            self.note_standard_builtin_call(builtin);
        }
        self.diagnostics
            .push(IrDiagnostic::unsupported_dynamic_source(gap));
    }

"#,
        )
    );
    assert_eq!(
        SOURCE
            .matches("IrDiagnostic::unsupported_dynamic_source")
            .count(),
        1
    );
    assert_eq!(
        SOURCE.matches("let UnsupportedDynamicSourceCall {").count(),
        1
    );
    assert_eq!(
        count_in_rust_sources(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            "record_unsupported_dynamic_source(",
        ),
        7,
        "one recorder and six call sites must own every newly unsupported invocation"
    );
    let call_candidate_preflight = normalized(bounded(
        CALL_CANDIDATE_SOURCE,
        "    fn preflight_dynamic_source_call_candidates(",
        "    pub(super) fn preflight_function_prototype_call_dynamic_source(",
    ));
    let already_accounted = call_candidate_preflight
        .find("ifmatches!(source,CallCandidateSource::AlreadyAccounted)")
        .expect("already-accounted optional calls must have an explicit diagnostic branch");
    let discard = call_candidate_preflight[already_accounted..]
        .find("ValueInfo::undefined()")
        .expect("already-accounted unsupported calls must preserve the prior placeholder");
    let recorder = call_candidate_preflight[already_accounted..]
        .find("self.record_unsupported_dynamic_source(unsupported)")
        .expect("syntax-owned unsupported calls must record their diagnostic");
    assert!(discard < recorder);
}

#[test]
fn contract_and_t13_record_one_shot_unsupported_accounting_ownership() {
    let contract_words = CONTRACT.split_whitespace().collect::<Vec<_>>().join(" ");
    let task_words = TASK.split_whitespace().collect::<Vec<_>>().join(" ");
    for marker in [
        "private, non-`Clone`, non-`Copy` `UnsupportedDynamicSourceCall`",
        "builtin-accounting identity and `DynamicSourceGap`",
        "sole recorder decomposes",
        "source-equivalent unsupported-accounting closure",
    ] {
        assert!(
            contract_words.contains(marker),
            "missing contract marker: {marker}"
        );
        assert!(task_words.contains(marker), "missing T13 marker: {marker}");
    }
}
