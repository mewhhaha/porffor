use std::fs;
use std::path::Path;

const COLLECTIONS_SOURCE: &str = include_str!("../src/builtins/collections.rs");
const CLI_ITERATOR_TESTS: &str = include_str!("../../lila-cli/tests/cli/iterator.rs");
const DATA_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_collection_data_receiver_realm.js");
const ITERATOR_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_collection_iterator_receiver_realm.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/collection-receiver-error-domain.md");
const TASK: &str = include_str!("../../../tasks/21-symbols-collections-weakrefs.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
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
fn receiver_error_is_the_exact_private_capability_free_domain() {
    let declaration_marker = "enum CollectionReceiverError {";
    let declaration_offset = COLLECTIONS_SOURCE
        .find(declaration_marker)
        .expect("receiver-error declaration");
    let preceding_item_end = COLLECTIONS_SOURCE[..declaration_offset]
        .rfind('}')
        .expect("item before receiver-error declaration");
    let following_item_offset = COLLECTIONS_SOURCE[declaration_offset..]
        .find("impl CollectionDataReceiverKind")
        .map(|offset| declaration_offset + offset)
        .expect("item after receiver-error declaration");
    assert_eq!(
        normalized(&COLLECTIONS_SOURCE[preceding_item_end + 1..following_item_offset]),
        "enumCollectionReceiverError{NonObject,MissingInternalSlots,}",
        "the exact declaration region must remain private and attribute-free"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "CollectionReceiverError"),
        19,
        "one declaration, three typed parameters, twelve message arms and three validator producers own every mention"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "CollectionReceiverError::NonObject"),
        7,
        "six message rows and one validator producer"
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "CollectionReceiverError::MissingInternalSlots"
        ),
        8,
        "six message rows and two validator producers"
    );
    for forbidden in [
        "impl CollectionReceiverError",
        "for CollectionReceiverError",
    ] {
        assert!(
            !COLLECTIONS_SOURCE.contains(forbidden),
            "found `{forbidden}`"
        );
    }
}

#[test]
fn receiver_error_message_tables_and_forwarding_are_exact() {
    let data_impl = bounded(
        COLLECTIONS_SOURCE,
        "impl CollectionDataReceiverKind {",
        "impl CollectionReceiverRequirement {",
    );
    let data_messages = bounded(
        data_impl,
        "    fn receiver_error_message(self, error: CollectionReceiverError) -> &'static str {",
        "\n    }\n}",
    );
    let expected_data_messages = r#"
        match (self, error) {
            (Self::Map, CollectionReceiverError::NonObject) => {
                "Map method receiver is not an object"
            }
            (Self::Map, CollectionReceiverError::MissingInternalSlots) => {
                "Map method receiver does not have [[MapData]]"
            }
            (Self::WeakMap, CollectionReceiverError::NonObject) => {
                "WeakMap method receiver is not an object"
            }
            (Self::WeakMap, CollectionReceiverError::MissingInternalSlots) => {
                "WeakMap method receiver does not have [[WeakMapData]]"
            }
            (Self::Set, CollectionReceiverError::NonObject) => {
                "Set method receiver is not an object"
            }
            (Self::Set, CollectionReceiverError::MissingInternalSlots) => {
                "Set method receiver does not have [[SetData]]"
            }
            (Self::WeakSet, CollectionReceiverError::NonObject) => {
                "WeakSet method receiver is not an object"
            }
            (Self::WeakSet, CollectionReceiverError::MissingInternalSlots) => {
                "WeakSet method receiver does not have [[WeakSetData]]"
            }
        }
"#;
    assert_eq!(
        normalized(data_messages),
        normalized(expected_data_messages)
    );

    let requirement_impl = bounded(
        COLLECTIONS_SOURCE,
        "impl CollectionReceiverRequirement {",
        "impl StrongCollectionCursor {",
    );
    let forwarding = bounded(
        requirement_impl,
        "    fn receiver_error_message(self, error: CollectionReceiverError) -> &'static str {",
        "\n    }\n}",
    );
    assert_eq!(
        normalized(forwarding),
        "matchself{Self::Data(kind)=>kind.receiver_error_message(error),Self::Iterator(cursor)=>cursor.receiver_error_message(error),}"
    );

    let cursor_impl = bounded(
        COLLECTIONS_SOURCE,
        "impl StrongCollectionCursor {",
        "#[derive(Clone, Copy)]\nenum GroupByResult",
    );
    let iterator_messages = bounded(
        cursor_impl,
        "    fn receiver_error_message(self, error: CollectionReceiverError) -> &'static str {",
        "\n    }\n\n    fn collection_payload_offset",
    );
    let expected_iterator_messages = r#"
        match (self, error) {
            (Self::Map, CollectionReceiverError::NonObject) => {
                "Map Iterator.prototype.next receiver is not an object"
            }
            (Self::Map, CollectionReceiverError::MissingInternalSlots) => {
                "Map Iterator.prototype.next receiver does not have [[Map]]"
            }
            (Self::Set, CollectionReceiverError::NonObject) => {
                "Set Iterator.prototype.next receiver is not an object"
            }
            (Self::Set, CollectionReceiverError::MissingInternalSlots) => {
                "Set Iterator.prototype.next receiver does not have [[Set]]"
            }
        }
"#;
    assert_eq!(
        normalized(iterator_messages),
        normalized(expected_iterator_messages)
    );
    for message in [
        "Map method receiver is not an object",
        "Map method receiver does not have [[MapData]]",
        "WeakMap method receiver is not an object",
        "WeakMap method receiver does not have [[WeakMapData]]",
        "Set method receiver is not an object",
        "Set method receiver does not have [[SetData]]",
        "WeakSet method receiver is not an object",
        "WeakSet method receiver does not have [[WeakSetData]]",
        "Map Iterator.prototype.next receiver is not an object",
        "Map Iterator.prototype.next receiver does not have [[Map]]",
        "Set Iterator.prototype.next receiver is not an object",
        "Set Iterator.prototype.next receiver does not have [[Set]]",
    ] {
        let string_literal = format!("\"{message}\"");
        assert_eq!(
            COLLECTIONS_SOURCE.matches(&string_literal).count(),
            1,
            "exact receiver message `{message}`"
        );
    }
    for body in [data_messages, forwarding, iterator_messages] {
        for forbidden in ["_ =>", "error ==", "error !=", "matches!(error"] {
            assert!(!body.contains(forbidden), "found `{forbidden}`");
        }
    }
}

