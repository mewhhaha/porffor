const PARENT_SOURCE: &str = include_str!("../src/builtins/disposable_stack.rs");
const CAPABILITY_TRANSFER_SOURCE: &str =
    include_str!("../src/builtins/disposable_stack/capability_transfer.rs");
const BODY_SOURCE: &str = concat!(
    include_str!("../src/builtins/disposable_stack.rs"),
    include_str!("../src/builtins/disposable_stack/capability_transfer.rs")
);
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const INSTALLER_SOURCE: &str = include_str!("../src/intrinsics/resource_management.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CATALOG_SOURCE: &str = include_str!("../../lila-ir/src/builtins/catalog.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_disposable_stack_lifecycle.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/disposable-stack-synchronous-lifecycle.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier = source.find(earlier).expect("earlier operation");
    let later = source.find(later).expect("later operation");
    assert!(earlier < later);
}

#[test]
fn disposable_stack_surface_is_catalogued_dispatched_and_rooted_as_one_unit() {
    let catalog = bounded(
        CATALOG_SOURCE,
        "    DisposableStackConstructor {",
        "\n}\n\nimpl StandardBuiltinId {",
    );
    let dispatch = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::DisposableStackConstructor => {",
        "            StandardBuiltinId::AsyncDisposableStackPrototypeUse => {",
    );
    let dependencies = bounded(
        PLANNING_SOURCE,
        "        if builtin == StandardBuiltinId::DisposableStackConstructor {",
        "        if builtin == StandardBuiltinId::DisposableStackPrototypeDispose {",
    );

    for (builtin, emitter, contract_row) in [
        ("Use", "emit_disposable_stack_use(function)?", "| `use` |"),
        (
            "Adopt",
            "emit_disposable_stack_adopt(function)?",
            "| `adopt` |",
        ),
        (
            "Defer",
            "emit_disposable_stack_defer(function)?",
            "| `defer` |",
        ),
        (
            "Move",
            "emit_disposable_stack_move(function)?",
            "| `move` |",
        ),
        (
            "Dispose",
            "emit_disposable_stack_dispose(function)?",
            "| `dispose` |",
        ),
        (
            "DisposedGetter",
            "emit_disposable_stack_disposed_getter(function)?",
            "| `disposed getter` |",
        ),
    ] {
        let id = format!("DisposableStackPrototype{builtin}");
        assert_eq!(catalog.matches(&format!("    {id} {{")).count(), 1);
        assert_eq!(
            dispatch
                .matches(&format!("StandardBuiltinId::{id} =>"))
                .count(),
            1
        );
        assert_eq!(dispatch.matches(emitter).count(), 1);
        assert_eq!(
            dependencies
                .matches(&format!("StandardBuiltinId::{id},"))
                .count(),
            1
        );
        assert!(CONTRACT.contains(contract_row));
    }

    let zero_length = bounded(
        PLANNING_SOURCE,
        "        // Pinned by the two DisposableStack families' `length.js` files",
        "        StandardBuiltinId::AsyncDisposableStackPrototypeUse",
    );
    for builtin in ["Move", "Dispose", "DisposedGetter"] {
        assert!(zero_length.contains(&format!(
            "StandardBuiltinId::DisposableStackPrototype{builtin}"
        )));
    }
    let one_length = bounded(
        PLANNING_SOURCE,
        "        StandardBuiltinId::AsyncDisposableStackPrototypeUse",
        "        | StandardBuiltinId::AsyncDisposableStackDisposeAsyncRejected => 1,",
    );
    for builtin in ["Use", "Defer"] {
        assert!(one_length.contains(&format!(
            "StandardBuiltinId::DisposableStackPrototype{builtin}"
        )));
    }
    let two_length = bounded(
        PLANNING_SOURCE,
        "        StandardBuiltinId::AsyncDisposableStackPrototypeAdopt",
        " => 2,\n    }\n}",
    );
    assert!(two_length.contains("StandardBuiltinId::DisposableStackPrototypeAdopt"));

    let dispose_dependencies = bounded(
        PLANNING_SOURCE,
        "        if builtin == StandardBuiltinId::DisposableStackPrototypeDispose {",
        "        if builtin == StandardBuiltinId::ArrayFromAsync {",
    );
    assert_eq!(
        dispose_dependencies
            .matches("StandardBuiltinId::SuppressedErrorConstructor")
            .count(),
        1
    );
}

