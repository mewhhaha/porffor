const SOURCE: &str = include_str!("../src/lib.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/test262-negative-phase-authority.md");
const TASK: &str = include_str!("../../../tasks/26-zero-failure-conformance-closure.md");

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn negative_expectations_carry_the_closed_phase_authority() {
    let source = compact(SOURCE);

    assert!(source.contains("pubenumNegativePhase{Parse,Early,Resolution,Runtime,}"));
    assert!(source
        .contains("pubstructNegativeExpectation{pubphase:NegativePhase,puberror_type:String,}"));
    assert!(!source.contains("pubphase:String"));
    assert!(!source.contains("NegativePhase::of"));
}

#[test]
fn discovery_is_the_only_free_text_phase_boundary() {
    let source = compact(SOURCE);

    assert!(source.contains(
        "fnparse_negative(test_path:&str,value:Option<&str>,)->Result<Option<NegativeExpectation>,String>"
    ));
    assert!(source.contains(
        ".map(NegativePhase::parse).transpose().map_err(|err|format!(\"invalidTest262negativemetadatafor{test_path}:{err}\"))?.unwrap_or(NegativePhase::Runtime)"
    ));
    assert!(source.contains(
        "letnegative=parse_negative(&path,frontmatter.get(\"negative\").map(String::as_str))?.map(Arc::new);"
    ));
    assert!(source.contains("unknownnegative.phase`{value}`"));
}

#[test]
fn every_phase_decision_consumes_the_enum_exhaustively() {
    let source = compact(SOURCE);

    assert!(source.contains(
        "pubconstfnas_str(self)->&'staticstr{matchself{Self::Parse=>\"parse\",Self::Early=>\"early\",Self::Resolution=>\"resolution\",Self::Runtime=>\"runtime\",}}"
    ));
    assert!(source.contains(
        "constfnis_compile_only(self)->bool{matchself{Self::Parse|Self::Early|Self::Resolution=>true,Self::Runtime=>false,}}"
    ));
    for phase_owner in [
        "Self::Parse=>FailureKind::Parser",
        "Self::Early|Self::Resolution=>FailureKind::EarlyError",
        "Self::Runtime=>FailureKind::Runtime",
    ] {
        assert!(
            source.contains(phase_owner),
            "missing phase owner: {phase_owner}"
        );
    }
    assert!(source.contains(".is_some_and(|negative|negative.phase.is_compile_only())"));
    assert!(source.contains("letexpected=negative.phase;"));
    assert!(source.contains("negative.phase.failure_kind()"));
    assert!(source.contains("Some(negative.phase.as_str().to_string())"));
}

#[test]
fn contract_and_task_record_the_negative_phase_invariant() {
    for required in [
        "NegativeExpectation.phase: NegativePhase",
        "unknown\n`negative.phase`",
        "compile error",
        "NegativePhase::Runtime",
    ] {
        assert!(
            CONTRACT.contains(required),
            "missing contract evidence: {required}"
        );
    }

    assert!(TASK.contains("NegativePhase::{Parse, Early, Resolution, Runtime}"));
    assert!(TASK.contains("negative_phase_authority_structure"));
    assert!(TASK.contains("complete current evidence or advance the T26 release gate"));
}
