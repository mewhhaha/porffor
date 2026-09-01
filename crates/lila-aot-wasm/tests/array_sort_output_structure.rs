use std::fs;
use std::path::Path;

const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str = include_str!("../../../docs/rust-rewrite/contracts/array-sort-output.md");
const TASK_T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const TASK_T16: &str = include_str!("../../../tasks/16-arrays-and-array-builtins.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn window<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_offset = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"));
    let tail = &source[start_offset..];
    let end_offset = tail
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"));
    &tail[..end_offset]
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
fn array_sort_output_is_the_exact_private_non_copyable_domain() {
    let declaration = bounded(
        ARRAY_SOURCE,
        "enum ArraySortOutput {",
        "\n\nenum ToLocaleStringReceiverKind",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        ["Receiver,", "Copy,", "}"]
    );

    let declaration_offset = ARRAY_SOURCE
        .find("enum ArraySortOutput {")
        .expect("missing Array sort output domain");
    assert_eq!(
        ARRAY_SOURCE[..declaration_offset]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("}")
    );
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!ARRAY_SOURCE.contains(&format!("impl {capability} for ArraySortOutput")));
    }
    assert!(!ARRAY_SOURCE.contains("pub(crate) enum ArraySortOutput"));
    assert!(!ARRAY_SOURCE.contains("pub enum ArraySortOutput"));
    assert!(!ARRAY_SOURCE.contains("pub(super) enum ArraySortOutput"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(count_in_rust_sources(&source_root, "ArraySortOutput"), 12);
    assert_eq!(
        count_in_rust_sources(&source_root, "ArraySortOutput::Receiver"),
        5
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "ArraySortOutput::Copy"),
        5
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "compile_array_sort_with_output("),
        3
    );
}

#[test]
fn sort_and_to_sorted_select_their_exact_outputs() {
    let producers = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypeFill => {",
        "            StandardBuiltinId::ArrayPrototypeToLocaleString => {",
    );
    let normalized_producers = normalized(producers);

    for mapping in [
        "StandardBuiltinId::ArrayPrototypeSort=>{self.compile_array_prototype_sort_builtin(function)?;}",
        "StandardBuiltinId::ArrayPrototypeToSorted=>{self.compile_array_prototype_to_sorted_builtin(function)?;}",
    ] {
        assert_eq!(
            normalized_producers.matches(mapping).count(),
            1,
            "producer mapping `{mapping}`"
        );
    }
    assert_eq!(
        producers
            .matches("compile_array_prototype_sort_builtin(")
            .count(),
        1
    );
    assert_eq!(
        producers
            .matches("compile_array_prototype_to_sorted_builtin(")
            .count(),
        1
    );
    for forbidden in [
        "ArraySortOutput",
        "compile_array_sort_with_output(",
        "_ =>",
        "unreachable!",
    ] {
        assert!(!producers.contains(forbidden));
    }

    for (entry, variant) in [
        ("compile_array_prototype_sort_builtin", "Receiver"),
        ("compile_array_prototype_to_sorted_builtin", "Copy"),
    ] {
        let fixed_entry = bounded(
            ARRAY_SOURCE,
            &format!("    pub(super) fn {entry}("),
            "\n    }",
        );
        assert_eq!(
            fixed_entry
                .matches(&format!("ArraySortOutput::{variant}"))
                .count(),
            1
        );
        assert_eq!(
            fixed_entry
                .matches("compile_array_sort_with_output(")
                .count(),
            1
        );
    }
    assert!(!STANDARD_SOURCE.contains("ArraySortOutput"));
    assert!(!STANDARD_SOURCE.contains("compile_array_sort_with_output("));
}

