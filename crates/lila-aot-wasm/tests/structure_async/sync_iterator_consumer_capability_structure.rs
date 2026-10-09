use std::fs;
use std::path::Path;

const CONTROL_FLOW_SOURCE: &str = include_str!("../../src/control_flow.rs");
const RESUMABLE_SYNC_FOR_OF_ITERATOR_SOURCE: &str =
    include_str!("../../src/control_flow/resumable_sync_for_of_iterator.rs");
const ARRAY_SOURCE: &str = include_str!("../../src/builtins/array.rs");
const MATH_SOURCE: &str = include_str!("../../src/builtins/math/sum_precise.rs");
const AGGREGATE_SOURCE: &str =
    include_str!("../../src/builtins/errors/aggregate_error_preparation.rs");
const RUNTIME_ERROR_SOURCE: &str = include_str!("../../src/builtins/errors/runtime_error.rs");
const ARRAY_DESTRUCTURING_SOURCE: &str =
    include_str!("../../src/control_flow/array_destructuring.rs");
const RESUMABLE_ARRAY_SOURCE: &str =
    include_str!("../../src/control_flow/resumable_array_destructuring.rs");
const LIST_FORMAT_SOURCE: &str = include_str!("../../src/builtins/intl_listformat/iterable.rs");
const LIST_FORMAT_ENGINE_TESTS: &str =
    include_str!("../../../lila-engine/tests/aot_intl/aot_intl_list_format.rs");
const LIST_FORMAT_ABRUPT_FIXTURE: &str = include_str!(
    "../../../lila-engine/tests/fixtures/intl_list_format/iterator_abrupts_do_not_close.js"
);
const LIST_FORMAT_CLOSE_FIXTURE: &str = include_str!(
    "../../../lila-engine/tests/fixtures/intl_list_format/nonstring_close_precedence.js"
);
const ARRAY_CLI_TESTS: &str = include_str!("../../../lila-cli/tests/cli/array.rs");
const ARRAY_ACCUMULATION_FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_array_accumulation_iterator_errors.js");
const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/sync-iterator-consumer-capability.md");
const TASK: &str = include_str!("../../../../tasks/15-generators-iterators-resource-management.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn count_in_rust_sources(root: &Path, needle: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
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
fn sync_iterator_consumer_is_the_exact_capability_free_domain() {
    let declaration = bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) enum SyncIteratorConsumer {",
        "enum SyncIteratorProtocolError {",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && *line != "}")
            .collect::<Vec<_>>(),
        [
            "ArrayDestructuring,",
            "ArrayAccumulation,",
            "ArrayFrom,",
            "ForOf,",
            "MathSumPrecise,",
            "ListFormat,",
            "AggregateError,",
            "MapConstructor,",
            "SetConstructor,",
            "ObjectFromEntries,",
            "MapGroupBy,",
            "ObjectGroupBy,",
            "SetLike,",
            "IteratorHelper,"
        ]
    );
    // Copying a diagnostic policy cannot copy the native iterator release owner.
    assert!(CONTROL_FLOW_SOURCE
        .contains("#[derive(Clone, Copy)]\npub(crate) enum SyncIteratorConsumer {"));
    let owner = bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) struct OwnedSyncIterator {",
        "impl OwnedSyncIterator {",
    );
    assert!(owner.contains("record: GcLocal<IteratorRecord>,"));
    assert!(owner.contains("consumer: SyncIteratorConsumer,"));
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert!(!CONTROL_FLOW_SOURCE.contains(&format!("impl {capability} for OwnedSyncIterator")));
    }
    let declaration_offset = CONTROL_FLOW_SOURCE
        .find("pub(crate) struct OwnedSyncIterator {")
        .unwrap();
    assert!(CONTROL_FLOW_SOURCE[..declaration_offset]
        .trim_end()
        .ends_with(
        "#[must_use = \"a completed iterator record must be cleared or captured by its consumer\"]"
    ));
}

