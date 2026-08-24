const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/iterator.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_iterator_helper_prototype_dispatch_matrix.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn exact_identifier_count(source: &str, identifier: &str) -> usize {
    source
        .match_indices(identifier)
        .filter(|(offset, _)| {
            source[*offset + identifier.len()..]
                .chars()
                .next()
                .is_none_or(|next| !next.is_ascii_alphanumeric() && next != '_')
        })
        .count()
}

#[test]
fn operation_domain_is_private_and_has_one_exhaustive_consumer() {
    assert_eq!(
        STANDARD_SOURCE
            .lines()
            .filter(|line| line.trim() == "enum IteratorHelperPrototypeOperation {")
            .count(),
        1,
        "the operation domain must have one private definition"
    );
    assert!(!STANDARD_SOURCE.contains("pub enum IteratorHelperPrototypeOperation"));
    assert!(!STANDARD_SOURCE.contains("pub(crate) enum IteratorHelperPrototypeOperation"));
    assert!(!STANDARD_SOURCE.contains("pub(super) enum IteratorHelperPrototypeOperation"));

    let variants = bounded(
        STANDARD_SOURCE,
        "enum IteratorHelperPrototypeOperation {",
        "}",
    );
    assert_eq!(
        without_whitespace(variants),
        "Next,Return,",
        "the shared helper prototype has exactly the next and return operations"
    );

    let signature = bounded(
        STANDARD_SOURCE,
        "fn emit_iterator_helper_dispatch(",
        ") -> Result<(), EmitError> {",
    );
    assert_eq!(
        without_whitespace(signature),
        without_whitespace(
            r#"
                &mut self,
                this_payload_local: u32,
                this_tag_local: u32,
                operation: IteratorHelperPrototypeOperation,
                function: &mut Function,
            "#,
        ),
        "the dispatcher must consume the closed operation directly"
    );
    assert!(!signature.contains(": bool"));
    assert!(!signature.contains("is_return"));

    let dispatch = bounded(
        STANDARD_SOURCE,
        "fn emit_iterator_helper_dispatch(",
        "#[allow(clippy::too_many_arguments)]\n    fn emit_iterator_zip_close_all_preserving_current_throw(",
    );
    const OPERATION_PROJECTION: &str = r#"
        let builtin = match operation {
            IteratorHelperPrototypeOperation::Next => next_builtin,
            IteratorHelperPrototypeOperation::Return => return_builtin,
        };
    "#;
    let dispatch = without_whitespace(dispatch);
    assert_eq!(dispatch.matches("matchoperation{").count(), 1);
    assert_eq!(
        dispatch
            .matches(without_whitespace(OPERATION_PROJECTION).as_str())
            .count(),
        1,
        "the operation must have one exhaustive target projection"
    );
    assert_eq!(
        dispatch
            .matches("IteratorHelperPrototypeOperation::")
            .count(),
        2
    );
    assert!(!dispatch.contains("_=>"));
    assert!(!dispatch.contains("ifoperation"));
    assert!(!dispatch.contains("matches!(operation"));
}

#[test]
fn shared_prototype_builtins_are_the_only_operation_producers() {
    assert_eq!(
        STANDARD_SOURCE
            .matches("self.emit_iterator_helper_dispatch(")
            .count(),
        2,
        "next and return must be the only dispatcher callers"
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("IteratorHelperPrototypeOperation::Next")
            .count(),
        2,
        "Next must occur once at its producer and once at the consumer"
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("IteratorHelperPrototypeOperation::Return")
            .count(),
        2,
        "Return must occur once at its producer and once at the consumer"
    );

    let next = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::IteratorHelperNext => {",
        "StandardBuiltinId::IteratorHelperReturn => {",
    );
    let returning = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::IteratorHelperReturn => {",
        "StandardBuiltinId::IteratorPrototypeToArray => {",
    );
    for (body, operation) in [(next, "Next"), (returning, "Return")] {
        assert_eq!(
            body.matches("self.emit_iterator_helper_dispatch(").count(),
            1
        );
        assert_eq!(
            body.matches("IteratorHelperPrototypeOperation::").count(),
            1
        );
        assert_eq!(
            body.matches(&format!("IteratorHelperPrototypeOperation::{operation}"))
                .count(),
            1,
            "%IteratorHelperPrototype%.{operation} must select its named operation"
        );
    }
}

