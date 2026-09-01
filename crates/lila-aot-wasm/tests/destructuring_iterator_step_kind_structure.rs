use std::fs;
use std::path::Path;

const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const CLI_ARRAY_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const ITERATOR_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_destructuring_iterators.js");
const ABRUPT_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_destructuring_iterator_abrupt.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-destructuring-iterator-step-kind.md");
const TASK: &str = include_str!("../../../tasks/15-generators-iterators-resource-management.md");

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

fn positions_in_order(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let offset = source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker `{marker}`"));
        cursor += offset + marker.len();
    }
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
fn step_kind_is_the_exact_private_no_capability_domain() {
    let declaration_marker = "enum DestructuringIteratorStepKind {";
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches(&format!("\n{declaration_marker}"))
            .count(),
        1,
        "the declaration must remain private"
    );
    let declaration_offset = CONTROL_FLOW_SOURCE
        .find(declaration_marker)
        .expect("missing step-kind declaration");
    let preceding_source = &CONTROL_FLOW_SOURCE[..declaration_offset];
    let preceding_item_end = preceding_source
        .rfind('}')
        .expect("missing item before step-kind declaration");
    assert!(
        preceding_source[preceding_item_end + 1..].trim().is_empty(),
        "the declaration must remain directly attribute-free"
    );
    assert_eq!(
        normalized(bounded(
            CONTROL_FLOW_SOURCE,
            declaration_marker,
            "#[must_use = \"a prepared destructuring target must be consumed by its write\"]",
        )),
        "Elision,Value,}"
    );
    assert!(!CONTROL_FLOW_SOURCE.contains("impl DestructuringIteratorStepKind"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "DestructuringIteratorStepKind"),
        7,
        "the declaration, typed consumer, two exhaustive arms and three producers own every mention"
    );
}

#[test]
fn exactly_three_array_element_producers_select_their_step_kind() {
    let producer = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_array_destructuring_element(",
        "    fn emit_destructuring_iterator_step(",
    );
    assert_eq!(
        producer
            .matches("self.emit_destructuring_iterator_step(")
            .count(),
        3
    );
    assert_eq!(
        producer
            .matches("DestructuringIteratorStepKind::Elision,")
            .count(),
        1
    );
    assert_eq!(
        producer
            .matches("DestructuringIteratorStepKind::Value,")
            .count(),
        2
    );

    let elision_arm = bounded(
        producer,
        "            ArrayDestructuringElementIr::Elision => {",
        "            ArrayDestructuringElementIr::Target { target, default } => {",
    );
    assert_eq!(
        elision_arm
            .matches("DestructuringIteratorStepKind::Elision,")
            .count(),
        1
    );
    assert!(!elision_arm.contains("DestructuringIteratorStepKind::Value,"));

    let target_arm = bounded(
        producer,
        "            ArrayDestructuringElementIr::Target { target, default } => {",
        "            ArrayDestructuringElementIr::Rest { target } => {",
    );
    assert_eq!(
        target_arm
            .matches("DestructuringIteratorStepKind::Value,")
            .count(),
        1
    );
    assert!(!target_arm.contains("DestructuringIteratorStepKind::Elision,"));

    let rest_arm = bounded(
        producer,
        "            ArrayDestructuringElementIr::Rest { target } => {",
        "\n        }\n        Ok(())",
    );
    assert_eq!(
        rest_arm
            .matches("DestructuringIteratorStepKind::Value,")
            .count(),
        1
    );
    assert!(!rest_arm.contains("DestructuringIteratorStepKind::Elision,"));

    let normalized_producer = normalized(producer);
    for (kind, expected_count) in [("Elision", 1), ("Value", 2)] {
        let call = normalized(&format!(
            r#"self.emit_destructuring_iterator_step(
                    locals,
                    DestructuringIteratorStepKind::{kind},
                    consumer,
                    function,
                )?;"#
        ));
        assert_eq!(
            normalized_producer.matches(&call).count(),
            expected_count,
            "wrong `{kind}` producer mapping"
        );
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "emit_destructuring_iterator_step("),
        4,
        "the private definition and three array-element calls are the complete census"
    );
}