#[test]
fn disposable_stack_heap_domains_are_closed_and_call_shape_is_exhaustive() {
    let state = bounded(
        HEAP_SOURCE,
        "pub(crate) enum DisposableStackState {",
        "/// The complete synchronous resource-entry domain.",
    );
    assert_eq!(state.matches("Pending").count(), 2);
    assert_eq!(state.matches("Disposed").count(), 2);
    assert!(!state.contains("_ =>"));

    let entry_domain = bounded(
        HEAP_SOURCE,
        "pub(crate) enum DisposableStackEntryKind {",
        "/// `[[AsyncDisposableState]]` is a two-element domain",
    );
    for kind in ["Use", "Adopt", "Defer"] {
        assert!(entry_domain.contains(&format!("Self::{kind}")));
    }
    assert!(entry_domain.contains("pub(crate) const ALL: [Self; 3]"));
    assert!(entry_domain.contains("Self::Use => DisposableStackDisposeCall::ResourceReceiver"));
    assert!(entry_domain.contains(
        "Self::Adopt => DisposableStackDisposeCall::UndefinedReceiverWithResourceArgument"
    ));
    assert!(entry_domain
        .contains("Self::Defer => DisposableStackDisposeCall::UndefinedReceiverNoArguments"));
    assert!(!entry_domain.contains("_ =>"));
}

#[test]
fn registration_validates_before_the_only_entry_publication_path() {
    let use_body = bounded(
        BODY_SOURCE,
        "    pub(crate) fn emit_disposable_stack_use(",
        "    pub(crate) fn emit_disposable_stack_adopt(",
    );
    let adopt_body = bounded(
        BODY_SOURCE,
        "    pub(crate) fn emit_disposable_stack_adopt(",
        "    pub(crate) fn emit_disposable_stack_defer(",
    );
    let defer_body = bounded(
        BODY_SOURCE,
        "    pub(crate) fn emit_disposable_stack_defer(",
        "    pub(crate) fn emit_disposable_stack_move(",
    );

    for body in [use_body, adopt_body, defer_body] {
        assert_before(
            body,
            "emit_disposable_stack_record_from_receiver(",
            "emit_disposable_stack_require_pending(",
        );
        assert_before(
            body,
            "emit_disposable_stack_require_pending(",
            "emit_builtin_arg_to_locals(",
        );
        assert_eq!(body.matches("emit_disposable_stack_push_entry(").count(), 1);
    }
    assert_before(
        use_body,
        "DisposableStackTypeError::UseValueNotDisposable",
        "emit_disposable_stack_push_entry(",
    );
    assert_before(
        adopt_body,
        "DisposableStackTypeError::AdoptCallbackNotCallable",
        "emit_disposable_stack_push_entry(",
    );
    assert_before(
        defer_body,
        "DisposableStackTypeError::DeferCallbackNotCallable",
        "emit_disposable_stack_push_entry(",
    );

    let get_method = bounded(
        BODY_SOURCE,
        "    fn emit_disposable_stack_get_method(",
        "    fn emit_disposable_stack_record_from_receiver(",
    );
    assert_before(
        get_method,
        "emit_is_callable_i32(method_tag_local, method_payload_local, function)?",
        "DisposableStackTypeError::DisposeMethodNotCallable",
    );

    let push = bounded(
        BODY_SOURCE,
        "    fn emit_disposable_stack_push_entry(",
        "    fn emit_disposable_stack_get_method(",
    );
    let new_entry = bounded(
        push,
        "        function.instruction(&Instruction::LocalSet(entry_local));",
        "        self.release_temp_local(entry_local);",
    );
    let kind = new_entry
        .find("HEAP_DISPOSABLE_STACK_ENTRY_KIND_OFFSET")
        .expect("entry kind initialization");
    let method = new_entry
        .find("HEAP_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET")
        .expect("entry method initialization");
    let length = new_entry
        .rfind("HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET")
        .expect("entry length publication");
    assert!(kind < method && method < length);
    assert_eq!(
        new_entry
            .matches("HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET")
            .count(),
        1,
        "a fully initialized entry is published by one final length store"
    );
}

