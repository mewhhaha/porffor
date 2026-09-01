const BOA_EXPORT_AST: &str =
    include_str!("../../../vendor/boa_ast-0.21.1/src/declaration/export.rs");
const BOA_IMPORT_AST: &str =
    include_str!("../../../vendor/boa_ast-0.21.1/src/declaration/import.rs");
const BOA_MODULE_ITEMS: &str =
    include_str!("../../../vendor/boa_ast-0.21.1/src/module_item_list/mod.rs");
const BOA_SCOPE_ANALYZER: &str =
    include_str!("../../../vendor/boa_ast-0.21.1/src/scope_analyzer.rs");
const BOA_IMPORT_PARSER: &str =
    include_str!("../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/import.rs");
const BOA_EXPORT_PARSER: &str =
    include_str!("../../../vendor/boa_parser-0.21.1/src/parser/statement/declaration/export.rs");
const RECORD_SOURCE: &str = include_str!("../src/modules/record.rs");
const GRAPH_TESTS_SOURCE: &str = include_str!("../src/modules/graph_tests.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/module-request-identity.md");
const TASK: &str = include_str!("../../../tasks/12-modules-linking-loading.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn code_without_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn reexport_ast_can_only_own_an_evaluation_phase_request() {
    let request_domain = bounded(
        BOA_EXPORT_AST,
        "/// An evaluation-phase module request owned by a re-export.",
        "/// The kind of re-export",
    );
    let declaration = bounded(
        request_domain,
        "pub struct ReExportRequest {",
        "impl ReExportRequest {",
    );
    assert_eq!(
        code_without_whitespace(declaration),
        "request:ModuleRequest,}"
    );
    assert!(!declaration.contains("pub request"));

    let constructor = bounded(
        request_domain,
        "pub fn new(specifier: ModuleSpecifier, attributes: Box<[ImportAttribute]>) -> Self {",
        "/// Gets the evaluation-phase module request.",
    );
    assert_eq!(constructor.matches("ImportPhase::Evaluation").count(), 1);
    assert_eq!(
        constructor
            .matches("ModuleRequest::with_phase_and_attributes")
            .count(),
        1
    );
    assert_eq!(request_domain.matches("pub fn new(").count(), 1);
    assert_eq!(request_domain.matches("pub fn ").count(), 1);
    assert_eq!(
        request_domain
            .matches("pub const fn module_request(")
            .count(),
        1
    );
    assert!(!request_domain.contains("pub(crate) fn"));
    assert!(!request_domain.contains("impl From<"));
    assert!(!request_domain.contains("impl TryFrom<"));
    assert!(!request_domain.contains("impl Default"));
    assert!(!request_domain.contains("module_request_mut"));
    assert!(!request_domain.contains("Deref"));
    assert!(!request_domain.contains("arbitrary::Arbitrary"));

    assert!(request_domain.contains("serde(transparent)"));
    assert!(request_domain.contains("if request.phase() != ImportPhase::Evaluation"));
    assert!(request_domain.contains("Ok(Self::new("));
    assert!(!request_domain.contains("ImportPhase::Source"));
    assert!(!request_domain.contains("ImportPhase::Defer"));

    let declaration = bounded(BOA_EXPORT_AST, "    ReExport {", "    /// List of exports.");
    assert!(declaration.contains("request: ReExportRequest,"));
    assert!(!declaration.contains("request: ModuleRequest,"));
    assert!(!declaration.contains("ModuleSpecifier"));
    assert!(!declaration.contains("specifier:"));

    let visits = bounded(
        BOA_EXPORT_AST,
        "impl VisitWith for ExportDeclaration {",
        "/// Export specifier",
    );
    assert_eq!(
        visits.matches("Self::ReExport { request, kind }").count(),
        2
    );
    assert_eq!(visits.matches("request.visit_with(visitor)?;").count(), 1);
    assert_eq!(
        visits.matches("request.visit_with_mut(visitor)?;").count(),
        1
    );
    assert!(!visits.contains("visit_module_specifier(specifier)"));
    assert!(!visits.contains("visit_module_specifier_mut(specifier)"));
    assert!(request_domain.contains("impl VisitWith for ReExportRequest"));

    let module_request_visits = bounded(
        BOA_IMPORT_AST,
        "impl VisitWith for ModuleRequest {",
        "impl VisitWith for ImportKind {",
    );
    assert!(!module_request_visits.contains("phase"));
    assert_eq!(
        module_request_visits
            .matches("visitor.visit_module_specifier_mut(&mut self.specifier)?;")
            .count(),
        1
    );
    assert_eq!(
        module_request_visits
            .matches("attribute.visit_with_mut(visitor)?;")
            .count(),
        1
    );
}