#[test]
fn shared_iterator_operations_borrow_consumer_and_project_the_error_realm() {
    let acquisition = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_get_sync_iterator(",
        "    /// GetIteratorDirect after a keys()/iterator method Call:",
    );
    assert!(acquisition.contains("consumer: SyncIteratorConsumer,"));
    assert!(acquisition.contains("source: &ValueLocals,"));
    assert!(acquisition.contains("self.emit_value_to_object_locals(source, &boxed, function)?;"));
    assert!(acquisition
        .contains("self.emit_object_read(boxed.value(), source, &key, &method, function)?;"));
    assert_eq!(
        acquisition
            .matches("self.emit_sync_iterator_protocol_type_error(")
            .count(),
        1
    );
    assert!(acquisition.contains("&consumer,"));
    assert!(!acquisition.contains("consumer.clone()"));
    assert!(!acquisition.contains("match consumer"));
    let completion = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_get_sync_iterator_direct(",
        "    fn emit_sync_iterator_protocol_type_error(",
    );
    assert!(completion.contains("consumer: SyncIteratorConsumer,"));
    assert!(completion.contains("SyncIteratorProtocolError::MethodResultNotObject"));
    assert!(completion.contains("Ok(OwnedSyncIterator { record, consumer })"));
    assert!(completion.contains(".struct_type::<IteratorRecord>().construct("));
    assert!(!completion.contains("consumer.clone()"));

    let step = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_sync_iterator_step_into(",
        "    fn prepare_destructuring_target<'b>(",
    );
    assert!(step.contains("iterator: &OwnedSyncIterator,"));
    assert_eq!(step.matches("&iterator.consumer,").count(), 2);
    assert!(!step.contains("consumer.clone()"));
    let projection = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_sync_iterator_protocol_type_error(",
        "    /// Runs one sync IteratorStep/IteratorValue pair",
    );
    assert!(projection.contains("consumer: &SyncIteratorConsumer,"));
    assert_eq!(projection.matches("match (consumer, error) {").count(), 1);
    assert!(!projection.contains("_ =>"));
    assert_eq!(
        projection.matches("self.emit_throw_runtime_error(").count(),
        1
    );
    assert!(projection.contains("lila_ir::NativeErrorKind::TypeError"));
    assert!(projection.contains("result: &CompletionLocals,"));
    let error = bounded(
        RUNTIME_ERROR_SOURCE,
        "    pub(crate) fn emit_runtime_error_object(",
        "    fn emit_fresh_native_error_object(",
    );
    assert!(error.contains("let realm = self.emit_execution_realm(function);"));
    assert!(error.contains("self.emit_load_non_array_realm_intrinsic("));
    let realm = bounded(
        RUNTIME_ERROR_SOURCE,
        "    pub(crate) fn emit_execution_realm(",
        "    pub(crate) fn emit_runtime_error_object(",
    );
    for actual_source in [
        "FunctionContextSchema::REALM",
        "EnvironmentSchema::DEFINING_REALM",
        "EnvironmentSchema::PARENT",
        "self.load_current_realm(function)",
    ] {
        assert!(
            realm.contains(actual_source),
            "missing actual Realm authority {actual_source}"
        );
    }
}

