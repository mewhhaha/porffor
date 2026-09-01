const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE: &str =
    include_str!("../src/control_flow/async_function_for_of_iterator.rs");

const FOR_OF_PROTOCOL_ERRORS: [&str; 5] = [
    "NotIterable",
    "NotIterable",
    "MethodResultNotObject",
    "NextNotCallable",
    "NextResultNotObject",
];

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

fn protocol_error_variants(source: &str) -> Vec<&str> {
    const PREFIX: &str = "SyncIteratorProtocolError::";

    source
        .match_indices(PREFIX)
        .map(|(offset, _)| {
            let variant = &source[offset + PREFIX.len()..];
            let end = variant
                .find(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                .unwrap_or(variant.len());
            &variant[..end]
        })
        .collect()
}

fn assert_direct_for_of_owner(source: &str, owner: &str) {
    assert_eq!(
        source
            .matches("let consumer = SyncIteratorConsumer::ForOf;")
            .count(),
        1,
        "{owner} must own one for-of consumer"
    );
    assert_eq!(
        source
            .matches("self.emit_sync_iterator_protocol_type_error(")
            .count(),
        5,
        "{owner} protocol producer count"
    );
    assert_eq!(
        source.matches("&consumer,").count(),
        5,
        "{owner} must project every protocol failure through its consumer"
    );
    assert_eq!(
        protocol_error_variants(source),
        FOR_OF_PROTOCOL_ERRORS,
        "{owner} protocol failure order"
    );
    assert_eq!(
        source.matches("emit_throw_runtime_error(").count(),
        0,
        "{owner} must not emit a raw main-Realm TypeError"
    );
    assert_eq!(
        source
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        0,
        "{owner} must not bypass the shared protocol projector"
    );
}

#[test]
fn direct_for_of_owners_project_five_typed_protocol_errors() {
    let async_disposable = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn compile_async_disposable_for_of_iterator(",
        "    pub(crate) fn compile_for_of_iterator(",
    );
    let ordinary = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn compile_for_of_iterator(",
        "    pub(crate) fn compile_object_destructure_to_locals(",
    );

    assert_direct_for_of_owner(async_disposable, "compile_async_disposable_for_of_iterator");
    assert_direct_for_of_owner(ordinary, "compile_for_of_iterator");
}

#[test]
fn ordinary_direct_for_of_uses_proxy_aware_callability_and_dispatch() {
    let owner = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn compile_for_of_iterator(",
        "    pub(crate) fn compile_object_destructure_to_locals(",
    );

    assert_eq!(owner.matches("self.emit_is_callable_i32(").count(), 2);
    assert_eq!(
        owner
            .matches("self.emit_function_or_proxy_call_leave_throw_completion(")
            .count(),
        2
    );
    assert_eq!(owner.matches("self.emit_function_handle_call(").count(), 0);
    assert_eq!(owner.matches("ValueKind::Function.tag()").count(), 0);

    let iterator_method = bounded(
        owner,
        "        self.emit_propagate_throw_from_locals_if_needed(\n            method_payload_local,",
        "        function.instruction(&Instruction::I64Const(self.strings.payload(\"next\")));",
    );
    let method_callability = iterator_method
        .find("self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;")
        .expect("iterator method IsCallable check");
    let method_protocol_error = iterator_method
        .find("SyncIteratorProtocolError::NotIterable")
        .expect("iterator method protocol error");
    let method_call = iterator_method
        .find("self.emit_function_or_proxy_call_leave_throw_completion(")
        .expect("iterator method Proxy-aware Call");
    let method_throw = iterator_method
        .find(
            "self.emit_propagate_throw_from_locals_if_needed(\n            iterator_payload_local,",
        )
        .expect("iterator method throw propagation");
    let method_result = iterator_method
        .find("self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);")
        .expect("iterator method object-result check");
    assert!(method_callability < method_protocol_error);
    assert!(method_protocol_error < method_call);
    assert!(method_call < method_throw);
    assert!(method_throw < method_result);
    assert!(iterator_method.contains(
        "method_payload_local,\n            method_tag_local,\n            iterable_payload_local,\n            iterable_tag_local,"
    ));

    let next_method = bounded(
        owner,
        "        self.emit_object_read(\n            iterator_payload_local,",
        "        function.instruction(&Instruction::I64Const(self.strings.payload(\"done\")));",
    );
    let next_callability = next_method
        .find("self.emit_is_callable_i32(next_tag_local, next_payload_local, function)?;")
        .expect("next method IsCallable check");
    let next_protocol_error = next_method
        .find("SyncIteratorProtocolError::NextNotCallable")
        .expect("next method protocol error");
    let next_call = next_method
        .find("self.emit_function_or_proxy_call_leave_throw_completion(")
        .expect("next method Proxy-aware Call");
    let next_throw = next_method[next_call..]
        .find("self.emit_propagate_current_completion_if_throw(function);")
        .map(|offset| next_call + offset)
        .expect("next method call throw propagation");
    let next_result = next_method
        .find("self.emit_is_heap_object_like_tag_i32(result_tag_local, function);")
        .expect("next method object-result check");
    assert!(next_callability < next_protocol_error);
    assert!(next_protocol_error < next_call);
    assert!(next_call < next_throw);
    assert!(next_throw < next_result);
    assert!(next_method.contains(
        "next_payload_local,\n            next_tag_local,\n            iterator_payload_local,\n            iterator_tag_local,"
    ));
}

#[test]
fn async_disposable_for_of_boxes_primitives_in_the_current_function_realm() {
    let owner = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn compile_async_disposable_for_of_iterator(",
        "    pub(crate) fn compile_for_of_iterator(",
    );

    assert_eq!(
        owner
            .matches("self.emit_value_to_current_function_realm_object_locals(")
            .count(),
        1
    );
    assert_eq!(owner.matches("emit_value_to_object_locals(").count(), 0);
}

#[test]
fn resumable_sync_for_of_delegates_five_typed_protocol_checks() {
    let owner = bounded(
        ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE,
        "    pub(crate) fn compile_async_function_for_of_iterator(",
        "\n    }\n}",
    );
    assert_eq!(
        owner
            .matches("self.emit_get_iterator_from_value_locals(")
            .count(),
        1
    );
    assert_eq!(
        owner.matches("self.emit_sync_iterator_step_value(").count(),
        1
    );
    assert_eq!(
        owner
            .matches("let consumer = SyncIteratorConsumer::ForOf;")
            .count(),
        1
    );
    assert_eq!(owner.matches("&consumer,").count(), 2);
    assert_eq!(
        owner
            .matches("self.emit_sync_iterator_protocol_type_error(")
            .count(),
        0
    );

    let acquisition_delegation = bounded(
        owner,
        "        self.emit_get_iterator_from_value_locals(",
        "        self.write_binding_from_locals(",
    );
    let step_delegation = bounded(
        owner,
        "        self.emit_sync_iterator_step_value(",
        "        function.instruction(&Instruction::LocalGet(done_local));",
    );
    for delegation in [acquisition_delegation, step_delegation] {
        assert_eq!(delegation.matches("&consumer,").count(), 1);
    }

    let acquisition = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_get_iterator_from_value_locals(",
        "    fn finish_get_iterator_from_method(",
    );
    let acquisition_finish = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn finish_get_iterator_from_method(",
        "    fn emit_sync_iterator_protocol_type_error(",
    );
    let step = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_sync_iterator_step_value(",
        "    fn prepare_destructuring_target<'b>(",
    );
    let shared_owners = [acquisition, acquisition_finish, step];
    assert_eq!(
        shared_owners
            .iter()
            .map(|source| {
                source
                    .matches("self.emit_sync_iterator_protocol_type_error(")
                    .count()
            })
            .sum::<usize>(),
        5
    );
    assert_eq!(
        shared_owners
            .iter()
            .flat_map(|source| protocol_error_variants(source))
            .collect::<Vec<_>>(),
        FOR_OF_PROTOCOL_ERRORS
    );
}

#[test]
fn for_of_protocol_errors_project_from_the_closed_body_realm_source() {
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("self.emit_sync_iterator_protocol_type_error(")
            .count(),
        17
    );

    let projector = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_sync_iterator_protocol_type_error(",
        "    fn compile_array_destructuring_element(",
    );
    assert_eq!(projector.matches("SyncIteratorConsumer::ForOf").count(), 4);
    assert_eq!(
        projector
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );
    assert_eq!(projector.matches("emit_throw_runtime_error(").count(), 1);
    for source in [
        "NumericErrorRealmSource::StandardBuiltinEnvironment",
        "NumericErrorRealmSource::GlobalFallback",
        "NumericErrorRealmSource::NumericConversionHelperArgument",
    ] {
        assert_eq!(
            projector.matches(source).count(),
            1,
            "Realm source {source}"
        );
    }
    assert_eq!(
        projector
            .matches("match self.numeric_error_realm_source()")
            .count(),
        1
    );
    assert!(!projector.contains("_ =>"));
}
