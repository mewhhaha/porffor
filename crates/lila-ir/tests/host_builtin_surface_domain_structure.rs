const BUILTINS_SOURCE: &str = include_str!("../src/builtins.rs");
const CATALOG_SOURCE: &str = include_str!("../src/builtins/host_catalog.rs");
const POLICY_SOURCE: &str = include_str!("../src/builtins/host_surface.rs");
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
fn host_builtin_exposure_exhaustively_owns_realm_scope() {
    let projection = bounded(
        POLICY_SOURCE,
        "    pub(super) const fn realm_scope(self) -> super::HostBuiltinRealmScope {",
        "/// The host-global surface an IR compilation is authorized to expose.",
    );
    assert_eq!(
        code_without_whitespace(projection),
        "matchself{Self::EcmaGlobal=>super::HostBuiltinRealmScope::EveryRealm,\
         Self::ProductExtension|Self::Test262Capability=>{\
         super::HostBuiltinRealmScope::EntryRealmOnly}}}}"
    );
    assert!(!projection.contains("_ =>"));
    assert!(BUILTINS_SOURCE.contains("enum HostBuiltinRealmScope {"));
    assert!(!BUILTINS_SOURCE.contains("pub enum HostBuiltinRealmScope {"));
    assert!(!LIB_SOURCE.contains("HostBuiltinRealmScope"));

    let surface = bounded(
        BUILTINS_SOURCE,
        "pub enum HostBuiltinSurface {",
        "/// Defines host builtin identity",
    );
    assert!(surface.contains("Global(HostBuiltinExposure),"));
    assert!(surface.contains("InternalCallable,"));
    assert!(!surface.contains("realms:"));
    assert!(!surface.contains("HostBuiltinRealmScope"));
}

#[test]
fn host_builtin_catalog_has_one_exposure_choice_per_global_row() {
    assert_eq!(
        CATALOG_SOURCE
            .matches("HostBuiltinSurface::global(HostBuiltinExposure::")
            .count(),
        18
    );
    assert_eq!(
        CATALOG_SOURCE
            .matches("HostBuiltinSurface::InternalCallable")
            .count(),
        1
    );
    assert_eq!(
        CATALOG_SOURCE
            .matches("HostBuiltinExposure::EcmaGlobal")
            .count(),
        2
    );
    assert_eq!(
        CATALOG_SOURCE
            .matches("HostBuiltinExposure::ProductExtension")
            .count(),
        2
    );
    assert_eq!(
        CATALOG_SOURCE
            .matches("HostBuiltinExposure::Test262Capability")
            .count(),
        14
    );
    assert!(!CATALOG_SOURCE.contains("HostBuiltinRealmScope::"));
}

#[test]
fn global_lookup_policy_and_every_realm_iteration_consume_the_closed_surface() {
    let every_realm = bounded(
        BUILTINS_SOURCE,
        "            pub fn every_realm_globals()",
        "            pub fn from_global_name(",
    );
    assert!(every_realm.contains("HostBuiltinSurface::Global(exposure)"));
    assert!(every_realm.contains("exposure.realm_scope() == HostBuiltinRealmScope::EveryRealm"));
    assert!(!every_realm.contains("realms:"));

    let allows = bounded(
        POLICY_SOURCE,
        "    pub const fn allows(self, builtin: HostBuiltinId) -> bool {",
        "    pub fn global_builtins(",
    );
    assert!(allows.contains("let HostBuiltinSurface::Global(exposure)"));
    for exposure in ["EcmaGlobal", "ProductExtension", "Test262Capability"] {
        assert!(allows.contains(&format!("HostBuiltinExposure::{exposure}")));
    }
    assert!(!allows.contains("HostBuiltinRealmScope"));
}
