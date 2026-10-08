use lila_ir::{HostBuiltinExposure, HostBuiltinId, HostBuiltinSurface, HostSurfacePolicy};
use std::collections::BTreeSet;

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
    // The actual catalog macro requires a surface for every row. New
    // capabilities extend that registry, rather than weakening its policy.
    assert!(BUILTINS_SOURCE.contains("surface: $surface:expr,"));
    assert!(CATALOG_SOURCE.contains("host_builtin_catalog! {"));
    assert!(!CATALOG_SOURCE.contains("HostBuiltinRealmScope::"));
    let mut function_ids = BTreeSet::new();
    let mut global_names = BTreeSet::new();
    for builtin in HostBuiltinId::ALL.iter().copied() {
        assert!(function_ids.insert(builtin.function_id()), "{builtin:?}");
        match builtin.surface() {
            HostBuiltinSurface::Global(exposure) => {
                assert_eq!(builtin.global_name(), Some(builtin.as_str()));
                assert!(global_names.insert(builtin.as_str()), "{builtin:?}");
                assert_eq!(
                    HostBuiltinId::from_global_name(builtin.as_str()),
                    Some(builtin)
                );
                let (product, every_realm) = match exposure {
                    HostBuiltinExposure::EcmaGlobal => (true, true),
                    HostBuiltinExposure::ProductExtension => (true, false),
                    HostBuiltinExposure::Test262Capability => (false, false),
                };
                assert_eq!(HostSurfacePolicy::Product.allows(builtin), product);
                assert!(HostSurfacePolicy::Test262.allows(builtin));
                assert_eq!(
                    HostBuiltinId::every_realm_globals().any(|entry| entry == builtin),
                    every_realm,
                );
            }
            HostBuiltinSurface::InternalCallable => {
                assert!(builtin.global_name().is_none());
                assert!(HostBuiltinId::from_global_name(builtin.as_str()).is_none());
                assert!(!HostSurfacePolicy::Product.allows(builtin));
                assert!(!HostSurfacePolicy::Test262.allows(builtin));
            }
        }
    }
    assert_eq!(
        HostBuiltinId::GetAbstractModuleSource.surface(),
        HostBuiltinSurface::Global(HostBuiltinExposure::Test262Capability),
    );
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
