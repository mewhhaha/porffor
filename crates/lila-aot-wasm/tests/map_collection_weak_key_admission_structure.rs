use std::fs;
use std::path::Path;

const COLLECTIONS_PARENT: &str = include_str!("../src/builtins/collections.rs");
const MAP_GET_OR_INSERT: &str = include_str!("../src/builtins/collections/map_get_or_insert.rs");
const COLLECTIONS_RECURSIVE: &str = concat!(
    include_str!("../src/builtins/collections.rs"),
    include_str!("../src/builtins/collections/map_get_or_insert.rs"),
);

const FALSE_POLICY: &str = concat!(
    "        match collection_kind {\n",
    "            MapCollectionKind::Map => {}\n",
    "            MapCollectionKind::WeakMap => {\n",
    "                self.emit_can_be_held_weakly_i32(key_payload_local, key_tag_local, function);\n",
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

const UNDEFINED_POLICY: &str = concat!(
    "        match collection_kind {\n",
    "            MapCollectionKind::Map => {}\n",
    "            MapCollectionKind::WeakMap => {\n",
    "                self.emit_can_be_held_weakly_i32(key_payload_local, key_tag_local, function);\n",
    "                function.instruction(&Instruction::I32Eqz);\n",
    "                function.instruction(&Instruction::If(BlockType::Empty));\n",
    "                function.instruction(&Instruction::I64Const(0));\n",
    "                function.instruction(&Instruction::LocalSet(self.result_local));\n",
    "                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));\n",
    "                function.instruction(&Instruction::LocalSet(self.result_tag_local));\n",
    "                self.emit_return_current_completion(function);\n",
    "                function.instruction(&Instruction::End);\n",
    "            }\n",
    "        }\n",
);

const REQUIRE_POLICY: &str = concat!(
    "match collection_kind {\n",
    "                    MapCollectionKind::Map => {}\n",
    "                    MapCollectionKind::WeakMap => {\n",
    "                        self.emit_require_weak_key(\n",
    "                            key_payload_local,\n",
    "                            key_tag_local,\n",
    "                            \"WeakMap key must be an object or unregistered symbol\",\n",
    "                            function,\n",
    "                        )?;\n",
    "                    }\n",
    "                }",
);

const SET_REQUIRE_POLICY: &str = concat!(
    "        match collection_kind {\n",
    "            MapCollectionKind::Map => {}\n",
    "            MapCollectionKind::WeakMap => {\n",
    "                self.emit_require_weak_key(\n",
    "                    key_payload_local,\n",
    "                    key_tag_local,\n",
    "                    \"WeakMap key must be an object or unregistered symbol\",\n",
    "                    function,\n",
    "                )?;\n",
    "            }\n",
    "        }",
);

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

fn without_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn map_collection_kind_is_closed_without_equality_policy() {
    let declaration = bounded(
        COLLECTIONS_PARENT,
        "#[derive(Clone, Copy)]\nenum MapCollectionKind {",
        "#[derive(Clone, Copy)]\nenum SetCollectionKind",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take_while(|line| *line != "}")
        .collect::<Vec<_>>();
    assert_eq!(variants, ["Map,", "WeakMap,"]);
    assert!(!declaration.contains("pub"));
    assert!(!declaration.contains("Default"));
    assert!(!declaration.contains("Debug"));
    assert!(!COLLECTIONS_RECURSIVE.contains("PartialEq for MapCollectionKind"));
    assert!(!COLLECTIONS_RECURSIVE.contains("Eq for MapCollectionKind"));
    assert!(!COLLECTIONS_RECURSIVE.contains("collection_kind == MapCollectionKind"));
    assert!(!COLLECTIONS_RECURSIVE.contains("collection_kind != MapCollectionKind"));

    let implementation = bounded(
        COLLECTIONS_PARENT,
        "impl MapCollectionKind {",
        "#[derive(Clone, Copy)]\nenum SetAlgebraOperation",
    );
    assert_eq!(implementation.matches("match self {").count(), 6);
    assert_eq!(implementation.matches("Self::Map =>").count(), 6);
    assert_eq!(implementation.matches("Self::WeakMap =>").count(), 6);
    assert_eq!(
        implementation
            .matches("self.receiver_kind().brand()")
            .count(),
        1
    );
    for forbidden in ["_ =>", "unreachable!", "default", "==", "!="] {
        assert!(
            !implementation.contains(forbidden),
            "forbidden `{forbidden}`"
        );
    }
}

#[test]
fn named_entry_points_are_the_only_map_collection_kind_producers() {
    let normalized = without_whitespace(COLLECTIONS_RECURSIVE);
    for call in [
        "emit_map_collection_record_from_receiver(MapCollectionKind::Map,map_record_local,function,)",
        "emit_map_collection_constructor(MapCollectionKind::Map,function)",
        "emit_map_collection_constructor(MapCollectionKind::WeakMap,function)",
        "emit_map_collection_prototype_delete(MapCollectionKind::Map,function)",
        "emit_map_collection_prototype_delete(MapCollectionKind::WeakMap,function)",
        "emit_map_collection_prototype_get(MapCollectionKind::Map,function)",
        "emit_map_collection_prototype_get(MapCollectionKind::WeakMap,function)",
        "emit_map_prototype_get_or_insert_inner(MapCollectionKind::Map,MapGetOrInsertValueSource::ValueArgument,function,)",
        "emit_map_prototype_get_or_insert_inner(MapCollectionKind::Map,MapGetOrInsertValueSource::ComputedCallback,function,)",
        "emit_map_prototype_get_or_insert_inner(MapCollectionKind::WeakMap,MapGetOrInsertValueSource::ValueArgument,function,)",
        "emit_map_prototype_get_or_insert_inner(MapCollectionKind::WeakMap,MapGetOrInsertValueSource::ComputedCallback,function,)",
        "emit_map_collection_prototype_has(MapCollectionKind::Map,function)",
        "emit_map_collection_prototype_has(MapCollectionKind::WeakMap,function)",
        "emit_map_collection_prototype_set(MapCollectionKind::Map,function)",
        "emit_map_collection_prototype_set(MapCollectionKind::WeakMap,function)",
    ] {
        assert_eq!(normalized.matches(call).count(), 1, "producer `{call}`");
    }

    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (variant, expected) in [("Map", 15), ("WeakMap", 14)] {
        let needle = format!("MapCollectionKind::{variant}");
        let owned_count = COLLECTIONS_RECURSIVE.matches(needle.as_str()).count();
        assert_eq!(owned_count, expected, "variant `{variant}`");
        assert_eq!(
            count_in_rust_sources(&src, needle.as_str()),
            owned_count,
            "variant `{variant}` must remain owned by the collections builtin tree"
        );
    }
}

#[test]
fn delete_get_and_has_exhaustively_reject_invalid_weak_keys_before_lookup() {
    for (start, end, policy) in [
        (
            "    fn emit_map_collection_prototype_delete(",
            "    pub(crate) fn emit_map_prototype_get(",
            FALSE_POLICY,
        ),
        (
            "    fn emit_map_collection_prototype_get(",
            "    pub(crate) fn emit_map_prototype_has(",
            UNDEFINED_POLICY,
        ),
        (
            "    fn emit_map_collection_prototype_has(",
            "    pub(crate) fn emit_map_prototype_for_each(",
            FALSE_POLICY,
        ),
    ] {
        let emitter = bounded(COLLECTIONS_PARENT, start, end);
        assert_eq!(emitter.matches(policy).count(), 1, "emitter `{start}`");
        assert!(!emitter.contains("_ =>"), "emitter `{start}`");
        assert_before(
            emitter,
            "self.emit_map_collection_record_from_receiver(collection_kind, map_record_local, function)?;",
            "self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);",
        );
        assert_before(
            emitter,
            "self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);",
            policy,
        );
        assert_before(emitter, policy, "self.emit_find_map_entry(");
    }
}

#[test]
fn insertion_paths_exhaustively_require_weak_keys_in_the_existing_order() {
    let get_or_insert = bounded(
        MAP_GET_OR_INSERT,
        "    fn emit_map_prototype_get_or_insert_inner(",
        "\n    }\n}",
    );
    assert_eq!(get_or_insert.matches(REQUIRE_POLICY).count(), 2);
    assert!(!get_or_insert.contains("_ =>"));

    let preparation = bounded(
        get_or_insert,
        "        match value_source {",
        "        function.instruction(&Instruction::LocalGet(key_tag_local));",
    );
    let value_argument = bounded(
        preparation,
        "            MapGetOrInsertValueSource::ValueArgument => {",
        "            MapGetOrInsertValueSource::ComputedCallback => {",
    );
    assert_before(
        value_argument,
        REQUIRE_POLICY,
        "self.emit_builtin_arg_to_locals(1,",
    );

    let computed_callback = get_or_insert
        .split_once("            MapGetOrInsertValueSource::ComputedCallback => {")
        .expect("computed callback preparation")
        .1;
    assert_before(
        computed_callback,
        "self.emit_builtin_arg_to_locals(\n                    1,\n                    callback_payload_local,",
        "self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;",
    );
    assert_before(
        computed_callback,
        "self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;",
        "self.emit_throw_current_function_realm_type_error(",
    );
    assert_before(
        computed_callback,
        "self.emit_throw_current_function_realm_type_error(",
        REQUIRE_POLICY,
    );
    assert_before(
        computed_callback,
        REQUIRE_POLICY,
        "self.emit_find_map_entry(",
    );

    let set = bounded(
        COLLECTIONS_PARENT,
        "    fn emit_map_collection_prototype_set(",
        "    pub(crate) fn emit_set_prototype_add(",
    );
    assert_eq!(set.matches(SET_REQUIRE_POLICY).count(), 1);
    assert!(!set.contains("_ =>"));
    assert_before(
        set,
        "self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;",
        "self.emit_map_collection_record_from_receiver(collection_kind, map_record_local, function)?;",
    );
    assert_before(
        set,
        "self.emit_map_collection_record_from_receiver(collection_kind, map_record_local, function)?;",
        "self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);",
    );
    assert_before(
        set,
        "self.emit_builtin_arg_to_locals(0, key_payload_local, key_tag_local, function);",
        "self.emit_builtin_arg_to_locals(1, value_payload_local, value_tag_local, function);",
    );
    assert_before(
        set,
        "self.emit_builtin_arg_to_locals(1, value_payload_local, value_tag_local, function);",
        SET_REQUIRE_POLICY,
    );
    assert_before(
        set,
        SET_REQUIRE_POLICY,
        "function.instruction(&Instruction::LocalGet(key_tag_local));",
    );
    assert_before(set, SET_REQUIRE_POLICY, "self.emit_find_map_entry(");
}
