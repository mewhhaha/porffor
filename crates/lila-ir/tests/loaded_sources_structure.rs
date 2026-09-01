const MODULES_SOURCE: &str = include_str!("../src/modules/mod.rs");
const OWNER_SOURCE: &str = include_str!("../src/modules/loaded_sources.rs");
const MODULE_KEY_SOURCE: &str = include_str!("../src/modules/module_key.rs");
const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");
const GRAPH_TESTS_SOURCE: &str = include_str!("../src/modules/graph_tests.rs");
const GRAPH_BUILD_SOURCE: &str = include_str!("../src/modules/graph_build.rs");
const GRAPH_RESOLUTION_SOURCE: &str = include_str!("../src/modules/graph_resolution.rs");
const LINK_SOURCE: &str = include_str!("../src/modules/link.rs");
const NAMESPACE_SOURCE: &str = include_str!("../src/modules/namespace.rs");
const DYNAMIC_SOURCE: &str = include_str!("../src/modules/dynamic.rs");
const LOWERING_SOURCE: &str = include_str!("../src/lowering.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const ENGINE_LOADER_SOURCE: &str = include_str!("../../lila-engine/src/module_loader.rs");
const ENGINE_LIB_SOURCE: &str = include_str!("../../lila-engine/src/lib.rs");

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
fn loaded_sources_have_one_private_owner_and_narrow_public_facade() {
    assert_eq!(MODULES_SOURCE.matches("\nmod loaded_sources;\n").count(), 1);
    assert_eq!(
        MODULES_SOURCE
            .matches("pub use loaded_sources::{ModuleGraphSources, ModuleSourceIr};")
            .count(),
        1
    );
    assert!(!MODULES_SOURCE.contains("\npub mod loaded_sources;\n"));
    assert!(!MODULES_SOURCE.contains("\nmod loaded_sources {\n"));
    assert!(!GRAPH_SOURCE.contains("pub struct ModuleSourceIr"));
    assert!(!GRAPH_SOURCE.contains("enum ModuleParse"));
    assert!(!GRAPH_SOURCE.contains("pub struct ModuleGraphSources"));
    assert!(!GRAPH_SOURCE.contains("impl ModuleGraphSources"));
    assert!(!GRAPH_SOURCE.contains("ANONYMOUS_MODULE_KEY"));
    assert!(!GRAPH_TESTS_SOURCE.contains("pub struct ModuleSourceIr"));
    assert!(!GRAPH_TESTS_SOURCE.contains("enum ModuleParse"));
    assert!(!GRAPH_TESTS_SOURCE.contains("pub struct ModuleGraphSources"));
    assert!(!GRAPH_TESTS_SOURCE.contains("impl ModuleGraphSources"));
    assert!(!GRAPH_TESTS_SOURCE.contains("ANONYMOUS_MODULE_KEY"));
    assert_eq!(OWNER_SOURCE.matches("pub struct ModuleSourceIr").count(), 1);
    assert_eq!(OWNER_SOURCE.matches("enum ModuleParse").count(), 1);
    assert_eq!(
        OWNER_SOURCE
            .matches("pub struct ModuleGraphSources")
            .count(),
        1
    );
    assert_eq!(OWNER_SOURCE.matches("impl ModuleGraphSources").count(), 1);
    assert!(!OWNER_SOURCE.contains("super::graph"));
    assert_eq!(LIB_SOURCE.matches("ModuleSourceIr").count(), 1);
    assert_eq!(LIB_SOURCE.matches("ModuleGraphSources").count(), 1);
}

#[test]
fn module_source_keeps_private_parse_state_and_the_exact_public_method_inventory() {
    let fields = bounded(
        OWNER_SOURCE,
        "pub struct ModuleSourceIr {",
        "#[derive(Debug, Clone, PartialEq, Eq)]\npub(super) enum ModuleParse",
    );
    assert_eq!(
        code_without_whitespace(fields),
        "key:ModuleKey,meta_url:String,pub(super)parse:ModuleParse,}"
    );

    let parse = bounded(
        OWNER_SOURCE,
        "pub(super) enum ModuleParse {",
        "impl ModuleSourceIr",
    );
    assert_eq!(
        code_without_whitespace(parse),
        "Module(ParsedModule),ScriptEntry(ParsedScript),Rejected{\
         source:SourceUnit,error:lila_front::ParseError,},}"
    );
    assert!(!OWNER_SOURCE.contains("pub enum ModuleParse"));
    assert_eq!(
        OWNER_SOURCE.matches("pub(super) enum ModuleParse").count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("pub(super) parse: ModuleParse")
            .count(),
        1
    );

    for method in [
        "new",
        "from_parsed",
        "from_parsed_script",
        "key",
        "source_text",
        "meta_url",
        "module_requests",
        "goal",
    ] {
        assert_eq!(
            OWNER_SOURCE.matches(&format!("pub fn {method}(")).count(),
            1,
            "{method} must have one public owner"
        );
    }
    assert_eq!(OWNER_SOURCE.matches("pub fn ").count(), 9);
    assert_eq!(OWNER_SOURCE.matches("#[doc(hidden)]").count(), 1);
    assert_eq!(
        OWNER_SOURCE.matches("scan_module_requests(source)").count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("scan_script_module_requests(source)")
            .count(),
        1
    );
}

#[test]
fn graph_sources_keep_the_exact_public_closure_record_and_single_constructor() {
    let fields = bounded(
        OWNER_SOURCE,
        "pub struct ModuleGraphSources {",
        "impl ModuleGraphSources",
    );
    assert_eq!(
        code_without_whitespace(fields),
        "pubmodules:Vec<ModuleSourceIr>,pubentry:ModuleUnitId,\
         pubresolutions:Vec<(ModuleUnitId,ModuleRequestKeyIr,ModuleUnitId)>,}"
    );
    assert_eq!(OWNER_SOURCE.matches("pub fn single(").count(), 1);
    assert!(OWNER_SOURCE.contains(".unwrap_or_else(|| ANONYMOUS_MODULE_KEY.to_string())"));
    assert!(OWNER_SOURCE.contains("ModuleKey::from_host(key.clone())"));
    assert_eq!(
        MODULE_KEY_SOURCE
            .matches("pub const ANONYMOUS_MODULE_KEY: &str = \"<entry>\";")
            .count(),
        1
    );
    assert_eq!(
        MODULES_SOURCE
            .matches("pub use module_key::{ModuleKey, ANONYMOUS_MODULE_KEY};")
            .count(),
        1
    );
}

#[test]
fn loaded_source_callers_use_the_facade_while_construction_has_one_private_owner() {
    assert_eq!(OWNER_SOURCE.matches("ModuleGraphSources").count(), 4);
    assert_eq!(GRAPH_SOURCE.matches("ModuleGraphSources").count(), 2);
    assert_eq!(GRAPH_TESTS_SOURCE.matches("ModuleGraphSources").count(), 35);
    assert_eq!(GRAPH_BUILD_SOURCE.matches("ModuleGraphSources").count(), 2);
    assert_eq!(LINK_SOURCE.matches("ModuleGraphSources").count(), 4);
    assert_eq!(NAMESPACE_SOURCE.matches("ModuleGraphSources").count(), 2);
    assert_eq!(DYNAMIC_SOURCE.matches("ModuleGraphSources").count(), 3);
    assert_eq!(LOWERING_SOURCE.matches("ModuleGraphSources").count(), 6);
    assert_eq!(
        ENGINE_LOADER_SOURCE.matches("ModuleGraphSources").count(),
        6
    );
    assert_eq!(ENGINE_LIB_SOURCE.matches("ModuleGraphSources").count(), 6);

    assert_eq!(OWNER_SOURCE.matches("ModuleSourceIr").count(), 4);
    assert_eq!(GRAPH_SOURCE.matches("ModuleSourceIr").count(), 1);
    assert_eq!(GRAPH_TESTS_SOURCE.matches("ModuleSourceIr").count(), 67);
    assert_eq!(LINK_SOURCE.matches("ModuleSourceIr").count(), 1);
    assert_eq!(NAMESPACE_SOURCE.matches("ModuleSourceIr").count(), 2);
    assert_eq!(DYNAMIC_SOURCE.matches("ModuleSourceIr").count(), 1);
    assert_eq!(ENGINE_LOADER_SOURCE.matches("ModuleSourceIr").count(), 8);

    assert!(GRAPH_BUILD_SOURCE.contains("pub(crate) fn build_graph("));
    assert!(!GRAPH_SOURCE.contains("pub(crate) fn build_graph("));
    assert!(GRAPH_SOURCE.contains("pub(crate) fn link(graph: &mut ModuleGraphIr)"));
    assert!(GRAPH_RESOLUTION_SOURCE.contains("pub fn resolve_export("));
    assert!(!GRAPH_SOURCE.contains("pub fn resolve_export("));
}
