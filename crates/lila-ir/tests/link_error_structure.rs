const MODULES_SOURCE: &str = include_str!("../src/modules/mod.rs");
const OWNER_SOURCE: &str = include_str!("../src/modules/link_error.rs");
const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");
const GRAPH_TESTS_SOURCE: &str = include_str!("../src/modules/graph_tests.rs");
const GRAPH_CLASSIFICATION_SOURCE: &str =
    include_str!("../src/modules/graph_evaluation_classification.rs");
const GRAPH_BUILD_SOURCE: &str = include_str!("../src/modules/graph_build.rs");
const GRAPH_RESOLUTION_SOURCE: &str = include_str!("../src/modules/graph_resolution.rs");
const EARLY_SOURCE: &str = include_str!("../src/modules/early.rs");
const RECORD_SOURCE: &str = include_str!("../src/modules/record.rs");
const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");

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
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn module_link_error_has_one_private_owner_and_narrow_public_facade() {
    assert_eq!(MODULES_SOURCE.matches("\nmod link_error;\n").count(), 1);
    assert_eq!(
        MODULES_SOURCE
            .matches("pub use link_error::ModuleLinkErrorIr;")
            .count(),
        1
    );
    assert!(!MODULES_SOURCE.contains("\npub mod link_error;\n"));
    assert!(!MODULES_SOURCE.contains("\nmod link_error {\n"));
    assert!(!GRAPH_SOURCE.contains("pub enum ModuleLinkErrorIr"));
    assert!(!GRAPH_SOURCE.contains("impl ModuleLinkErrorIr"));
    assert!(!GRAPH_TESTS_SOURCE.contains("pub enum ModuleLinkErrorIr"));
    assert!(!GRAPH_TESTS_SOURCE.contains("impl ModuleLinkErrorIr"));
    assert_eq!(
        OWNER_SOURCE.matches("pub enum ModuleLinkErrorIr").count(),
        1
    );
    assert_eq!(OWNER_SOURCE.matches("impl ModuleLinkErrorIr").count(), 1);
    assert_eq!(LIB_SOURCE.matches("ModuleLinkErrorIr").count(), 1);
    assert!(OWNER_SOURCE.contains("use super::module_key::ModuleKey;"));
    assert!(!OWNER_SOURCE.contains("super::graph::ModuleKey"));
    assert!(OWNER_SOURCE.contains("[`crate::ModuleEvaluationModeIr`]"));
    assert!(!OWNER_SOURCE.contains("allow(unused_imports)"));
}

#[test]
fn module_link_error_preserves_the_closed_eight_variant_domain() {
    let variants = bounded(
        OWNER_SOURCE,
        "pub enum ModuleLinkErrorIr {",
        "impl ModuleLinkErrorIr",
    );
    assert_eq!(
        code_without_whitespace(variants),
        "UnresolvedModule{referrer:ModuleUnitId,request:ModuleRequestKeyIr,},\
         MissingExport{referrer:ModuleUnitId,request:ModuleRequestIr,import_name:ExportName,},\
         AmbiguousExport{module:ModuleUnitId,export_name:ExportName,},\
         DuplicateExport{module:ModuleUnitId,export_name:ExportName,},\
         InconsistentLoad{key:ModuleKey,},\
         InconsistentResolution{referrer:ModuleUnitId,request:ModuleRequestKeyIr,},\
         TooManyUnits{count:usize,},\
         UnsupportedPhase{module:ModuleUnitId,phase:ImportPhaseIr,reason:String,},}"
    );
}

#[test]
fn module_link_error_keeps_exhaustive_code_message_and_diagnostic_projections() {
    assert_eq!(OWNER_SOURCE.matches("pub const fn code(&self)").count(), 1);
    assert_eq!(OWNER_SOURCE.matches("pub fn message(&self)").count(), 1);
    assert_eq!(
        OWNER_SOURCE.matches("pub fn to_diagnostic(&self)").count(),
        1
    );

    let code_projection = bounded(
        OWNER_SOURCE,
        "pub const fn code(&self) -> EarlyErrorCode {",
        "/// Human-readable message.",
    );
    assert_eq!(code_projection.matches("Self::").count(), 8);

    let message_projection = bounded(
        OWNER_SOURCE,
        "pub fn message(&self) -> String {",
        "/// The diagnostic this error becomes on `ProgramIr`.",
    );
    assert_eq!(message_projection.matches("Self::").count(), 8);
    assert!(!OWNER_SOURCE.contains("_ =>"));

    let diagnostic_projection = OWNER_SOURCE
        .split_once("pub fn to_diagnostic(&self) -> IrDiagnostic {")
        .expect("diagnostic projection")
        .1;
    assert_eq!(
        code_without_whitespace(diagnostic_projection),
        "IrDiagnostic::rejected(self.code(),self.message(),None)}}"
    );
}

#[test]
fn graph_build_early_and_lowering_keep_their_existing_error_roles() {
    assert_eq!(OWNER_SOURCE.matches("ModuleLinkErrorIr").count(), 2);
    assert_eq!(GRAPH_SOURCE.matches("ModuleLinkErrorIr").count(), 9);
    assert_eq!(GRAPH_TESTS_SOURCE.matches("ModuleLinkErrorIr").count(), 7);
    assert_eq!(
        GRAPH_CLASSIFICATION_SOURCE
            .matches("ModuleLinkErrorIr")
            .count(),
        3
    );
    assert_eq!(GRAPH_BUILD_SOURCE.matches("ModuleLinkErrorIr").count(), 4);
    assert_eq!(EARLY_SOURCE.matches("ModuleLinkErrorIr").count(), 2);
    assert_eq!(RECORD_SOURCE.matches("ModuleLinkErrorIr").count(), 1);
    assert_eq!(LOWERING_SOURCE.matches("ModuleLinkErrorIr").count(), 1);
    assert_eq!(
        GRAPH_RESOLUTION_SOURCE
            .matches("pub fn resolve_export(")
            .count(),
        1
    );
    assert_eq!(
        GRAPH_RESOLUTION_SOURCE
            .matches("fn resolve_export_inner(")
            .count(),
        1
    );
    assert!(!GRAPH_SOURCE.contains("fn resolve_export"));
    assert!(!GRAPH_TESTS_SOURCE.contains("fn resolve_export"));
    assert!(EARLY_SOURCE.contains("let error = ModuleLinkErrorIr::DuplicateExport {"));
    assert!(EARLY_SOURCE.contains("error.message(),"));
    assert!(LOWERING_SOURCE.contains(".map(ModuleLinkErrorIr::to_diagnostic)"));
}
