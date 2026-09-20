const FUNCTIONS: &str = include_str!("../src/functions.rs");
const HOST: &str = include_str!("../src/builtins/host.rs");
const CONSTRUCTOR: &str = include_str!("../src/builtins/string/constructor.rs");
const STANDARD: &str = include_str!("../src/builtins/standard.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .expect(start)
        .1
        .split_once(end)
        .expect(end)
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn every_string_constructor_allocation_carries_its_own_conversion_realm() {
    let allocation = normalized(bounded(
        FUNCTIONS,
        "pub(crate) fn emit_function_value_payload_with_prototype_materialization(",
        "pub(crate) fn emit_function_value_payload_in_realm(",
    ));
    assert!(allocation.contains(concat!(
        "matches!(meta.standard_builtin,Some(StandardBuiltinId::EvalFunction|",
        "StandardBuiltinId::ThrowTypeError|StandardBuiltinId::StringConstructor))"
    )));
    assert_eq!(
        allocation
            .matches(concat!(
                "self.store_i64_local_at_offset(object_local,HEAP_FUNCTION_ENV_HANDLE_OFFSET,",
                "object_local,function,);"
            ))
            .count(),
        1
    );

    let created_realm = normalized(bounded(
        FUNCTIONS,
        "fn emit_function_value_payload_in_realm_with_prototype_materialization(",
        "pub(crate) fn reserve_realm_function_prototype_local(",
    ));
    assert!(created_realm.contains(concat!(
        "self.emit_function_value_payload_with_prototype_materialization(",
        "meta,prototype_materialization,function,)?;"
    )));
    assert!(created_realm.contains(concat!(
        "self.emit_store_function_defining_realm(function_object_local,",
        "context.realm.index(),function,);"
    )));
    assert!(!created_realm.contains("HEAP_FUNCTION_ENV_HANDLE_OFFSET"));
    assert!(normalized(HOST).contains(concat!(
        "self.emit_function_value_payload_in_realm(&string_meta,&realm_functions,",
        "string_constructor_local,function,)?;"
    )));

    let active_identity = bounded(
        STANDARD,
        "pub(crate) enum ActiveStandardBuiltinFunction {",
        "enum ArrayBufferSliceKind {",
    );
    assert!(!active_identity.contains("StringConstructor"));
}

#[test]
fn string_call_keeps_undefined_new_target_and_constructor_owns_conversion_before_prototype() {
    let constructor = normalized(CONSTRUCTOR);
    let primitive = constructor
        .find("self.emit_tagged_to_primitive_locals_in_current_function_realm(")
        .unwrap();
    let string = constructor
        .find("self.emit_current_function_realm_primitive_to_string_local(")
        .unwrap();
    let prototype = constructor
        .find("self.emit_new_target_prototype_to_locals(")
        .unwrap();
    let allocation = constructor
        .find("self.emit_alloc_plain_object_with_prototype_and_tag(")
        .unwrap();
    assert!(primitive < string && string < prototype && prototype < allocation);
    assert!(!constructor.contains("emit_normalize_undefined_new_target"));
    assert!(constructor.contains(concat!(
        "NewTargetPrototypeFallback::RequiredResolvedRealmOrdinary(",
        "OrdinaryDefaultPrototype::String,)"
    )));
    let construct = bounded(
        FUNCTIONS,
        "pub(crate) fn emit_function_handle_construct_with_argv(",
        "pub(crate) fn copy_function_realm_typed_array_prototypes(",
    );
    let construct_normalized = normalized(construct);
    let direct_return = bounded(
        &construct_normalized,
        "letdirect_returning_constructor_table_indices:Vec<i64>=[",
        "].into_iter()",
    );
    assert_eq!(
        direct_return
            .matches("StandardBuiltinId::StringConstructor,")
            .count(),
        1
    );
    assert!(!construct.contains("string_constructor_table_index"));
    assert!(!construct.contains("BOXED_PRIMITIVE_KIND_STRING"));
}
