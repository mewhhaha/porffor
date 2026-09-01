const COLLECTIONS_SOURCE: &str = include_str!("../src/builtins/collections.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier operation `{earlier}`"));
    let later_offset = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later operation `{later}`"));
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn set_collection_kind_is_a_closed_domain_without_equality_policy() {
    let declaration_region = bounded(
        COLLECTIONS_SOURCE,
        "#[derive(Clone, Copy)]\nenum SetCollectionKind {",
        "#[derive(Clone, Copy, PartialEq, Eq)]\nenum MapConstructorTypeError",
    )
    .chars()
    .filter(|character| !character.is_whitespace())
    .collect::<String>();
    assert_eq!(declaration_region, "Set,WeakSet,}");
    let declaration = bounded(
        COLLECTIONS_SOURCE,
        "enum SetCollectionKind {",
        "\n}\n\n#[derive(Clone, Copy, PartialEq, Eq)]\nenum MapConstructorTypeError",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take_while(|line| *line != "}")
        .collect::<Vec<_>>();

    assert_eq!(variants, ["Set,", "WeakSet,"]);
    assert!(!declaration.contains("pub"));
    assert!(!declaration.contains("Default"));
    assert!(!COLLECTIONS_SOURCE.contains("collection_kind == SetCollectionKind"));
    for implementation in [
        "PartialEq for SetCollectionKind",
        "Eq for SetCollectionKind",
        "PartialEq for CollectionAlgorithmTypeError",
        "Eq for CollectionAlgorithmTypeError",
    ] {
        assert!(!COLLECTIONS_SOURCE.contains(implementation));
    }
    let algorithm_declaration_region = bounded(
        COLLECTIONS_SOURCE,
        "#[derive(Clone, Copy, PartialEq, Eq)]\nenum SetConstructorTypeError {",
        "impl CollectionAlgorithmTypeError {",
    )
    .chars()
    .filter(|character| !character.is_whitespace())
    .collect::<String>();
    assert_eq!(
        algorithm_declaration_region,
        concat!(
            "RequiresNew,AdderNotCallable,IteratorMethodNotCallable,",
            "IteratorMethodResultNotObject,IteratorNextNotCallable,",
            "IteratorNextResultNotObject,}",
            "#[derive(Clone,Copy)]enumCollectionAlgorithmTypeError{",
            "MapConstructor(MapCollectionKind,MapConstructorTypeError),",
            "SetConstructor(SetCollectionKind,SetConstructorTypeError),",
            "ForEachCallback(StrongCollectionCursor),}"
        )
    );
}

#[test]
fn named_entry_points_are_the_only_set_collection_kind_producers() {
    for call in [
        "emit_set_collection_constructor(SetCollectionKind::Set, function)",
        "emit_set_collection_constructor(SetCollectionKind::WeakSet, function)",
        "emit_set_collection_prototype_add(SetCollectionKind::Set, function)",
        "emit_set_collection_prototype_add(SetCollectionKind::WeakSet, function)",
        "emit_set_collection_prototype_delete(SetCollectionKind::Set, function)",
        "emit_set_collection_prototype_delete(SetCollectionKind::WeakSet, function)",
        "emit_set_collection_prototype_has(SetCollectionKind::Set, function)",
        "emit_set_collection_prototype_has(SetCollectionKind::WeakSet, function)",
        concat!(
            "emit_set_collection_record_from_receiver(\n",
            "            SetCollectionKind::Set,\n",
            "            set_record_local,\n",
            "            function,\n",
            "        )"
        ),
        concat!(
            "emit_find_set_collection_entry(\n",
            "            SetCollectionKind::Set,\n",
            "            set_record_local,\n",
            "            value_payload_local,\n",
            "            value_tag_local,\n",
            "            found_entry_local,\n",
            "            function,\n",
            "        )"
        ),
        concat!(
            "emit_ensure_set_collection_capacity(\n",
            "            SetCollectionKind::Set,\n",
            "            set_record_local,\n",
            "            entries_ptr_local,\n",
            "            entries_len_local,\n",
            "            entries_cap_local,\n",
            "            function,\n",
            "        )"
        ),
    ] {
        assert_eq!(
            COLLECTIONS_SOURCE.matches(call).count(),
            1,
            "producer `{call}`"
        );
    }

    assert_eq!(
        COLLECTIONS_SOURCE.matches("SetCollectionKind::Set").count(),
        10,
        "seven Set producers and three policy arms"
    );
    assert_eq!(
        COLLECTIONS_SOURCE
            .matches("SetCollectionKind::WeakSet")
            .count(),
        7,
        "four WeakSet producers and three policy arms"
    );
}

#[test]
fn add_exhaustively_restricts_only_weak_set_values_before_normalization() {
    let emitter = bounded(
        COLLECTIONS_SOURCE,
        "    fn emit_set_collection_prototype_add(",
        "    pub(crate) fn emit_set_prototype_clear(",
    );
    let policy = concat!(
        "        match collection_kind {\n",
        "            SetCollectionKind::Set => {}\n",
        "            SetCollectionKind::WeakSet => {\n",
        "                self.emit_require_weak_key(\n",
        "                    value_payload_local,\n",
        "                    value_tag_local,\n",
        "                    \"WeakSet value must be an object or unregistered symbol\",\n",
        "                    function,\n",
        "                )?;\n",
        "            }\n",
        "        }\n",
    );

    assert_eq!(emitter.matches(policy).count(), 1);
    assert_before(
        emitter,
        "self.emit_set_collection_record_from_receiver(collection_kind, set_record_local, function)?;",
        "self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);",
    );
    assert_before(
        emitter,
        "self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);",
        policy,
    );
    assert_before(
        emitter,
        policy,
        "function.instruction(&Instruction::LocalGet(value_tag_local));",
    );
    assert_before(emitter, policy, "self.emit_find_set_collection_entry(");
}

#[test]
fn delete_and_has_exhaustively_return_false_for_invalid_weak_set_values() {
    let policy = concat!(
        "        match collection_kind {\n",
        "            SetCollectionKind::Set => {}\n",
        "            SetCollectionKind::WeakSet => {\n",
        "                self.emit_can_be_held_weakly_i32(value_payload_local, value_tag_local, function);\n",
        "                function.instruction(&Instruction::I32Eqz);\n",
        "                function.instruction(&Instruction::If(BlockType::Empty));\n",
        "                function.instruction(&Instruction::I64Const(0));\n",
        "                function.instruction(&Instruction::LocalSet(self.result_local));\n",
        "                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));\n",
        "                function.instruction(&Instruction::LocalSet(self.result_tag_local));\n",
        "                self.emit_return_current_completion(function);\n",
        "                function.instruction(&Instruction::End);\n",
        "            }\n",
        "        }\n",
    );

    for (start, end) in [
        (
            "    fn emit_set_collection_prototype_delete(",
            "    pub(crate) fn emit_set_prototype_has(",
        ),
        (
            "    fn emit_set_collection_prototype_has(",
            "    pub(crate) fn emit_set_prototype_for_each(",
        ),
    ] {
        let emitter = bounded(COLLECTIONS_SOURCE, start, end);
        assert_eq!(emitter.matches(policy).count(), 1, "emitter `{start}`");
        assert_before(
            emitter,
            "self.emit_set_collection_record_from_receiver(collection_kind, set_record_local, function)?;",
            "self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);",
        );
        assert_before(
            emitter,
            "self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);",
            policy,
        );
        assert_before(emitter, policy, "self.emit_find_set_collection_entry(");
    }
}
