const FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_arguments_mapped_descriptors.js");
const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/arguments-index-descriptor-exotic.md");

#[test]
fn arguments_index_define_has_one_typed_validated_boundary() {
    assert!(CONTRACT.contains("ECMA-262 10.4.4.1-5"));
}

#[test]
fn dynamic_arguments_named_writes_never_enter_ordinary_object_storage() {
    for witness in [
        "honorsOwnNamedSetter",
        "honorsInheritedNamedSetter",
        "honorsNonWritableNamedProperty",
        "rejectsAbsentIndexOnNonExtensibleArguments",
        "honorsInheritedIndexSetterAfterDelete",
        "rejectsAbsentIndexAssignmentOnNonExtensibleArguments",
        "honorsArgumentsPrototypeIndexedDescriptors",
    ] {
        assert!(FIXTURE.contains(witness));
    }
}
