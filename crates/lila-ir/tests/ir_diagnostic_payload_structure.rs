use lila_front::EarlyErrorCode;
use lila_ir::{
    DynamicSourceGap, DynamicSourceKind, IrDiagnostic, IrDiagnosticKind, IrDiagnosticPhase,
    NativeErrorKind, UnsupportedFeature,
};

const DIAGNOSTICS: &str = include_str!("../src/diagnostics.rs");
const IR: &str = include_str!("../src/ir.rs");
const MODULE_EARLY: &str = include_str!("../src/modules/early.rs");
const MODULE_GRAPH_TESTS: &str = include_str!("../src/modules/graph_tests.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/ir-diagnostic-payload-authority.md");
const TASK: &str = include_str!("../../../tasks/07-parser-grammar-early-errors.md");

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

#[test]
fn diagnostic_stores_one_private_closed_payload() {
    let diagnostic = normalized(bounded(
        DIAGNOSTICS,
        "pub struct IrDiagnostic {",
        "#[derive(Debug, Clone, PartialEq, Eq)]\nenum IrDiagnosticPayload",
    ));
    assert_eq!(
        diagnostic,
        "payload:IrDiagnosticPayload,pubspan:Option<SourceSpan>,pubmessage:String,}"
    );

    let payload = normalized(bounded(
        DIAGNOSTICS,
        "enum IrDiagnosticPayload {",
        "impl IrDiagnostic",
    ));
    assert!(payload.starts_with(concat!(
        "Rejected(EarlyErrorCode),",
        "Unsupported,",
        "UnsupportedFeature(UnsupportedFeature),",
        "Lowering,",
        "}"
    )));
    assert!(!DIAGNOSTICS.contains("pub enum IrDiagnosticPayload"));
    assert!(!DIAGNOSTICS.contains("Copy, PartialEq, Eq)]\nenum IrDiagnosticPayload"));
    assert!(!DIAGNOSTICS.contains("pub kind: IrDiagnosticKind"));
    assert!(!DIAGNOSTICS.contains("code: Option<EarlyErrorCode>"));
    assert!(!DIAGNOSTICS.contains("unsupported_feature: Option<UnsupportedFeature>"));
}

#[test]
fn diagnostic_payload_projections_are_exhaustive() {
    for (start, end) in [
        (
            "pub const fn kind(&self) -> IrDiagnosticKind {",
            "/// The condition this diagnostic reports",
        ),
        (
            "pub const fn code(&self) -> Option<EarlyErrorCode> {",
            "/// The closed compiler capability",
        ),
        (
            "pub const fn unsupported_feature(&self) -> Option<UnsupportedFeature> {",
            "#[must_use]\n    pub const fn phase",
        ),
    ] {
        let projection = bounded(DIAGNOSTICS, start, end);
        assert!(projection.contains("match &self.payload"), "{start}");
        assert!(!normalized(projection).contains("_=>"), "{start}");
    }

    assert!(DIAGNOSTICS.contains("self.kind().phase()"));
    assert!(DIAGNOSTICS.contains("self.kind().error_type()"));
}

#[test]
fn every_diagnostic_kind_consumer_uses_the_projection() {
    for source in [IR, MODULE_EARLY, MODULE_GRAPH_TESTS] {
        assert!(!source.contains("diagnostic.kind,"));
        assert!(!source.contains("diagnostic.kind =="));
    }
    assert!(IR.contains("diagnostic.kind(),"));
    assert!(MODULE_EARLY.contains("diagnostic.kind()"));
    assert!(MODULE_GRAPH_TESTS.contains("diagnostic.kind()"));
}

#[test]
fn constructors_project_only_their_owned_classification() {
    let rejected = IrDiagnostic::rejected(
        EarlyErrorCode::ObjectDuplicateProto,
        "duplicate prototype",
        None,
    );
    assert_eq!(rejected.kind(), IrDiagnosticKind::EarlyError);
    assert_eq!(rejected.code(), Some(EarlyErrorCode::ObjectDuplicateProto));
    assert_eq!(rejected.unsupported_feature(), None);
    assert_eq!(rejected.phase(), IrDiagnosticPhase::Early);
    assert_eq!(rejected.error_type(), Some(NativeErrorKind::SyntaxError));

    let unsupported = IrDiagnostic::unsupported("compiler gap");
    assert_eq!(unsupported.kind(), IrDiagnosticKind::Unsupported);
    assert_eq!(unsupported.code(), None);
    assert_eq!(unsupported.unsupported_feature(), None);
    assert_eq!(unsupported.phase(), IrDiagnosticPhase::Lowering);
    assert_eq!(unsupported.error_type(), None);

    let gap = DynamicSourceGap::runtime_source(DynamicSourceKind::DirectEval);
    let unsupported_feature = IrDiagnostic::unsupported_dynamic_source(gap);
    assert_eq!(unsupported_feature.kind(), IrDiagnosticKind::Unsupported);
    assert_eq!(unsupported_feature.code(), None);
    assert_eq!(
        unsupported_feature.unsupported_feature(),
        Some(UnsupportedFeature::DynamicSource(gap))
    );
    assert_eq!(unsupported_feature.phase(), IrDiagnosticPhase::Lowering);
    assert_eq!(unsupported_feature.error_type(), None);

    let lowering = IrDiagnostic::lowering("lowering failed");
    assert_eq!(lowering.kind(), IrDiagnosticKind::Lowering);
    assert_eq!(lowering.code(), None);
    assert_eq!(lowering.unsupported_feature(), None);
    assert_eq!(lowering.phase(), IrDiagnosticPhase::Lowering);
    assert_eq!(lowering.error_type(), None);

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("IrDiagnosticPayload"));
        assert!(evidence.contains("diagnostic.kind()"));
    }
}
