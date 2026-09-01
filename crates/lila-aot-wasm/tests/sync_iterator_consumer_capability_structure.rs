use std::fs;
use std::path::Path;

const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE: &str =
    include_str!("../src/control_flow/async_function_for_of_iterator.rs");
const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const MATH_SOURCE: &str = include_str!("../src/builtins/math.rs");
const ARRAY_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const ARRAY_ACCUMULATION_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_accumulation_iterator_errors.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/sync-iterator-consumer-capability.md");
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
        "/// Whether `Iterator.prototype.flatMap`",
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
            "ForOf,",
            "MathSumPrecise,",
        ]
    );
    let declaration_offset = CONTROL_FLOW_SOURCE
        .find("pub(crate) enum SyncIteratorConsumer {")
        .expect("sync iterator consumer declaration");
    assert_eq!(
        CONTROL_FLOW_SOURCE[..declaration_offset]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("}")
    );
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
        assert!(
            !CONTROL_FLOW_SOURCE.contains(&format!("impl {capability} for SyncIteratorConsumer"))
        );
    }
}

#[test]
fn shared_iterator_operations_borrow_consumer_and_project_the_error_realm() {
    let acquisition = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_get_iterator_from_value_locals(",
        "    /// The half of GetIterator after the `@@iterator` method has been loaded:",
    );
    assert!(acquisition.contains("consumer: &SyncIteratorConsumer,"));
    assert!(!acquisition.contains("consumer: SyncIteratorConsumer,"));
    assert_eq!(
        acquisition
            .matches("self.emit_value_to_current_function_realm_object_locals(")
            .count(),
        1
    );
    assert_eq!(
        acquisition
            .matches("self.emit_value_to_object_locals(")
            .count(),
        0
    );
    assert!(!acquisition.contains("consumer.clone()"));
    assert!(!acquisition.contains("match consumer"));

    let completion = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn finish_get_iterator_from_method(",
        "    fn emit_sync_iterator_protocol_type_error(",
    );
    assert!(completion.contains("consumer: &SyncIteratorConsumer,"));
    assert_eq!(
        completion
            .matches("self.emit_sync_iterator_protocol_type_error(")
            .count(),
        2
    );
    assert!(!completion.contains("consumer.clone()"));

    let step = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_sync_iterator_step_value(",
        "    fn prepare_destructuring_target<'b>(",
    );
    assert!(step.contains("consumer: &SyncIteratorConsumer,"));
    assert_eq!(
        step.matches("self.emit_sync_iterator_protocol_type_error(")
            .count(),
        2
    );
    assert!(!step.contains("consumer.clone()"));

    let projection = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_sync_iterator_protocol_type_error(",
        "    fn compile_array_destructuring_element(",
    );
    assert!(projection.contains("consumer: &SyncIteratorConsumer,"));
    assert_eq!(projection.matches("match (consumer, error) {").count(), 1);
    assert_eq!(
        projection
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );
    assert_eq!(projection.matches("emit_throw_runtime_error(").count(), 1);
    assert_eq!(
        projection
            .matches("match self.numeric_error_realm_source()")
            .count(),
        1
    );
    for source in [
        "NumericErrorRealmSource::StandardBuiltinEnvironment",
        "NumericErrorRealmSource::GlobalFallback",
        "NumericErrorRealmSource::NumericConversionHelperArgument",
    ] {
        assert_eq!(
            projection.matches(source).count(),
            1,
            "Realm source {source}"
        );
    }
    assert!(!projection.contains("_ =>"));
}

#[test]
fn each_shared_semantic_owner_constructs_one_consumer_and_borrows_it_for_the_full_walk() {
    let destructuring = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn compile_array_destructure_from_value_locals(",
        "    /// Reserves the common GetIterator/IteratorStep/IteratorValue working set.",
    );
    assert_eq!(
        destructuring
            .matches("let consumer = SyncIteratorConsumer::ArrayDestructuring;")
            .count(),
        1
    );
    assert_eq!(destructuring.matches("&consumer").count(), 2);

    assert_eq!(
        ARRAY_SOURCE
            .matches("let consumer = SyncIteratorConsumer::ArrayAccumulation;")
            .count(),
        1
    );
    assert_eq!(ARRAY_SOURCE.matches("&consumer").count(), 2);

    assert_eq!(
        MATH_SOURCE
            .matches("let consumer = SyncIteratorConsumer::MathSumPrecise;")
            .count(),
        1
    );
    assert_eq!(MATH_SOURCE.matches("&consumer").count(), 2);

    assert_eq!(
        ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE
            .matches("let consumer = SyncIteratorConsumer::ForOf;")
            .count(),
        1
    );
    assert_eq!(
        ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE
            .matches("&consumer")
            .count(),
        2
    );
}

#[test]
fn consumer_routes_and_runtime_witness_are_a_closed_census() {
    assert_eq!(
        CONTROL_FLOW_SOURCE.matches("SyncIteratorConsumer").count(),
        26
    );
    assert_eq!(
        ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE
            .matches("SyncIteratorConsumer")
            .count(),
        1
    );
    assert_eq!(ARRAY_SOURCE.matches("SyncIteratorConsumer").count(), 2);
    assert_eq!(MATH_SOURCE.matches("SyncIteratorConsumer").count(), 2);
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "SyncIteratorConsumer"),
        31
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "let consumer = SyncIteratorConsumer::"),
        6
    );
    assert_eq!(count_in_rust_sources(&source_root, "&consumer"), 18);
    for retired in [
        "SyncIteratorErrorPolicy",
        "LegacyMainRealm",
        "ForOfCurrentRealm",
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

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("SyncIteratorConsumer"));
        assert!(evidence.contains("borrow"));
    }
    assert!(CONTRACT.contains("has no `Clone`, `Copy`"));
    assert!(CONTRACT.contains("16 diagnostic rows"));
    assert!(TASK.contains("capability-free"));
    assert!(TASK.contains("sync-iterator-consumer-capability.md"));
}
