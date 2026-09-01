const DIFFERENTIAL_SOURCE: &str = include_str!("../src/differential.rs");
const HARNESS_SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/differential-oracle-compile-boundary.md");
const TASK: &str = include_str!("../../../tasks/25-differential-fuzzing-performance.md");

const ORACLE_GATE: &str = "#[cfg(any(test,feature=\"spec-exec-oracle\"))]";

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn replay_and_comparison_machinery_requires_test_or_oracle_capability() {
    let source = normalized(DIFFERENTIAL_SOURCE);
    for declaration in [
        "uselila_engine::{CompileOptions,ModuleLoadingPolicy,ObservedCompletion,ObservedJsValue};",
        "implDifferentialGoal{",
        "enumOutputComparisonPolicy{",
        "implDifferentialBackend{",
        "implExecutionDisposition{",
        "implCompletionKindObservation{",
        "implPrimitiveValueObservation{",
        "fnvalue(&self)->&PrimitiveValueObservation{",
        "implUnsupportedObservedValueType{",
        "implFailurePhase{",
        "implOutputUnavailableReason{",
        "constfnoutput_policy(self)->OutputComparisonPolicy{",
        "#[derive(Debug)]structBackendExecution{",
        "#[derive(Debug)]enumBackendExecutionResult{",
        "implBackendExecutionResult{",
        "constfnexecution_failure_phase(backend:DifferentialBackend)->FailurePhase{",
        "fncompile_options_for_case(case:&DifferentialCase)->CompileOptions{",
        "fncompare_executions(",
        "fnobeys_output_policy(",
        "fnproject_backend_execution(",
        "fnproject_primitive_completion(",
        "constfncompare_v1_dispositions(",
        "fncompare_v2_observations(",
        "fncompare_v3_observations(",
        "fnv2_execution_signature(",
        "fnv3_mismatch_signature(",
        "fnv3_backend_observation_signature(",
        "fnprimitive_value_signature(",
        "constFNV_OFFSET_BASIS:u64=",
        "fnfnv_update(",
        "fnfnv_field(",
        "fncase_fingerprint(",
    ] {
        let offset = source
            .find(declaration)
            .unwrap_or_else(|| panic!("missing `{declaration}`"));
        assert!(
            source[..offset].ends_with(ORACLE_GATE),
            "`{declaration}` is outside the oracle compile boundary"
        );
    }
    assert_eq!(source.matches(ORACLE_GATE).count(), 32);
}

#[test]
fn test_only_mutation_and_feature_only_loader_have_explicit_boundaries() {
    let harness = normalized(HARNESS_SOURCE);
    assert!(harness.contains("#[cfg(test)]fnvalues_mut(&mutself)->Option<&mutVec<T>>{"));
    assert!(!harness.contains("fnskip_template_source("));

    let differential = normalized(DIFFERENTIAL_SOURCE);
    assert!(differential
        .contains("#[cfg(feature=\"spec-exec-oracle\")]fnmodule_loader_context_sources("));
    assert!(differential.contains("#[cfg(not(feature=\"spec-exec-oracle\"))]pubfnreplay_case("));
}

#[test]
fn boundary_has_frozen_source_evidence() {
    for evidence in [CONTRACT, TASK] {
        for hash in [
            "8ed6a8721c8d157ea263418918138258a2e68a26670059923570f814b293b69e",
            "bcecce80a7145d8c00525efc0bbfe0ec3b3a7110a6b7f8aa1590706231d21a89",
        ] {
            assert!(evidence.contains(hash));
        }
        assert!(evidence.contains("default product build"));
    }
}