#[test]
fn receiver_representation_arms_select_the_exact_failure_in_order() {
    let validator = bounded(
        COLLECTIONS_SOURCE,
        "    fn emit_collection_record_from_receiver(",
        "    fn emit_strong_collection_iterator_record_from_receiver(",
    );
    let representation_dispatch = bounded(
        validator,
        "        function.instruction(&Instruction::Block(BlockType::Empty));",
        "        function.instruction(&Instruction::Unreachable);\n        function.instruction(&Instruction::End);",
    );
    let representation_match = bounded(
        representation_dispatch,
        "            match representation {",
        "            function.instruction(&Instruction::Br(1));",
    );
    let object_tag = bounded(
        representation_match,
        "                CollectionReceiverRepresentation::ObjectTagBrandLayout => {",
        "                CollectionReceiverRepresentation::ObjectWithoutBrandLayout => {",
    );
    let object_without_brand = bounded(
        representation_match,
        "                CollectionReceiverRepresentation::ObjectWithoutBrandLayout => {",
        "                CollectionReceiverRepresentation::NonObject => {",
    );
    let non_object = bounded(
        representation_match,
        "                CollectionReceiverRepresentation::NonObject => {",
        "                CollectionReceiverRepresentation::NonRuntime => {",
    );
    let non_runtime = bounded(
        representation_match,
        "                CollectionReceiverRepresentation::NonRuntime => {",
        "            }\n",
    );

    let expected_object_tag = r#"
                    self.load_i64_to_local_from_offset(
                        receiver_payload_local,
                        HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                        receiver_brand_local,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(receiver_brand_local));
                    function.instruction(&Instruction::I64Const(requirement.brand() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        receiver_payload_local,
                        HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                        record_local,
                        function,
                    );
                    function.instruction(&Instruction::Else);
                    self.emit_throw_current_function_realm_type_error(
                        requirement
                            .receiver_error_message(CollectionReceiverError::MissingInternalSlots),
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion(function);
                    function.instruction(&Instruction::End);
                }
"#;
    assert_eq!(normalized(object_tag), normalized(expected_object_tag));

    let expected_object_without_brand = r#"
                    self.emit_throw_current_function_realm_type_error(
                        requirement
                            .receiver_error_message(CollectionReceiverError::MissingInternalSlots),
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion(function);
                }
"#;
    assert_eq!(
        normalized(object_without_brand),
        normalized(expected_object_without_brand)
    );

    let expected_non_object = r#"
                    self.emit_throw_current_function_realm_type_error(
                        requirement.receiver_error_message(CollectionReceiverError::NonObject),
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion(function);
                }
"#;
    assert_eq!(normalized(non_object), normalized(expected_non_object));
    assert_eq!(non_runtime.matches("Instruction::Unreachable").count(), 1);
    assert!(!non_runtime.contains("CollectionReceiverError"));

    let missing = "receiver_error_message(CollectionReceiverError::MissingInternalSlots)";
    let non_object_error = "receiver_error_message(CollectionReceiverError::NonObject)";
    let normalized_match = normalized(representation_match);
    let mut preceding_error_end = 0;
    for error in [missing, missing, non_object_error] {
        let error_offset = normalized_match[preceding_error_end..]
            .find(&normalized(error))
            .map(|offset| preceding_error_end + offset)
            .expect("ordered receiver error producer");
        preceding_error_end = error_offset + normalized(error).len();
    }
    assert_eq!(validator.matches(missing).count(), 2);
    assert_eq!(validator.matches(non_object_error).count(), 1);
}

#[test]
fn contract_and_existing_fixture_sources_cover_both_error_categories() {
    assert!(CONTRACT.contains("CollectionReceiverError"));
    assert!(TASK.contains("collection-receiver-error-domain.md"));
    for test_name in [
        "fn run_wasm_backend_succeeds_for_collection_iterator_receiver_realm_fixture()",
        "fn run_wasm_backend_succeeds_for_collection_data_receiver_realm_fixture()",
    ] {
        assert!(
            CLI_ITERATOR_TESTS.contains(test_name),
            "missing `{test_name}`"
        );
    }
    for marker in [
        "Map Iterator.prototype.next receiver is not an object",
        "Set Iterator.prototype.next receiver does not have [[Set]]",
    ] {
        assert!(ITERATOR_FIXTURE.contains(marker), "missing `{marker}`");
    }
    for marker in [
        "WeakMap method receiver is not an object",
        "WeakSet method receiver does not have [[WeakSetData]]",
    ] {
        assert!(DATA_FIXTURE.contains(marker), "missing `{marker}`");
    }
}
