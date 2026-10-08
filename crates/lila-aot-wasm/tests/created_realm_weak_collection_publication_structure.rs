//! Catalogue callability remains independent of the retired duplicate installer.
const CATALOG_SOURCE: &str = include_str!("../../lila-ir/src/builtins/catalog.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .expect(start)
        .1
        .split_once(end)
        .expect(end)
        .0
}

#[test]
fn weak_collection_constructors_and_methods_retain_their_capability_domain() {
    let catalog = bounded(
        CATALOG_SOURCE,
        "    MapPrototypeSizeGetter {",
        "    WeakRefConstructor {",
    );
    for constructor in ["WeakMapConstructor", "WeakSetConstructor"] {
        let entry = catalog
            .split_once(&format!("    {constructor} {{"))
            .unwrap_or_else(|| panic!("missing catalog constructor `{constructor}`"))
            .1
            .split_once("    }")
            .expect("catalog constructor end")
            .0;
        assert!(entry.contains("flags: [CONSTRUCTABLE, SYNCHRONOUS_USER_CODE],"));
    }
    for method in [
        "WeakMapPrototypeDelete",
        "WeakMapPrototypeGet",
        "WeakMapPrototypeGetOrInsert",
        "WeakMapPrototypeGetOrInsertComputed",
        "WeakMapPrototypeHas",
        "WeakMapPrototypeSet",
        "WeakSetPrototypeAdd",
        "WeakSetPrototypeDelete",
        "WeakSetPrototypeHas",
    ] {
        let entry = catalog
            .split_once(&format!("    {method} {{"))
            .unwrap_or_else(|| panic!("missing catalog method `{method}`"))
            .1
            .split_once("    }")
            .expect("catalog method end")
            .0;
        assert!(
            !entry.contains("CONSTRUCTABLE"),
            "{method} must not construct"
        );
    }
}