#[test]
fn imports_and_reexports_share_attributes_but_only_the_typed_owner_selects_phase() {
    assert_eq!(
        BOA_IMPORT_PARSER.matches("parse_module_request(").count(),
        2
    );
    assert!(BOA_IMPORT_PARSER.contains("fn parse_module_request<R: ReadChar>("));
    assert!(!BOA_IMPORT_PARSER.contains("pub(super) fn parse_module_request<R: ReadChar>("));
    assert_eq!(
        BOA_IMPORT_PARSER
            .matches("parse_module_request_attributes(")
            .count(),
        2
    );
    assert!(BOA_IMPORT_PARSER.contains("pub(super) fn parse_re_export_request<R: ReadChar>("));
    assert!(BOA_IMPORT_PARSER.contains("Ok(AstReExportRequest::new(specifier, attributes))"));
    assert!(!BOA_IMPORT_PARSER.contains("parse_import_request"));

    assert_eq!(
        BOA_EXPORT_PARSER
            .matches("parse_re_export_request(")
            .count(),
        2
    );
    assert!(!BOA_EXPORT_PARSER.contains("parse_module_request("));
    assert!(!BOA_EXPORT_PARSER.contains("ImportPhase"));
    assert_eq!(
        BOA_EXPORT_PARSER
            .matches("AstExportDeclaration::ReExport")
            .count(),
        2
    );
    assert!(!BOA_EXPORT_PARSER.contains("parse_ignored_import_attributes"));
}

#[test]
fn every_field_sensitive_consumer_retains_the_typed_request() {
    assert_eq!(
        BOA_MODULE_ITEMS
            .matches("request.module_request().clone()")
            .count(),
        2
    );
    assert!(!BOA_MODULE_ITEMS.contains("ModuleRequest::from(*specifier)"));
    assert!(BOA_SCOPE_ANALYZER.contains("request.visit_with_mut(self)?;"));

    assert_eq!(
        RECORD_SOURCE
            .matches("module_request(interner, request.module_request())")
            .count(),
        2
    );
    assert!(!RECORD_SOURCE.contains("module_request(interner, request);"));
    assert!(!RECORD_SOURCE.contains("parse_ignored_import_attributes"));
    assert!(!RECORD_SOURCE.contains("ExportDeclaration::ReExport { kind, specifier }"));
}

#[test]
fn behavior_and_contract_witnesses_keep_attributed_reexports_load_bearing() {
    for witness in [
        "attributed_reexports_retain_each_request_and_entry_shape",
        "attributed_import_and_reexport_share_one_canonical_request",
    ] {
        assert!(
            RECORD_SOURCE.contains(witness),
            "missing record witness {witness}"
        );
    }
    assert!(GRAPH_TESTS_SOURCE
        .contains("an_attributed_reexport_uses_the_matching_public_host_resolution_row"));
    assert!(CONTRACT.contains("Attributed re-export retention"));
    assert!(CONTRACT.contains("private-field `ReExportRequest`"));
    assert!(TASK.contains("Attributed re-export requests retain"));
    assert!(TASK.contains("`ReExportRequest`"));
}
