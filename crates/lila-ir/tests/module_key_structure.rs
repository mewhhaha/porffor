const MODULES_SOURCE: &str = include_str!("../src/modules/mod.rs");
const OWNER_SOURCE: &str = include_str!("../src/modules/module_key.rs");
const LOADED_SOURCES_SOURCE: &str = include_str!("../src/modules/loaded_sources.rs");
const GRAPH_SOURCE: &str = include_str!("../src/modules/graph.rs");
const GRAPH_TESTS_SOURCE: &str = include_str!("../src/modules/graph_tests.rs");
const GRAPH_BUILD_SOURCE: &str = include_str!("../src/modules/graph_build.rs");
const GRAPH_RESOLUTION_SOURCE: &str = include_str!("../src/modules/graph_resolution.rs");
const RECORD_SOURCE: &str = include_str!("../src/modules/record.rs");
const DYNAMIC_SOURCE: &str = include_str!("../src/modules/dynamic.rs");
const EARLY_SOURCE: &str = include_str!("../src/modules/early.rs");
const LINK_SOURCE: &str = include_str!("../src/modules/link.rs");
const LINK_ERROR_SOURCE: &str = include_str!("../src/modules/link_error.rs");
const NAMESPACE_SOURCE: &str = include_str!("../src/modules/namespace.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const ENGINE_LOADER_SOURCE: &str = include_str!("../../lila-engine/src/module_loader.rs");
const ENGINE_LIB_SOURCE: &str = include_str!("../../lila-engine/src/lib.rs");

fn code_without_whitespace(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn module_key_has_one_private_owner_and_narrow_public_facade() {
    assert_eq!(MODULES_SOURCE.matches("\nmod module_key;\n").count(), 1);
    assert_eq!(
        MODULES_SOURCE
            .matches("pub use module_key::{ModuleKey, ANONYMOUS_MODULE_KEY};")
            .count(),
        1
    );
    assert!(!MODULES_SOURCE.contains("\npub mod module_key;\n"));
    assert!(!MODULES_SOURCE.contains("\nmod module_key {\n"));
    assert!(!GRAPH_SOURCE.contains("pub struct ModuleKey"));
    assert!(!GRAPH_SOURCE.contains("impl ModuleKey"));
    assert!(!GRAPH_TESTS_SOURCE.contains("pub struct ModuleKey"));
    assert!(!GRAPH_TESTS_SOURCE.contains("impl ModuleKey"));
    assert_eq!(OWNER_SOURCE.matches("pub struct ModuleKey").count(), 1);
    assert_eq!(OWNER_SOURCE.matches("impl ModuleKey").count(), 1);
    assert_eq!(
        OWNER_SOURCE
            .matches("pub const ANONYMOUS_MODULE_KEY: &str = \"<entry>\";")
            .count(),
        1
    );
    assert_eq!(LIB_SOURCE.matches("ModuleKey").count(), 1);
}

#[test]
fn module_key_keeps_opaque_storage_and_one_host_constructor() {
    assert!(OWNER_SOURCE.starts_with("/// A stable module identity"));
    assert!(OWNER_SOURCE.contains("[`crate::ModuleRequestIr::specifier`]"));
    assert_eq!(
        OWNER_SOURCE
            .matches("pub struct ModuleKey(String);")
            .count(),
        1
    );
    assert_eq!(OWNER_SOURCE.matches("pub fn from_host(").count(), 1);
    assert_eq!(OWNER_SOURCE.matches("pub fn as_str(").count(), 1);
    assert!(!OWNER_SOURCE.contains("impl From<"));
    assert!(!OWNER_SOURCE.contains("pub fn new("));
    assert!(!OWNER_SOURCE.contains("pub struct ModuleKey(pub String)"));

    let methods = OWNER_SOURCE
        .split_once("impl ModuleKey {")
        .expect("ModuleKey methods")
        .1
        .split_once("/// Key a one-node graph uses")
        .expect("anonymous module key owner")
        .0;
    assert_eq!(
        code_without_whitespace(methods),
        "#[must_use]pubfnfrom_host(key:implInto<String>)->Self{Self(key.into())}\
         #[must_use]pubfnas_str(&self)->&str{&self.0}}"
    );
}

#[test]
fn module_key_callers_keep_the_public_identity_domain_without_compatibility_exports() {
    assert_eq!(OWNER_SOURCE.matches("ModuleKey").count(), 2);
    assert_eq!(LOADED_SOURCES_SOURCE.matches("ModuleKey").count(), 7);
    assert_eq!(GRAPH_SOURCE.matches("ModuleKey").count(), 1);
    assert_eq!(GRAPH_TESTS_SOURCE.matches("ModuleKey").count(), 67);
    assert_eq!(GRAPH_BUILD_SOURCE.matches("ModuleKey").count(), 2);
    assert_eq!(GRAPH_RESOLUTION_SOURCE.matches("ModuleKey").count(), 1);
    assert_eq!(RECORD_SOURCE.matches("ModuleKey").count(), 9);
    assert_eq!(DYNAMIC_SOURCE.matches("ModuleKey").count(), 3);
    assert_eq!(EARLY_SOURCE.matches("ModuleKey").count(), 1);
    assert_eq!(LINK_SOURCE.matches("ModuleKey").count(), 1);
    assert_eq!(LINK_ERROR_SOURCE.matches("ModuleKey").count(), 2);
    assert_eq!(NAMESPACE_SOURCE.matches("ModuleKey").count(), 2);
    assert_eq!(ENGINE_LOADER_SOURCE.matches("ModuleKey").count(), 32);
    assert_eq!(ENGINE_LIB_SOURCE.matches("ModuleKey").count(), 1);
    assert_eq!(
        LOADED_SOURCES_SOURCE
            .matches("ANONYMOUS_MODULE_KEY")
            .count(),
        2
    );
    assert_eq!(RECORD_SOURCE.matches("ANONYMOUS_MODULE_KEY").count(), 2);
    assert_eq!(LINK_SOURCE.matches("ANONYMOUS_MODULE_KEY").count(), 2);
    assert!(!GRAPH_SOURCE.contains("ANONYMOUS_MODULE_KEY"));
    assert!(LOADED_SOURCES_SOURCE.contains("key: ModuleKey,"));
    assert!(GRAPH_SOURCE.contains("pub keys: BTreeMap<ModuleKey, ModuleUnitId>,"));
    assert!(LINK_ERROR_SOURCE.contains(
        "InconsistentLoad {\n        /// The key loaded inconsistently.\n        key: ModuleKey,"
    ));
}
