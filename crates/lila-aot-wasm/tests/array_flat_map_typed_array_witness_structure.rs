const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_flat_map_resizable_typedarray.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start boundary: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end boundary after {start}: {end}"))
        .0
}

fn flat_map_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "pub(crate) fn compile_array_prototype_flat_map_builtin(",
        "fn emit_flat_map_append(",
    )
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn unique_position(source: &str, needle: &str, label: &str) -> usize {
    assert_eq!(
        source.matches(needle).count(),
        1,
        "{label} must occur exactly once"
    );
    source
        .find(needle)
        .unwrap_or_else(|| panic!("missing sentinel: {label}"))
}

fn unique_normalized_position(source: &str, snippet: &str, label: &str) -> usize {
    let snippet = without_whitespace(snippet);
    assert_eq!(
        source.matches(snippet.as_str()).count(),
        1,
        "{label} must occur exactly once"
    );
    source
        .find(snippet.as_str())
        .unwrap_or_else(|| panic!("missing normalized sentinel: {label}"))
}

#[test]
fn flat_map_uses_observable_length_and_shared_live_property_operations() {
    let body = flat_map_body();
    for (needle, expected) in [
        ("emit_array_iteration_length_before_callback_validation(", 1),
        ("emit_object_has_property_i32(", 2),
        ("emit_object_read(", 2),
        ("emit_typed_array_or_object_index_read_from_locals(", 1),
        ("emit_is_array_i64(", 1),
        ("emit_array_species_create(", 1),
    ] {
        assert_eq!(body.matches(needle).count(), expected, "{needle}");
    }
    for forbidden in [
        "TypedArrayViewLocals",
        "TypedArrayWitnessUse",
        "emit_load_typed_array_private_state(",
        "emit_typed_array_current_byte_length(",
        "emit_validate_typed_array_current_byte_length(",
        "emit_load_array_buffer_byte_length(",
        "emit_load_array_buffer_data(",
        "HEAP_TYPED_ARRAY_",
        "HEAP_LEN_OFFSET",
        "HEAP_OBJECT_BOXED_",
        "Instruction::I64TruncF64U",
        "Instruction::I64DivU",
    ] {
        assert!(
            !body.contains(forbidden),
            "flatMap must delegate property semantics, not use {forbidden}"
        );
    }
    assert_eq!(
        ARRAY_SOURCE
            .matches("emit_typed_array_current_byte_length(")
            .count(),
        0,
        "Array builtins must not reintroduce the legacy raw current-length observer"
    );
}

#[test]
fn flat_map_keeps_the_snapshot_presence_read_and_mapper_order() {
    let body = flat_map_body();
    let snapshot = unique_position(
        body,
        "emit_array_iteration_length_before_callback_validation(",
        "ToObject/LengthOfArrayLike",
    );
    let validate = unique_position(body, "emit_is_callable_i32(", "IsCallable");
    let target = unique_position(body, "emit_array_species_create(", "ArraySpeciesCreate");
    let first_loop = body
        .find("Instruction::Loop(BlockType::Empty)")
        .expect("source loop");
    let presence = body
        .find("emit_object_has_property_i32(")
        .expect("source HasProperty");
    let read = body
        .find("emit_typed_array_or_object_index_read_from_locals(")
        .expect("source Get");
    let mapper = unique_position(
        body,
        "emit_function_or_proxy_call_with_argv_leave_throw_completion(",
        "mapper Call",
    );
    let is_array = unique_position(body, "emit_is_array_i64(", "mapped IsArray");
    assert!(
        snapshot < validate
            && validate < target
            && target < first_loop
            && first_loop < presence
            && presence < read
            && read < mapper
            && mapper < is_array
    );
    let normalized = without_whitespace(body);
    for receiver in ["this", "mapped"] {
        let snippet = format!(
            "self.emit_object_has_property_i32({receiver}_payload_local,{receiver}_tag_local,key_local,present_local,function,)?;self.emit_return_current_completion_if_throw(function);"
        );
        assert!(
            normalized.contains(&snippet),
            "{receiver} HasProperty must propagate abrupt completion"
        );
    }
    let dispatcher = without_whitespace(STANDARD_SOURCE);
    unique_normalized_position(
        &dispatcher,
        r#"
        StandardBuiltinId::ArrayPrototypeFlatMap => {
            self.compile_array_prototype_flat_map_builtin(function)?;
        }
    "#,
        "Array.prototype.flatMap dispatcher edge",
    );
}