#[test]
fn sort_output_exhaustively_owns_all_four_semantic_projections() {
    let consumer = bounded(
        ARRAY_SOURCE,
        "    fn compile_array_sort_with_output(",
        "    #[allow(clippy::too_many_arguments)]\n    fn emit_array_target_create_data_property_or_throw(",
    );
    assert!(consumer.contains("output: ArraySortOutput,"));
    assert_eq!(consumer.matches("match &output {").count(), 4);
    assert_eq!(consumer.matches("ArraySortOutput::Receiver =>").count(), 4);
    assert_eq!(consumer.matches("ArraySortOutput::Copy =>").count(), 4);
    assert_eq!(consumer.matches("ArraySortOutput::").count(), 8);
    assert_eq!(consumer.matches("self.release_temp_local(").count(), 45);
    for forbidden in [
        "output ==",
        "output !=",
        "if output",
        "matches!(output",
        "output: bool",
        "_ =>",
        "unreachable!",
        "Default::default",
    ] {
        assert!(
            !consumer.contains(forbidden),
            "consumer contains `{forbidden}`"
        );
    }

    let allocation = window(
        consumer,
        "        match &output {\n            ArraySortOutput::Copy => {\n                function.instruction(&Instruction::LocalGet(len_local));",
        "\n\n        self.emit_is_typed_array_i32(",
    );
    let allocation_copy = bounded(
        allocation,
        "ArraySortOutput::Copy => {",
        "ArraySortOutput::Receiver => {",
    );
    let allocation_receiver = allocation
        .split_once("ArraySortOutput::Receiver => {")
        .expect("missing Receiver allocation arm")
        .1;
    assert_eq!(
        normalized(allocation_copy),
        "function.instruction(&Instruction::LocalGet(len_local));function.instruction(&Instruction::I64Const(u32::MAXasi64));function.instruction(&Instruction::I64GtU);function.instruction(&Instruction::If(BlockType::Empty));self.emit_throw_current_function_realm_range_error(\"Invalidarraylength\",self.result_local,self.result_tag_local,function,)?;self.emit_return_current_completion(function);function.instruction(&Instruction::End);self.emit_alloc_array_payload_with_length(len_local,target_payload_local,function,)?;function.instruction(&Instruction::I64Const(ValueKind::Array.tag()asi64));function.instruction(&Instruction::LocalSet(target_tag_local));}"
    );
    assert_eq!(
        normalized(allocation_receiver),
        "function.instruction(&Instruction::LocalGet(receiver_payload_local));function.instruction(&Instruction::LocalSet(target_payload_local));function.instruction(&Instruction::LocalGet(receiver_tag_local));function.instruction(&Instruction::LocalSet(target_tag_local));}}"
    );

    let presence = window(
        consumer,
        "        match &output {\n            ArraySortOutput::Copy => {\n                function.instruction(&Instruction::I64Const(1));\n                function.instruction(&Instruction::LocalSet(has_property_local));",
        "\n        function.instruction(&Instruction::LocalGet(has_property_local));",
    );
    let presence_copy = bounded(
        presence,
        "ArraySortOutput::Copy => {",
        "ArraySortOutput::Receiver => {",
    );
    let presence_receiver = presence
        .split_once("ArraySortOutput::Receiver => {")
        .expect("missing Receiver presence arm")
        .1;
    assert_eq!(
        normalized(presence_copy),
        "function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::LocalSet(has_property_local));}"
    );
    assert_eq!(
        presence_receiver
            .matches("self.emit_object_has_property_i32(")
            .count(),
        1
    );
    assert_eq!(
        presence_receiver
            .matches("Instruction::LocalGet(receiver_is_typed_array_local)")
            .count(),
        1
    );
    assert!(normalized(presence_receiver).contains(
        "self.emit_object_has_property_i32(receiver_payload_local,receiver_tag_local,key_local,has_property_local,function,)?;function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));function.instruction(&Instruction::I64Eqz);function.instruction(&Instruction::I32Eqz);function.instruction(&Instruction::If(BlockType::Empty));function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::LocalSet(has_property_local));function.instruction(&Instruction::End);"
    ));
    assert!(!presence_receiver.contains("emit_array_write("));

    let publication = window(
        consumer,
        "        match &output {\n            ArraySortOutput::Copy => {\n                self.emit_array_write(",
        "\n        self.emit_return_current_completion_if_throw(function);",
    );
    let publication_copy = bounded(
        publication,
        "ArraySortOutput::Copy => {",
        "ArraySortOutput::Receiver => {",
    );
    let publication_receiver = publication
        .split_once("ArraySortOutput::Receiver => {")
        .expect("missing Receiver publication arm")
        .1;
    assert_eq!(
        publication_copy.matches("self.emit_array_write(").count(),
        1
    );
    assert!(normalized(publication_copy).contains(
        "self.emit_array_write(target_payload_local,source_index_local,collected_payload_local,collected_tag_local,function,)?;"
    ));
    assert!(!publication_copy.contains("emit_object_write_strict("));
    assert!(!publication_copy.contains("emit_typed_array_element_write_from_locals("));
    assert_eq!(
        publication_receiver
            .matches("self.emit_object_write_strict(")
            .count(),
        1
    );
    assert_eq!(
        publication_receiver
            .matches("self.emit_typed_array_element_write_from_locals(")
            .count(),
        1
    );
    assert!(normalized(publication_receiver).contains(
        "function.instruction(&Instruction::LocalGet(receiver_is_typed_array_local));function.instruction(&Instruction::I64Eqz);function.instruction(&Instruction::If(BlockType::Empty));self.emit_object_write_strict(receiver_payload_local,receiver_tag_local,key_local,collected_payload_local,collected_tag_local,function,)?;function.instruction(&Instruction::Else);self.emit_typed_array_element_write_from_locals(receiver_payload_local,source_index_local,collected_payload_local,collected_tag_local,function,)?;function.instruction(&Instruction::End);"
    ));
    assert!(!publication_receiver.contains("self.emit_array_write("));

    let deletion = window(
        consumer,
        "        match &output {\n            ArraySortOutput::Receiver => {\n                function.instruction(&Instruction::LocalGet(buffer_len_local));",
        "\n\n        function.instruction(&Instruction::LocalGet(target_payload_local));",
    );
    let deletion_receiver = bounded(
        deletion,
        "ArraySortOutput::Receiver => {",
        "ArraySortOutput::Copy => {}",
    );
    assert_eq!(
        normalized(deletion_receiver),
        "function.instruction(&Instruction::LocalGet(buffer_len_local));function.instruction(&Instruction::LocalSet(source_index_local));function.instruction(&Instruction::Block(BlockType::Empty));function.instruction(&Instruction::Loop(BlockType::Empty));function.instruction(&Instruction::LocalGet(source_index_local));function.instruction(&Instruction::LocalGet(len_local));function.instruction(&Instruction::I64GeU);function.instruction(&Instruction::BrIf(1));self.emit_index_to_flat_map_key_local(source_index_local,source_number_payload_local,key_local,function,)?;self.emit_delete_property_or_throw(receiver_payload_local,receiver_tag_local,key_local,function,)?;function.instruction(&Instruction::LocalGet(source_index_local));function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::I64Add);function.instruction(&Instruction::LocalSet(source_index_local));function.instruction(&Instruction::Br(0));function.instruction(&Instruction::End);function.instruction(&Instruction::End);}"
    );
    assert_eq!(deletion.matches("ArraySortOutput::Copy => {}").count(), 1);

    let common_tail = consumer
        .split_once("        function.instruction(&Instruction::LocalGet(target_payload_local));")
        .expect("missing common result publication")
        .1;
    let first_release = common_tail
        .find("self.release_temp_local(")
        .expect("missing common release tail");
    assert_eq!(
        normalized(&common_tail[..first_release]),
        "function.instruction(&Instruction::LocalSet(self.result_local));function.instruction(&Instruction::LocalGet(target_tag_local));function.instruction(&Instruction::LocalSet(self.result_tag_local));self.set_completion_kind(CompletionKind::Normal,function);"
    );
    let released_locals = common_tail
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("self.release_temp_local(")
                .and_then(|line| line.strip_suffix(");"))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        released_locals,
        [
            "target_tag_local",
            "target_payload_local",
            "previous_string_payload_local",
            "key_string_payload_local",
            "compare_number_payload_local",
            "compare_result_tag_local",
            "compare_result_payload_local",
            "should_shift_local",
            "previous_tag_local",
            "previous_payload_local",
            "sort_key_tag_local",
            "sort_key_payload_local",
            "previous_entry_local",
            "preceding_index_local",
            "previous_index_local",
            "sort_index_local",
            "entry_local",
            "destination_entry_local",
            "source_entry_local",
            "copy_index_local",
            "new_buffer_size_local",
            "new_buffer_cap_local",
            "new_buffer_local",
            "buffer_cap_local",
            "buffer_len_local",
            "buffer_local",
            "collected_tag_local",
            "collected_payload_local",
            "receiver_is_typed_array_local",
            "typed_array_byte_length_tag_local",
            "typed_array_byte_length_payload_local",
            "has_property_local",
            "key_local",
            "source_number_payload_local",
            "source_index_local",
            "undefined_this_tag_local",
            "undefined_this_payload_local",
            "has_compare_local",
            "compare_tag_local",
            "compare_payload_local",
            "len_local",
            "length_tag_local",
            "length_payload_local",
            "receiver_tag_local",
            "receiver_payload_local",
        ]
    );
    let release_tail_lines = common_tail[first_release..]
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(release_tail_lines.len(), released_locals.len() + 2);
    for (line, local) in release_tail_lines.iter().zip(&released_locals) {
        assert_eq!(*line, format!("self.release_temp_local({local});"));
    }
    assert_eq!(release_tail_lines[released_locals.len()], "Ok(())");
    assert_eq!(release_tail_lines[released_locals.len() + 1], "}");

    let allocation_offset = consumer.find("match &output {").expect("allocation match");
    let presence_offset = consumer[allocation_offset + 1..]
        .find("match &output {")
        .map(|offset| allocation_offset + 1 + offset)
        .expect("presence match");
    let publication_offset = consumer[presence_offset + 1..]
        .find("match &output {")
        .map(|offset| presence_offset + 1 + offset)
        .expect("publication match");
    let deletion_offset = consumer[publication_offset + 1..]
        .find("match &output {")
        .map(|offset| publication_offset + 1 + offset)
        .expect("deletion match");
    let result_offset = consumer
        .find("Instruction::LocalSet(self.result_local)")
        .expect("result publication");
    assert!(allocation_offset < presence_offset);
    assert!(presence_offset < publication_offset);
    assert!(publication_offset < deletion_offset);
    assert!(deletion_offset < result_offset);
}

#[test]
fn array_sort_output_contract_records_fixed_dispatch_witnesses_and_nonclaims() {
    for marker in [
        "private, non-derived domain",
        "two fixed entries",
        "1745b093aab4e0643c08de0b1d402f3770ef5a9618635ae7b31ec318a8c74c4c",
        "aa8c4c988b2c5e64568cfc9f4a294c98a32144af941450cf59ac882948afbf25",
        "no new Array behavior",
        "does not close T16",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker: {marker}"
        );
    }
    for task in [TASK_T02, TASK_T16] {
        assert!(task.contains("array-sort-output.md"));
        assert!(task.contains("1745b093aab4e0643c08de0b1d402f3770ef5a9618635ae7b31ec318a8c74c4c"));
        assert!(task.contains("4/4"));
        assert!(task.contains("no new Array behavior"));
    }
}
