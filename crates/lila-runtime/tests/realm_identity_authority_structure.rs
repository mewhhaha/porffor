const RUNTIME_SOURCE: &str = include_str!("../src/lib.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn realm_stores_one_identity_without_persistent_intrinsic_or_global_views() {
    let realm = bounded(
        RUNTIME_SOURCE,
        "pub struct Realm {",
        "impl core::fmt::Debug for Realm",
    );
    assert_eq!(realm.matches("id: RealmId,").count(), 1);
    assert!(!realm.contains("intrinsics:"));
    assert!(!realm.contains("global:"));

    let build = bounded(
        RUNTIME_SOURCE,
        "pub fn build(self) -> Realm {",
        "impl Realm {",
    );
    assert!(!build.contains("intrinsics:"));
    assert!(!build.contains("global:"));
    assert!(!build.contains("RealmIntrinsics::new"));
    assert!(!build.contains("RealmGlobal::new"));
}

#[test]
fn realm_views_are_derived_only_from_the_realm_id() {
    let realm = bounded(RUNTIME_SOURCE, "impl Realm {", "#[cfg(test)]");
    let intrinsics = bounded(
        realm,
        "pub const fn intrinsics(&self) -> RealmIntrinsics {",
        "pub const fn global(&self) -> RealmGlobal {",
    );
    assert!(intrinsics.contains("RealmIntrinsics::new(self.id)"));

    let global = bounded(
        realm,
        "pub const fn global(&self) -> RealmGlobal {",
        "pub fn shell_name(&self)",
    );
    assert!(global.contains("RealmGlobal::new(self.id)"));
}

#[test]
fn global_view_stores_one_id_and_synthesizes_fixed_projections() {
    let global = bounded(
        RUNTIME_SOURCE,
        "pub struct RealmGlobal {",
        "/// A UTC Unix-epoch timestamp",
    );
    assert_eq!(global.matches("realm_id: RealmId,").count(), 1);
    assert!(!global.contains("global_object: RealmObjectId"));
    assert!(!global.contains("global_this: RealmObjectId"));
    assert!(!global.contains("global_environment: GlobalEnvironmentId"));
    assert_eq!(global.matches("realm_id: self.realm_id,").count(), 3);
    assert_eq!(
        global
            .matches("kind: RealmObjectKind::GlobalObject")
            .count(),
        1
    );
    assert_eq!(
        global.matches("kind: RealmObjectKind::GlobalThis").count(),
        1
    );
}