#[test]
fn focused_fixture_couples_each_buffer_transition_to_one_failure_bit() {
    let fixture = without_whitespace(CLI_FIXTURE);
    assert_eq!(
        fixture.matches("Array.prototype.flatMap.call(").count(),
        6,
        "fixture must exercise exactly six generic TypedArray calls"
    );
    assert_eq!(
        fixture.matches(".resize(").count(),
        5,
        "fixture must retain the odd, growth, shrink and fixed-view resize transitions"
    );
    assert_eq!(
        fixture.matches(".transfer()").count(),
        1,
        "fixture must detach exactly one backing buffer"
    );
    assert_eq!(
        fixture.matches("failures|=").count(),
        6,
        "fixture must retain one distinct failure publication per scenario"
    );

    let ordered = [
        (
            "odd-byte setup",
            r#"
            var oddBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
            var oddView = new Uint16Array(oddBuffer);
            oddView[0] = 11;
            oddView[1] = 22;
            "#,
        ),
        ("odd-byte resize", "oddBuffer.resize(5);"),
        (
            "odd-byte call",
            r#"
            var oddResult = Array.prototype.flatMap.call(oddView, function (value) {
              oddCalls += 1;
              return [value];
            });
            "#,
        ),
        (
            "odd-byte assertion",
            r#"
            if (oddCalls !== 2 || oddResult.length !== 2 || oddResult[0] !== 11 || oddResult[1] !== 22) {
              failures |= 1;
            }
            "#,
        ),
        (
            "growth setup",
            r#"
            var growBuffer = new ArrayBuffer(2, { maxByteLength: 6 });
            var growView = new Uint16Array(growBuffer);
            growView[0] = 31;
            "#,
        ),
        (
            "growth call and resize",
            r#"
            var growResult = Array.prototype.flatMap.call(growView, function (value, index) {
              growCalls += 1;
              if (index === 0) {
                growBuffer.resize(6);
                growView[1] = 32;
                growView[2] = 33;
              }
              return [value];
            });
            "#,
        ),
        (
            "growth assertion",
            r#"
            if (growCalls !== 1 || growResult.length !== 1 || growResult[0] !== 31) {
              failures |= 2;
            }
            "#,
        ),
        (
            "shrink setup",
            r#"
            var shrinkBuffer = new ArrayBuffer(6, { maxByteLength: 6 });
            var shrinkView = new Uint16Array(shrinkBuffer);
            shrinkView[0] = 41;
            shrinkView[1] = 42;
            shrinkView[2] = 43;
            "#,
        ),
        (
            "shrink call and resize",
            r#"
            var shrinkResult = Array.prototype.flatMap.call(shrinkView, function (value, index) {
              shrinkCalls += 1;
              if (index === 0) shrinkBuffer.resize(3);
              return [value];
            });
            "#,
        ),
        (
            "shrink assertion",
            r#"
            if (shrinkCalls !== 1 || shrinkResult.length !== 1 || shrinkResult[0] !== 41) {
              failures |= 4;
            }
            "#,
        ),
        (
            "fixed-view setup",
            r#"
            var fixedBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
            var fixedView = new Uint16Array(fixedBuffer, 2, 1);
            fixedView[0] = 51;
            "#,
        ),
        ("fixed-view out-of-bounds resize", "fixedBuffer.resize(1);"),
        (
            "fixed-view out-of-bounds call",
            r#"
            var fixedOutOfBoundsResult = Array.prototype.flatMap.call(fixedView, function (value) {
              fixedOutOfBoundsCalls += 1;
              return [value];
            });
            "#,
        ),
        (
            "fixed-view out-of-bounds assertion",
            r#"
            if (fixedOutOfBoundsCalls !== 0 || fixedOutOfBoundsResult.length !== 0) {
              failures |= 8;
            }
            "#,
        ),
        (
            "fixed-view regrowth",
            r#"
            fixedBuffer.resize(4);
            fixedView[0] = 52;
            "#,
        ),
        (
            "fixed-view regrown call",
            r#"
            var fixedRegrownResult = Array.prototype.flatMap.call(fixedView, function (value) {
              fixedRegrownCalls += 1;
              return [value];
            });
            "#,
        ),
        (
            "fixed-view regrown assertion",
            r#"
            if (fixedRegrownCalls !== 1 || fixedRegrownResult.length !== 1 || fixedRegrownResult[0] !== 52) {
              failures |= 16;
            }
            "#,
        ),
        (
            "detached setup",
            r#"
            var detachedBuffer = new ArrayBuffer(4);
            var detachedView = new Uint16Array(detachedBuffer);
            detachedView[0] = 61;
            "#,
        ),
        ("detach transition", "detachedBuffer.transfer();"),
        (
            "detached call",
            r#"
            var detachedResult = Array.prototype.flatMap.call(detachedView, function (value) {
              detachedCalls += 1;
              return [value];
            });
            "#,
        ),
        (
            "detached assertion",
            r#"
            if (detachedCalls !== 0 || detachedResult.length !== 0) {
              failures |= 32;
            }
            "#,
        ),
        ("final zero-failure publication", "failures === 0;"),
    ];

    let mut previous = None;
    for (label, snippet) in ordered {
        let position = unique_normalized_position(&fixture, snippet, label);
        if let Some(previous) = previous {
            assert!(previous < position, "fixture step out of order: {label}");
        }
        previous = Some(position);
    }

    let final_publication = without_whitespace("failures === 0;");
    assert!(
        fixture.ends_with(final_publication.as_str()),
        "the unique zero-failure publication must terminate the fixture"
    );

    let registration = bounded(
        CLI_TESTS,
        "fn run_wasm_backend_succeeds_for_supported_array_flat_map_resizable_typedarray_fixture()",
        "fn run_wasm_backend_succeeds_for_supported_array_flat_map_sparse_array_like_fixture()",
    );
    assert_eq!(
        CLI_TESTS
            .matches("wasm_array_flat_map_resizable_typedarray.js")
            .count(),
        1,
        "focused fixture must have exactly one CLI registration"
    );
    for required in [
        "Command::new(env!(\"CARGO_BIN_EXE_lila\"))",
        ".arg(\"--execution-backend\")",
        ".arg(\"wasm\")",
        "wasm_array_flat_map_resizable_typedarray.js",
        "output.status.success()",
        "backend_used: WasmAot",
        "boolean(true)",
    ] {
        assert!(
            registration.contains(required),
            "focused CLI registration must retain {required}"
        );
    }
}