#[test]
fn step_protocol_failures_use_the_typed_authority_after_marking_done() {
    let consumer = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_destructuring_iterator_step(",
        "    pub(crate) fn emit_sync_iterator_step_value(",
    );
    let signature = bounded(consumer, "&mut self,", ") -> Result<(), EmitError> {");
    assert!(signature.contains("consumer: &SyncIteratorConsumer,"));
    assert_eq!(
        consumer
            .matches("self.emit_sync_iterator_protocol_type_error(")
            .count(),
        2
    );
    assert_eq!(
        consumer
            .matches("SyncIteratorProtocolError::NextNotCallable")
            .count(),
        1
    );
    assert_eq!(
        consumer
            .matches("SyncIteratorProtocolError::NextResultNotObject")
            .count(),
        1
    );
    assert_eq!(consumer.matches("emit_throw_runtime_error(").count(), 0);
    assert_eq!(
        consumer
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        0
    );
    assert_eq!(consumer.matches("emit_iterator_close").count(), 0);

    positions_in_order(
        consumer,
        &[
            "Instruction::I64Const(1)",
            "Instruction::LocalSet(locals.done)",
            "self.emit_is_callable_i32(locals.next_tag, locals.next_payload, function)?;",
            "SyncIteratorProtocolError::NextNotCallable",
            "self.emit_function_handle_call(",
            "function.instruction(&Instruction::Else);",
            "self.emit_function_or_proxy_call_leave_throw_completion(",
            "self.emit_is_heap_object_like_tag_i32(locals.result_tag, function);",
            "Instruction::I64Const(1)",
            "Instruction::LocalSet(locals.done)",
            "SyncIteratorProtocolError::NextResultNotObject",
        ],
    );
}

#[test]
fn step_kind_exhaustively_owns_the_iterator_value_read() {
    let consumer = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_destructuring_iterator_step(",
        "    pub(crate) fn emit_sync_iterator_step_value(",
    );
    let signature = bounded(consumer, "&mut self,", ") -> Result<(), EmitError> {");
    assert!(signature.contains("step_kind: DestructuringIteratorStepKind,"));
    assert!(signature.contains("consumer: &SyncIteratorConsumer,"));
    assert!(!signature.contains("read_value: bool"));

    let projection = bounded(
        consumer,
        "        match step_kind {",
        "        self.pop_control(ControlFrameKind::If);",
    );
    let expected_projection = r#"
            DestructuringIteratorStepKind::Elision => {}
            DestructuringIteratorStepKind::Value => {
                function.instruction(&Instruction::I64Const(self.strings.payload("value")));
                function.instruction(&Instruction::LocalSet(locals.key));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(locals.done));
                self.emit_object_read(
                    locals.result_payload,
                    locals.result_tag,
                    locals.result_payload,
                    locals.result_tag,
                    locals.key,
                    locals.value_payload,
                    locals.value_tag,
                    function,
                )?;
                self.emit_propagate_current_completion_if_throw(function);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(locals.done));
            }
        }
"#;
    assert_eq!(normalized(projection), normalized(expected_projection));
    assert_eq!(consumer.matches("match step_kind {").count(), 1);
    assert_eq!(consumer.matches("self.emit_object_read(").count(), 2);
    assert_eq!(
        consumer.matches("self.strings.payload(\"value\")").count(),
        1
    );
    for forbidden in [
        "matches!(step_kind",
        "step_kind ==",
        "step_kind !=",
        "_ =>",
        "unreachable!",
    ] {
        assert!(!consumer.contains(forbidden), "found `{forbidden}`");
    }

    let else_offset = consumer
        .find("function.instruction(&Instruction::Else);")
        .expect("missing completed-iterator else arm");
    let match_offset = consumer
        .find("match step_kind {")
        .expect("missing exhaustive step-kind match");
    let close_offset = consumer
        .rfind("self.pop_control(ControlFrameKind::If);")
        .expect("missing completed-iterator conditional close");
    assert!(else_offset < match_offset);
    assert!(match_offset < close_offset);
}

#[test]
fn contract_and_existing_cli_witnesses_pin_both_step_kinds() {
    assert!(CONTRACT.contains("DestructuringIteratorStepKind"));
    assert!(CONTRACT
        .contains("cargo test -p lila-aot-wasm --test destructuring_iterator_step_kind_structure"));
    assert!(TASK.contains("DestructuringIteratorStepKind"));
    for test_name in [
        "fn run_wasm_backend_uses_iterators_for_array_destructuring()",
        "fn run_wasm_backend_preserves_array_destructuring_iterator_abrupt_completions()",
    ] {
        assert!(CLI_ARRAY_TESTS.contains(test_name), "missing `{test_name}`");
    }
    for marker in ["[,] = elisionIterable;", "elisionValueGets === 0"] {
        assert!(ABRUPT_FIXTURE.contains(marker), "missing `{marker}`");
    }
    assert!(ITERATOR_FIXTURE.contains("var [defaulted = throwOriginalError()] = abruptIterable;"));
    assert!(ITERATOR_FIXTURE.contains("[...restTarget[restKey]] = [14, 15];"));
}