#[test]
fn move_is_a_single_consuming_capability_transfer() {
    let transfer_type = bounded(
        CAPABILITY_TRANSFER_SOURCE,
        "#[must_use = \"a transferred DisposableStack capability must be installed exactly once\"]",
        "impl<'a> FunctionBuilder<'a> {",
    );
    assert!(transfer_type.contains("pub(super) struct TransferredDisposableStackCapabilityLocals"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(!transfer_type.contains(capability));
    }
    assert!(!CAPABILITY_TRANSFER_SOURCE.lines().any(|line| {
        line.trim_start().starts_with("impl ")
            && line.contains(" for TransferredDisposableStackCapabilityLocals")
    }));
    assert!(!PARENT_SOURCE.contains("TransferredDisposableStackCapabilityLocals"));
    assert!(!PARENT_SOURCE.contains("capability_transfer::"));
    assert_eq!(
        BODY_SOURCE
            .matches("TransferredDisposableStackCapabilityLocals")
            .count(),
        4,
        "the carrier must have one declaration, return, construction and consuming parameter"
    );
    for field in ["entries_ptr", "entries_len", "entries_cap"] {
        assert_eq!(
            CAPABILITY_TRANSFER_SOURCE.matches(field).count(),
            6,
            "each raw transfer field must remain child-owned from capture through release"
        );
    }
    for operation in [
        "emit_take_disposable_stack_capability(",
        "emit_install_transferred_disposable_stack_capability(",
    ] {
        assert_eq!(
            BODY_SOURCE.matches(operation).count(),
            2,
            "each transfer operation must have one child definition and one parent call"
        );
    }

    let move_body = bounded(
        BODY_SOURCE,
        "    pub(crate) fn emit_disposable_stack_move(",
        "    pub(crate) fn emit_disposable_stack_dispose(",
    );
    for (earlier, later) in [
        (
            "emit_disposable_stack_record_from_receiver(",
            "emit_disposable_stack_require_pending(",
        ),
        (
            "emit_disposable_stack_require_pending(",
            "emit_take_disposable_stack_capability(",
        ),
        (
            "emit_take_disposable_stack_capability(",
            "emit_install_transferred_disposable_stack_capability(",
        ),
        (
            "emit_install_transferred_disposable_stack_capability(",
            "emit_finalize_disposable_stack_instance(",
        ),
    ] {
        assert_before(move_body, earlier, later);
    }

    let take = bounded(
        CAPABILITY_TRANSFER_SOURCE,
        "    pub(super) fn emit_take_disposable_stack_capability(",
        "    pub(super) fn emit_install_transferred_disposable_stack_capability(",
    );
    let first_mutation = take
        .find("self.store_i64_const_at_offset(source_record_local, offset, 0, function)")
        .expect("source capability clear");
    for offset in [
        "HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET",
        "HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET",
        "HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET",
    ] {
        assert!(take.find(offset).expect("capability snapshot field") < first_mutation);
    }
    assert_before(
        take,
        "self.store_i64_const_at_offset(source_record_local, offset, 0, function)",
        "DisposableStackState::Disposed.word()",
    );

    let install = bounded(
        CAPABILITY_TRANSFER_SOURCE,
        "    pub(super) fn emit_install_transferred_disposable_stack_capability(",
        "\n    }\n}",
    );
    assert!(install.contains("transfer: TransferredDisposableStackCapabilityLocals"));
    assert!(install.contains(") -> PendingDisposableStackRecordLocal"));
}

#[test]
fn disposal_flips_state_before_the_exhaustive_lifo_callback_walk() {
    let disposal_type = bounded(
        BODY_SOURCE,
        "#[must_use = \"an active DisposableStack disposal must be consumed by its LIFO walker\"]",
        "impl<'a> FunctionBuilder<'a> {",
    );
    assert!(disposal_type.contains("struct DisposableStackDisposalLocals"));
    assert!(!disposal_type.contains("derive(Clone"));
    assert!(!disposal_type.contains("derive(Copy"));

    let dispose = bounded(
        BODY_SOURCE,
        "    pub(crate) fn emit_disposable_stack_dispose(",
        "    pub(crate) fn emit_disposable_stack_disposed_getter(",
    );
    assert_before(
        dispose,
        "emit_begin_disposable_stack_disposal(",
        "emit_consume_disposable_stack_disposal(",
    );

    let begin = bounded(
        BODY_SOURCE,
        "    fn emit_begin_disposable_stack_disposal(",
        "    fn emit_consume_disposable_stack_disposal(",
    );
    assert_before(
        begin,
        "DisposableStackState::Disposed.word()",
        "HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET",
    );
    assert_before(
        begin,
        "DisposableStackState::Disposed.word()",
        "HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET",
    );

    let walk = bounded(
        BODY_SOURCE,
        "    fn emit_consume_disposable_stack_disposal(",
        "    fn emit_disposable_stack_record_error(",
    );
    assert!(walk.contains("DisposableStackEntryKind::ALL"));
    assert!(walk.contains(".map(|kind| (kind, kind.dispose_call()))"));
    assert!(walk.contains(
        "LocalGet(disposal.next_index));\n        function.instruction(&Instruction::I64Const(1));\n        function.instruction(&Instruction::I64Sub);\n        function.instruction(&Instruction::LocalSet(disposal.next_index));"
    ));
    for call in [
        "DisposableStackDisposeCall::ResourceReceiver",
        "DisposableStackDisposeCall::UndefinedReceiverWithResourceArgument",
        "DisposableStackDisposeCall::UndefinedReceiverNoArguments",
    ] {
        assert_eq!(walk.matches(call).count(), 1);
    }
    assert!(!walk.contains("_ =>"));
    assert_eq!(
        walk.matches("emit_function_or_proxy_call_leave_throw_completion(")
            .count(),
        1,
        "every closed call shape reaches the one callback-emission path"
    );
    assert_before(
        walk,
        "emit_disposable_stack_record_error(",
        "function.instruction(&Instruction::Br(0));",
    );
}

#[test]
fn disposal_error_fold_preserves_first_identity_then_nests_new_over_previous() {
    let fold = bounded(
        BODY_SOURCE,
        "    fn emit_disposable_stack_record_error(",
        "    #[allow(clippy::too_many_arguments)]",
    );
    assert_before(
        fold,
        "LocalGet(new_error_payload_local)",
        "LocalGet(disposal.has_error)",
    );
    assert!(fold.contains(
        "self.emit_alloc_suppressed_error_instance_from_locals(\n            None,\n            new_error_payload_local,\n            new_error_tag_local,\n            disposal.error_payload,\n            disposal.error_tag,"
    ));
    assert_before(
        fold,
        "emit_alloc_suppressed_error_instance_from_locals(",
        "LocalSet(disposal.error_payload)",
    );
    assert_before(
        fold,
        "LocalSet(disposal.error_payload)",
        "LocalSet(disposal.has_error)",
    );
}

#[test]
fn dispose_and_symbol_dispose_share_exactly_one_function_value_allocation() {
    let installer = bounded(
        INSTALLER_SOURCE,
        "    pub(crate) fn install_disposable_stack_constructor_intrinsics(",
        "\n    }\n}",
    );
    assert_eq!(installer.matches("let dispose_meta = self").count(), 1);
    assert_eq!(
        installer
            .matches("emit_object_define_function_data_with_aliases(")
            .count(),
        1
    );
    assert_eq!(installer.matches("&[\"Symbol.dispose\"]").count(), 1);
    assert_eq!(
        installer
            .matches("StandardBuiltinId::DisposableStackPrototypeDispose.function_id()")
            .count(),
        1
    );

    let alias_helper = bounded(
        OBJECTS_SOURCE,
        "    pub(crate) fn emit_object_define_function_data_with_aliases(",
        "    pub(crate) fn emit_object_define_function_global_data(",
    );
    assert_eq!(
        alias_helper
            .matches("self.emit_function_value_payload(meta, function)?")
            .count(),
        1
    );
    assert!(alias_helper.contains("std::iter::once(key).chain(aliases.iter().copied())"));
    assert_before(
        alias_helper,
        "self.emit_function_value_payload(meta, function)?",
        "for property in std::iter::once(key).chain(aliases.iter().copied())",
    );
}

#[test]
fn lifecycle_consumer_oracle_pins_the_observable_contract() {
    for witness in [
        "Symbol.dispose identity",
        "use method acquired once",
        "mixed-kind LIFO",
        "state changed after callback",
        "disposed use observed getter",
        "move invoked callbacks",
        "move base prototype",
        "single error identity",
        "outer new error",
        "inner new error",
        "oldest suppressed error",
    ] {
        assert!(
            FIXTURE.contains(witness),
            "missing fixture witness {witness}"
        );
    }
    assert!(CONTRACT.contains("exactly 76 deferred"));
    assert!(CONTRACT.contains("this is 92 of"));
    assert!(CONTRACT.contains("the 93 files under `built-ins/DisposableStack`"));

    for error in [
        "DisposableStack method receiver is not an object",
        "DisposableStack method receiver does not have [[DisposableState]]",
        "DisposableStack is already disposed",
        "DisposableStack.prototype.use value is not an object",
        "DisposableStack.prototype.use value is not disposable",
        "DisposableStack.prototype.use dispose method is not callable",
        "DisposableStack.prototype.adopt onDispose is not callable",
        "DisposableStack.prototype.defer onDispose is not callable",
    ] {
        assert!(
            DATA_SOURCE.contains(error),
            "unpooled lifecycle error: {error}"
        );
        assert!(
            BODY_SOURCE.contains(error),
            "unused lifecycle error: {error}"
        );
    }

    assert!(BODY_SOURCE.contains("emit_disposable_stack_constructor"));
}
