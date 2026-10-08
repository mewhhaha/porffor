use std::fs;
use std::path::Path;

const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const ARRAY_DESTRUCTURING_SOURCE: &str = include_str!("../src/control_flow/array_destructuring.rs");
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
    // The native record owns policy and lifetime; value observation is a
    // borrowed output projection, so elision cannot acquire a value read.
    let step = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_sync_iterator_step_into(",
        "    fn prepare_destructuring_target<'b>(",
    );
    let signature = bounded(step, "&mut self,", ") -> Result<(), EmitError> {");
    assert!(signature.contains("iterator: &OwnedSyncIterator,"));
    assert!(signature.contains("value: Option<&ValueLocals>,"));
    assert!(!signature.contains("read_value: bool"));
    assert_eq!(step.matches("if let Some(output) = value {").count(), 1);
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for retired in [
        "DestructuringIteratorStepKind",
        "emit_destructuring_iterator_step(",
    ] {
        assert_eq!(count_in_rust_sources(&source_root, retired), 0);
    }
    assert_eq!(
        count_in_rust_sources(&source_root, "fn emit_sync_iterator_step_into("),
        1
    );
}

#[test]
fn exactly_three_array_element_producers_select_their_step_kind() {
    let producer = bounded(
        ARRAY_DESTRUCTURING_SOURCE,
        "    fn compile_array_destructuring_element(",
        "    fn emit_array_destructuring_rest_array(",
    );
    let elision = bounded(
        producer,
        "ArrayDestructuringElementIr::Elision => {",
        "ArrayDestructuringElementIr::Target { target, default } => {",
    );
    assert_eq!(
        elision
            .matches("self.emit_sync_iterator_step_without_value(")
            .count(),
        1
    );
    assert!(!elision.contains("emit_sync_iterator_step_value("));
    assert!(!elision.contains("prepare_destructuring_target"));

    let target = bounded(
        producer,
        "ArrayDestructuringElementIr::Target { target, default } => {",
        "ArrayDestructuringElementIr::Rest { target } => {",
    );
    positions_in_order(
        target,
        &[
            "self.prepare_destructuring_target(target, function)?;",
            "self.emit_sync_iterator_step_value(iterator, done, value, function)?;",
            "if let Some(default) = default {",
            "self.compile_expr_to_value(default, value, function)?;",
            "self.put_destructuring_target(prepared, value, function)?;",
        ],
    );
    assert!(!target.contains("emit_sync_iterator_step_without_value"));

    let rest = bounded(
        producer,
        "ArrayDestructuringElementIr::Rest { target } => {",
        "\n        }\n        Ok(())",
    );
    positions_in_order(
        rest,
        &[
            "self.prepare_destructuring_target(target, function)?;",
            "self.emit_array_destructuring_rest_array(iterator, done, value, function)?;",
            "self.put_destructuring_target(prepared, value, function)?;",
        ],
    );
    let rest_body = bounded(
        ARRAY_DESTRUCTURING_SOURCE,
        "    fn emit_array_destructuring_rest_array(",
        "\n    }\n}",
    );
    positions_in_order(
        rest_body,
        &[
            "self.emit_sync_iterator_step_value(iterator, done, value, function)?;",
            "self.emit_branch_if_to_target(exit, function);",
            "list.append(value, schema, function);",
            "self.emit_array_from_argument_list(&arguments, function)?;",
        ],
    );
    assert!(!rest_body.contains("emit_sync_iterator_step_without_value"));
    let retained = bounded(
        ARRAY_DESTRUCTURING_SOURCE,
        "match operation.use_view() {",
        "if let Some(binding) = operation.result_binding()",
    );
    for mapping in [
        "ArrayDestructuringOperationView::StepValue(_) =>",
        "ArrayDestructuringOperationView::Elision(_) =>",
        "ArrayDestructuringOperationView::RestArray(_) =>",
        "self.emit_sync_iterator_step_value(&iterator, done, &value, function)?;",
        "self.emit_sync_iterator_step_without_value(&iterator, done, function)?;",
        "self.emit_array_destructuring_rest_array(&iterator, done, &value, function)?;",
    ] {
        assert_eq!(
            retained.matches(mapping).count(),
            1,
            "missing shared operation {mapping}"
        );
    }
    assert!(!retained.contains("_ =>"));
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "fn compile_array_destructuring_element("),
        1
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "fn emit_array_destructuring_rest_array("),
        1
    );
}