#[test]
fn dispatch_rows_cover_seven_brands_and_eight_creation_surfaces() {
    let dispatch = bounded(
        STANDARD_SOURCE,
        "fn emit_iterator_helper_dispatch(",
        "#[allow(clippy::too_many_arguments)]\n    fn emit_iterator_zip_close_all_preserving_current_throw(",
    );
    let rows = bounded(
        dispatch,
        "for (brand, creator_builtin, next_builtin, return_builtin) in [",
        "] {",
    );
    const EXPECTED_ROWS: &str = r#"
        (
            OBJECT_INTERNAL_BRAND_ITERATOR_CONCAT_HELPER,
            StandardBuiltinId::IteratorConcat,
            StandardBuiltinId::IteratorConcatNext,
            StandardBuiltinId::IteratorConcatReturn,
        ),
        (
            OBJECT_INTERNAL_BRAND_ITERATOR_ZIP_HELPER,
            StandardBuiltinId::IteratorZip,
            StandardBuiltinId::IteratorZipNext,
            StandardBuiltinId::IteratorZipReturn,
        ),
        (
            OBJECT_INTERNAL_BRAND_ITERATOR_MAP_HELPER,
            StandardBuiltinId::IteratorPrototypeMap,
            StandardBuiltinId::IteratorMapNext,
            StandardBuiltinId::IteratorMapReturn,
        ),
        (
            OBJECT_INTERNAL_BRAND_ITERATOR_FILTER_HELPER,
            StandardBuiltinId::IteratorPrototypeFilter,
            StandardBuiltinId::IteratorFilterNext,
            StandardBuiltinId::IteratorFilterReturn,
        ),
        (
            OBJECT_INTERNAL_BRAND_ITERATOR_FLAT_MAP_HELPER,
            StandardBuiltinId::IteratorPrototypeFlatMap,
            StandardBuiltinId::IteratorFlatMapNext,
            StandardBuiltinId::IteratorFlatMapReturn,
        ),
        (
            OBJECT_INTERNAL_BRAND_ITERATOR_TAKE_HELPER,
            StandardBuiltinId::IteratorPrototypeTake,
            StandardBuiltinId::IteratorTakeNext,
            StandardBuiltinId::IteratorTakeReturn,
        ),
        (
            OBJECT_INTERNAL_BRAND_ITERATOR_DROP_HELPER,
            StandardBuiltinId::IteratorPrototypeDrop,
            StandardBuiltinId::IteratorDropNext,
            StandardBuiltinId::IteratorDropReturn,
        ),
    "#;
    assert_eq!(
        without_whitespace(rows),
        without_whitespace(EXPECTED_ROWS),
        "every helper brand must retain its exact creator, next and return targets"
    );

    let initialization = bounded(dispatch, "let creator_is_initialized =", ";");
    const EXPECTED_INITIALIZATION: &str = r#"
        self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(creator_builtin)
            || (brand == OBJECT_INTERNAL_BRAND_ITERATOR_ZIP_HELPER
                && self
                    .runtime_bootstrap_plan
                    .should_initialize_standard_builtin(StandardBuiltinId::IteratorZipKeyed))
    "#;
    assert_eq!(
        without_whitespace(initialization),
        without_whitespace(EXPECTED_INITIALIZATION),
        "zipKeyed must initialize the shared zip-brand dispatch row"
    );

    for creator in [
        "StandardBuiltinId::IteratorConcat",
        "StandardBuiltinId::IteratorZip",
        "StandardBuiltinId::IteratorZipKeyed",
        "StandardBuiltinId::IteratorPrototypeMap",
        "StandardBuiltinId::IteratorPrototypeFilter",
        "StandardBuiltinId::IteratorPrototypeFlatMap",
        "StandardBuiltinId::IteratorPrototypeTake",
        "StandardBuiltinId::IteratorPrototypeDrop",
    ] {
        assert_eq!(
            exact_identifier_count(dispatch, creator),
            1,
            "creation surface {creator} must have one dispatch owner"
        );
    }
}

