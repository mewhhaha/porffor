const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const HOST_BUILTINS_SOURCE: &str = include_str!("../src/builtins/host.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/shadowed-property-read-arm-removal.md");
const TASK: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");

fn property_read_source() -> &'static str {
    let start = OBJECTS_SOURCE
        .find("    pub(crate) fn compile_property_read_from_locals(")
        .expect("property-read entry");
    let end = OBJECTS_SOURCE[start..]
        .find("    fn compile_dynamic_property_read_from_locals(")
        .map(|offset| start + offset)
        .expect("dynamic property-read entry");
    &OBJECTS_SOURCE[start..end]
}

#[test]
fn property_read_dispatch_has_one_arm_per_value_kind() {
    let source = property_read_source();
    assert_eq!(
        source
            .matches("            ValueKind::Dynamic => {")
            .count(),
        1
    );
    assert_eq!(
        source
            .matches("            ValueKind::String => match key {")
            .count(),
        1
    );
    assert!(!source.contains("            ValueKind::String => {"));
    assert!(source.contains("return self.compile_dynamic_property_read_from_locals("));
}

#[test]
fn private_types_and_imports_have_direct_owners() {
    assert!(!LIB_SOURCE.contains("pub(crate) use functions::RealmRecordLocal;"));
    assert!(!OPERATIONS_SOURCE.contains("NativeErrorKind"));
    assert!(OPERATIONS_SOURCE.contains("use lila_ir::StaticRegExpCompilation;"));
    assert!(FUNCTIONS_SOURCE.contains("pub(crate) struct RealmRecordLocal(u32);"));
    assert!(HOST_BUILTINS_SOURCE.contains("    RealmRecordLocal,"));
}

#[test]
fn removal_has_frozen_source_evidence() {
    for evidence in [CONTRACT, TASK] {
        for hash in [
            "68165b09f3c33dde58a972643a8dd69cf970bca44fff30af6baa600ad1063f76",
            "ed859523f2e4b103fb5b069adf5931321c934efd3ef99f6e6e98b359e63e6c87",
            "763d09a61590ffcf1b4afeac60d93302e8094d3bab928f822518150cd87a02f1",
            "8abe81e3220990ad0a59d373e364761cc6f47981f226475794efe66ddc9a324c",
        ] {
            assert!(evidence.contains(hash));
        }
        assert!(evidence.contains("no new JavaScript behavior"));
    }
}
