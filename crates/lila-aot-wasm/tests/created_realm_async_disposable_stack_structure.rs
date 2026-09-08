const HEAP: &str = include_str!("../src/heap.rs");
const BOOTSTRAP: &str = include_str!("../src/builtins/bootstrap.rs");
const HOST: &str = include_str!("../src/builtins/host.rs");
const PUBLICATION: &str =
    include_str!("../src/builtins/host/created_realm_async_disposable_stack_intrinsics.rs");
const ALLOCATION: &str =
    include_str!("../src/functions/current_function_realm_async_disposable_stack.rs");
const STACK: &str = include_str!("../src/builtins/async_disposable_stack.rs");

#[test]
fn canonical_stack_prototype_is_traced_and_published_in_each_realm() {
    let slot = HEAP
        .split_once("name: \"%AsyncDisposableStack.prototype%\",")
        .expect("canonical stack prototype slot")
        .1
        .split_once("},")
        .expect("complete heap slot")
        .0;
    for marker in [
        "offset: HEAP_REALM_INTRINSICS_ASYNC_DISPOSABLE_STACK_PROTOTYPE_OFFSET",
        "width: 8",
        "pointer: true",
    ] {
        assert!(slot.contains(marker), "missing slot invariant: {marker}");
    }
    for source in [BOOTSTRAP, PUBLICATION] {
        assert!(source.contains("NonArrayRealmIntrinsicSlot::AsyncDisposableStackPrototype"));
    }
    assert!(PUBLICATION.contains("RealmFunctionMaterializationContext"));
    assert!(PUBLICATION.contains("#[must_use ="));
    assert!(PUBLICATION.contains("release_temp_local(constructor_local)"));
    assert!(PUBLICATION.contains("release_temp_local(prototype_local)"));
    assert!(!PUBLICATION.contains("Instruction::GlobalSet"));

    let stack_allocation = HOST
        .find("emit_materialize_created_realm_async_disposable_stack_intrinsics(")
        .expect("stack allocation");
    let following_allocation = HOST
        .find("emit_materialize_created_realm_finalization_registry_intrinsics(")
        .expect("following allocation");
    let following_publication = HOST
        .find("emit_publish_created_realm_finalization_registry_intrinsics(")
        .expect("following allocation released first");
    let stack_publication = HOST
        .find("emit_publish_created_realm_async_disposable_stack_intrinsics(")
        .expect("stack publication consumes its retained locals");
    assert!(stack_allocation < following_allocation);
    assert!(following_publication < stack_publication);
}

#[test]
fn constructor_and_move_use_canonical_realm_authority() {
    let constructor = STACK
        .split_once("pub(crate) fn emit_async_disposable_stack_constructor(")
        .expect("constructor")
        .1
        .split_once("pub(crate) fn emit_async_disposable_stack_use(")
        .expect("constructor end")
        .0;
    assert!(constructor.contains("NewTargetPrototypeFallback::RealmIntrinsic("));
    assert!(constructor.contains("HEAP_REALM_INTRINSICS_ASYNC_DISPOSABLE_STACK_PROTOTYPE_OFFSET"));
    assert!(!constructor.contains("NewTargetPrototypeFallback::CurrentGlobal"));

    for marker in [
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
        "HEAP_REALM_INTRINSICS_OFFSET",
        "NonArrayRealmIntrinsicSlot::AsyncDisposableStackPrototype.offset()",
        "Instruction::Unreachable",
        "emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)",
    ] {
        assert!(
            ALLOCATION.contains(marker),
            "missing allocator invariant: {marker}"
        );
    }
    assert!(!ALLOCATION.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(!ALLOCATION.contains("emit_object_get"));
    assert!(
        STACK.contains("emit_alloc_current_function_realm_async_disposable_stack_object(function)")
    );
}