#[test]
fn dispatch_matrix_fixture_has_one_active_registration_and_eight_surfaces() {
    const REGISTRATION_START: &str =
        "#[test]\nfn run_wasm_backend_dispatches_borrowed_iterator_helper_methods_for_all_families() {";
    assert_eq!(
        CLI_TESTS.matches(REGISTRATION_START).count(),
        1,
        "the dispatch matrix must have one active CLI test registration"
    );
    let registration_offset = CLI_TESTS
        .find(REGISTRATION_START)
        .expect("the active dispatch matrix CLI registration");
    let preceding_attributes = CLI_TESTS[..registration_offset]
        .rsplit_once("\n}\n")
        .expect("the CLI test preceding the dispatch matrix registration")
        .1;
    for forbidden_attribute in ["#[ignore", "#[cfg"] {
        assert!(
            !preceding_attributes.contains(forbidden_attribute),
            "the dispatch matrix CLI test must not have {forbidden_attribute} attached"
        );
    }
    let registration = bounded(
        CLI_TESTS,
        REGISTRATION_START,
        "#[test]\nfn run_wasm_backend_succeeds_for_iterator_from_wrapper_return_invalid_this_fixture() {",
    );
    assert!(!registration.contains("#[ignore"));
    assert_eq!(
        CLI_TESTS
            .matches("wasm_iterator_helper_prototype_dispatch_matrix.js")
            .count(),
        1,
        "the dispatch matrix fixture must have one CLI path"
    );
    for required in [
        ".arg(\"run\")",
        ".arg(\"--execution-backend\")",
        ".arg(\"wasm\")",
        "wasm_iterator_helper_prototype_dispatch_matrix.js",
        "output.status.success()",
        "backend_used: WasmAot",
        "boolean(true)",
    ] {
        assert_eq!(
            registration.matches(required).count(),
            1,
            "the active dispatch matrix registration must retain one {required}"
        );
    }

    let fixture = without_whitespace(CLI_FIXTURE);
    for (label, snippet) in [
        (
            "fail-loud assertion boundary",
            r#"
                function assertSameValue(actual, expected, label) {
                  if (actual !== expected) throw label;
                }
            "#,
        ),
        (
            "borrowed next acquisition",
            "var next = helperPrototype.next;",
        ),
        (
            "borrowed return acquisition",
            "var returnMethod = helperPrototype.return;",
        ),
        (
            "borrowed next call",
            "var nextResult = next.call(nextHelper);",
        ),
        (
            "borrowed return call",
            "var returnResult = returnMethod.call(returnHelper);",
        ),
    ] {
        let snippet = without_whitespace(snippet);
        assert_eq!(
            fixture.matches(snippet.as_str()).count(),
            1,
            "the fixture must retain one {label}"
        );
    }
    for acquisition in ["helperPrototype.next", "helperPrototype.return"] {
        assert_eq!(
            fixture.matches(acquisition).count(),
            1,
            "the fixture must acquire {acquisition} exactly once"
        );
    }

    let surfaces = [
        (
            "concat",
            r#"check("concat", Iterator.concat([1]), Iterator.concat([1]), 1, identity);"#,
        ),
        (
            "zip",
            r#"check("zip", Iterator.zip([[2]]), Iterator.zip([[2]]), 2, first);"#,
        ),
        (
            "zipKeyed",
            r#"
                check(
                  "zipKeyed",
                  Iterator.zipKeyed({ entry: [8] }),
                  Iterator.zipKeyed({ entry: [8] }),
                  8,
                  keyedValue
                );
            "#,
        ),
        (
            "map",
            r#"check("map", Iterator.from([3]).map(identity), Iterator.from([3]).map(identity), 3, identity);"#,
        ),
        (
            "filter",
            r#"check("filter", Iterator.from([4]).filter(keep), Iterator.from([4]).filter(keep), 4, identity);"#,
        ),
        (
            "flatMap",
            r#"
                check(
                  "flatMap",
                  Iterator.from([5]).flatMap(singleton),
                  Iterator.from([5]).flatMap(singleton),
                  5,
                  identity
                );
            "#,
        ),
        (
            "take",
            r#"check("take", Iterator.from([6]).take(1), Iterator.from([6]).take(1), 6, identity);"#,
        ),
        (
            "drop",
            r#"check("drop", Iterator.from([7]).drop(0), Iterator.from([7]).drop(0), 7, identity);"#,
        ),
    ];
    assert_eq!(
        CLI_FIXTURE
            .lines()
            .filter(|line| line.trim_start().starts_with("check("))
            .count(),
        surfaces.len(),
        "the seven helper brands plus the zipKeyed alias must have eight checks"
    );
    for (surface, call) in surfaces {
        let call = without_whitespace(call);
        assert_eq!(
            fixture.matches(call.as_str()).count(),
            1,
            "the fixture must retain one exact {surface} surface check"
        );
    }

    assert_eq!(
        CLI_FIXTURE
            .lines()
            .filter(|line| line.trim() == "true;")
            .count(),
        1,
        "the fixture must have one success publication"
    );
    assert_eq!(
        CLI_FIXTURE.lines().next_back().map(str::trim),
        Some("true;"),
        "the unique success publication must terminate the fixture"
    );
}