#[test]
fn step_protocol_failures_use_the_typed_authority_after_marking_done() {
    let consumer = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_sync_iterator_step_into(",
        "    fn prepare_destructuring_target<'b>(",
    );
    assert_eq!(
        consumer
            .matches("self.emit_sync_iterator_protocol_type_error(")
            .count(),
        2
    );
    for error in [
        "SyncIteratorProtocolError::NextNotCallable",
        "SyncIteratorProtocolError::NextResultNotObject",
    ] {
        assert_eq!(consumer.matches(error).count(), 1);
    }
    assert!(!consumer.contains("emit_throw_runtime_error("));
    assert!(!consumer.contains("emit_throw_current_function_realm_type_error("));
    assert!(!consumer.contains("emit_iterator_close"));
    positions_in_order(
        consumer,
        &[
            "self.emit_is_callable_i32(&next, function)?;",
            "SyncIteratorProtocolError::NextNotCallable",
            "self.emit_function_or_proxy_call_with_argv(",
            "SyncIteratorProtocolError::NextResultNotObject",
            "self.emit_object_read(&next_result, &next_result, &done_key, &pending, function)?;",
            "if let Some(output) = value {",
            "self.emit_object_read(&next_result, &next_result, &value_key, &pending, function)?;",
            "// IteratorStepValue marks the record done",
            "pending.kind().load(function);",
            "Instruction::I32Const(CompletionKind::Throw.code() as i32)",
            "Instruction::I32Const(1)",
            "done.store(function);",
            "row.field(IteratorRecordSchema::DONE).write(",
            "result.copy_from(&pending, function);",
        ],
    );
    let publication = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_sync_iterator_step(",
        "    fn emit_sync_iterator_step_into(",
    );
    positions_in_order(
        publication,
        &[
            "self.emit_sync_iterator_step_into(iterator, done, value, &result, function)?;",
            "self.completion().copy_from(&result, function);",
            "result.clear(function);",
            "self.emit_propagate_current_throw_if_needed(function);",
        ],
    );
}

#[test]
fn step_kind_exhaustively_owns_the_iterator_value_read() {
    let value = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_sync_iterator_step_value(",
        "    pub(crate) fn emit_sync_iterator_step_without_value(",
    );
    assert!(value.contains("value.set_undefined(function);"));
    assert!(value.contains("self.emit_sync_iterator_step(iterator, done, Some(value), function)"));
    let elision = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_sync_iterator_step_without_value(",
        "    /// Native helpers must finalize their own state",
    );
    assert!(elision.contains("self.emit_sync_iterator_step(iterator, done, None, function)"));
    assert!(!elision.contains("ValueLocals"));
    let consumer = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_sync_iterator_step_into(",
        "    fn prepare_destructuring_target<'b>(",
    );
    let projection = bounded(
        consumer,
        "        if let Some(output) = value {",
        "        self.pop_control(ControlFrameKind::Block);",
    );
    let expected_projection = r#"
        self.emit_object_read(&next_result, &next_result, &value_key, &pending, function)?;
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.emit_branch_if_to_target(finish, function);
        output.copy_from(pending.value(), function);
        }
    "#;
    assert_eq!(normalized(projection), normalized(expected_projection));
    assert_eq!(consumer.matches("self.emit_object_read(").count(), 2);
    assert_eq!(consumer.matches("&value_key, &pending").count(), 1);
    positions_in_order(
        consumer,
        &[
            "row.field(IteratorRecordSchema::DONE)",
            "self.emit_branch_if_to_target(finish, function);",
            "self.emit_object_read(&next_result, &next_result, &done_key, &pending, function)?;",
            "self.compile_truthy_tagged_i32(pending.value(), function)?;",
            "done.store(function);",
            "self.emit_branch_if_to_target(finish, function);",
            "if let Some(output) = value {",
        ],
    );
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