#[test]
fn each_shared_semantic_owner_constructs_one_consumer_and_borrows_it_for_the_full_walk() {
    let destructuring = bounded(
        ARRAY_DESTRUCTURING_SOURCE,
        "    pub(crate) fn compile_array_destructure_from_value_locals(",
        "    fn compile_array_destructuring_element(",
    );
    assert_eq!(
        destructuring
            .matches("SyncIteratorConsumer::ArrayDestructuring,")
            .count(),
        1
    );
    assert!(destructuring.contains(
        "self.compile_array_destructuring_element(element, &iterator, done, &value, function)?;"
    ));
    assert_eq!(
        destructuring
            .matches("self.emit_sync_iterator_close(&iterator, &pending, &closed, function)?;")
            .count(),
        2
    );
    assert_eq!(
        destructuring.matches("iterator.clear(function);").count(),
        1
    );
    assert!(RESUMABLE_ARRAY_SOURCE.contains(
        "self.emit_publish_retained_array_iterator(&storage, acquired.record(), function);"
    ));
    assert!(RESUMABLE_ARRAY_SOURCE
        .contains("self.emit_sync_iterator_close(&iterator, &pending, &closed, function)?;"));

    let accumulation = bounded(
        ARRAY_SOURCE,
        "ArrayAccumulationElementIr::Spread(spread) => {",
        "        if let Some(cell) = cell {",
    );
    assert_eq!(
        accumulation
            .matches("SyncIteratorConsumer::ArrayAccumulation,")
            .count(),
        1
    );
    assert_eq!(
        accumulation
            .matches("self.emit_sync_iterator_step_value(&iterator, done, &value, f)?;")
            .count(),
        1
    );
    assert_eq!(accumulation.matches("iterator.clear(f);").count(), 1);
    assert!(!accumulation.contains("emit_sync_iterator_close"));

    assert_eq!(
        MATH_SOURCE
            .matches("SyncIteratorConsumer::MathSumPrecise,")
            .count(),
        1
    );
    assert_eq!(
        MATH_SOURCE
            .matches("self.emit_sync_iterator_step_value(&iterator, done, &value, function)?;")
            .count(),
        1
    );
    assert_eq!(MATH_SOURCE.matches("iterator.clear(function);").count(), 1);
    let list = bounded(
        LIST_FORMAT_SOURCE,
        "    fn emit_list_of_strings(",
        "    pub(crate) fn emit_intl_list_format(",
    );
    assert_eq!(list.matches("SyncIteratorConsumer::ListFormat,").count(), 1);
    assert_eq!(list.matches("self.emit_get_sync_iterator(").count(), 1);
    assert_eq!(
        list.matches("self.emit_sync_iterator_step_value(&iterator, done, &value, f)?;")
            .count(),
        1
    );
    assert!(list.contains("IteratorRecordSchema::ITERATOR"));
    assert!(list
        .contains("self.emit_iterator_close_with_completion(&receiver, &pending, &closed, f)?;"));
    assert!(list.contains("iterator.clear(f);"));

    let aggregate = bounded(
        AGGREGATE_SOURCE,
        "    pub(super) fn emit_aggregate_error_iterable_to_list(",
        "\n    }\n}",
    );
    assert_eq!(
        aggregate
            .matches("SyncIteratorConsumer::AggregateError,")
            .count(),
        1
    );
    assert_eq!(aggregate.matches("self.emit_get_sync_iterator(").count(), 1);
    assert_eq!(
        aggregate
            .matches("self.emit_sync_iterator_step_value(&iterator, done, &value, function)?;")
            .count(),
        1
    );
    assert!(aggregate.contains("iterator.clear(function);"));
    assert!(!aggregate.contains("emit_sync_iterator_close"));
    assert!(RESUMABLE_SYNC_FOR_OF_ITERATOR_SOURCE
        .contains("self.emit_get_sync_iterator(&source, SyncIteratorConsumer::ForOf, function)?;"));
    assert!(RESUMABLE_SYNC_FOR_OF_ITERATOR_SOURCE
        .contains("self.emit_sync_iterator_step_value(&iterator, done, &value, function)?;"));
    assert!(RESUMABLE_SYNC_FOR_OF_ITERATOR_SOURCE.contains("iterator.clear(function);"));
    for owner in [destructuring, accumulation, list, aggregate] {
        assert!(!owner.contains("consumer.clone()"));
        assert!(!owner.contains("match consumer"));
    }
}

