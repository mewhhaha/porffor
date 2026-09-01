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
fn realm_id_cannot_represent_the_absent_zero_identity() {
    let realm_id = bounded(
        RUNTIME_SOURCE,
        "pub struct RealmId(",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum IntrinsicRole",
    );

    assert!(realm_id.starts_with("NonZeroU64);"));
    assert!(realm_id.contains("self.0.get()"));
    assert!(!realm_id.contains("pub const fn new"));
}

#[test]
fn realm_builder_refuses_to_wrap_the_identity_allocator() {
    let build = bounded(
        RUNTIME_SOURCE,
        "pub fn build(self) -> Realm {",
        "impl Realm {",
    );

    assert!(build.contains(".fetch_update(Ordering::Relaxed, Ordering::Relaxed"));
    assert!(build.contains("next.checked_add(1)"));
    assert!(build.contains("realm ID space exhausted at {exhausted}"));
    assert!(build.contains("RealmId(NonZeroU64::new(raw_id)"));
    assert!(!build.contains("fetch_add"));
    assert_eq!(RUNTIME_SOURCE.matches("RealmId(").count(), 2);
    assert_eq!(build.matches("RealmId(").count(), 1);
}