#[test]
fn consumer_routes_and_runtime_witness_are_a_closed_census() {
    let projection = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_sync_iterator_protocol_type_error(",
        "    /// Runs one sync IteratorStep/IteratorValue pair",
    );
    for policy in [
        "ArrayDestructuring",
        "ArrayAccumulation",
        "ArrayFrom",
        "ForOf",
        "MathSumPrecise",
        "ListFormat",
        "AggregateError",
        "MapConstructor",
        "SetConstructor",
        "ObjectFromEntries",
        "MapGroupBy",
        "ObjectGroupBy",
        "SetLike",
        "IteratorHelper",
    ] {
        assert_eq!(
            projection
                .matches(&format!("SyncIteratorConsumer::{policy},"))
                .count(),
            4,
            "every policy owns the full four-error domain"
        );
    }
    let errors = bounded(
        CONTROL_FLOW_SOURCE,
        "enum SyncIteratorProtocolError {",
        "/// The activation layout shared by the two execution kinds",
    );
    assert_eq!(
        errors
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && *line != "}")
            .collect::<Vec<_>>(),
        [
            "NotIterable,",
            "MethodResultNotObject,",
            "NextNotCallable,",
            "NextResultNotObject,"
        ]
    );
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for owner in [
        "fn emit_get_sync_iterator(",
        "fn emit_get_sync_iterator_direct(",
        "fn emit_sync_iterator_step_into(",
        "fn emit_sync_iterator_protocol_type_error(",
    ] {
        assert_eq!(
            count_in_rust_sources(&source_root, owner),
            1,
            "sole physical iterator owner {owner}"
        );
    }
    for retired in [
        "SyncIteratorErrorPolicy",
        "LegacyMainRealm",
        "ForOfCurrentRealm",
        "SyncIteratorLocals",
        "DestructuringIteratorLocals",
    ] {
        assert_eq!(count_in_rust_sources(&source_root, retired), 0);
    }

    for marker in [
        "captureArraySpreadError",
        "array spread value is not iterable",
        "array spread iterator method must return object",
        "array spread iterator next must be callable",
        "array spread iterator next result must be object",
        "nonCallableNextClosed === 0",
        "primitiveNextResultClosed === 0",
        "doneErrorClosed === 0",
        "valueErrorClosed === 0",
        "originalStringIteratorDescriptor",
        "stringIteratorReceiver === \"ab\"",
    ] {
        assert!(
            ARRAY_ACCUMULATION_FIXTURE.contains(marker),
            "array accumulation fixture marker `{marker}`"
        );
    }
    assert!(ARRAY_CLI_TESTS
        .contains("fn run_wasm_backend_preserves_array_accumulation_iterator_errors()"));

    for marker in [
        "iterator.get",
        "iterator.call",
        "next.get",
        "next.call",
        "done.get",
        "value.get",
        "same(caught, marker",
        "same(closes, 0",
        "no return Get",
    ] {
        assert!(
            LIST_FORMAT_ABRUPT_FIXTURE.contains(marker),
            "ListFormat abrupt marker `{marker}`"
        );
    }
    for marker in [
        "initial TypeError wins",
        "close failure cannot replace initial throw",
        "each nonstring closes",
        "no coercion hooks",
    ] {
        assert!(
            LIST_FORMAT_CLOSE_FIXTURE.contains(marker),
            "ListFormat close marker `{marker}`"
        );
    }
    for test in [
        "iterator_abrupts_do_not_close",
        "nonstring_close_precedence",
    ] {
        assert!(LIST_FORMAT_ENGINE_TESTS.contains(&format!("fn {test}()")));
    }

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("SyncIteratorConsumer"));
        assert!(evidence.contains("borrow"));
    }
    // The retained contract records the earlier policy domain; the actual
    // native record and complete closed policy projection are checked above.
    assert!(CONTRACT.contains("ListFormat"));
    assert!(TASK.contains("capability-free"));
    assert!(TASK.contains("sync-iterator-consumer-capability.md"));
}
