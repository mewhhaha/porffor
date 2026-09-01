use std::fs;
use std::path::{Path, PathBuf};

const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const BIGINT_SOURCE: &str = include_str!("../src/builtins/bigint.rs");
const TEMPORAL_SOURCE: &str = include_str!("../src/builtins/temporal.rs");
const TEMPORAL_INSTANT_SOURCE: &str = include_str!("../src/builtins/temporal_instant.rs");

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

fn assert_one_value_call_with_policy(body: &str, policy: &str, label: &str) {
    assert_eq!(
        body.matches("emit_value_to_bigint_locals(").count(),
        1,
        "{label} must have exactly one value-to-BigInt call"
    );

    let call = body
        .split_once("emit_value_to_bigint_locals(")
        .expect("caller count was checked")
        .1
        .split_once(")?;")
        .unwrap_or_else(|| panic!("{label} must retain the fallible helper call boundary"))
        .0;
    let normalized_call = without_whitespace(call);
    let arguments = normalized_call.split(',').collect::<Vec<_>>();
    assert_eq!(
        arguments.len(),
        7,
        "{label} must retain six direct arguments and a trailing comma"
    );
    assert_eq!(
        arguments[2], policy,
        "{label} must select {policy} directly"
    );
}

fn collect_rust_sources(dir: &Path, sources: &mut Vec<(PathBuf, String)>) {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read source entry").path())
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        if path.is_dir() {
            collect_rust_sources(&path, sources);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            sources.push((path, source));
        }
    }
}

#[test]
fn bigint_number_policy_is_closed_and_projects_only_at_the_number_branch() {
    let policy = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) enum BigIntNumberPolicy {",
        "\n}",
    );
    assert_eq!(
        without_whitespace(policy),
        "RejectNumber,NumberToBigInt,",
        "the Number-admission domain must remain exactly two-state"
    );
    assert!(!OPERATIONS_SOURCE.contains(")]\npub(crate) enum BigIntNumberPolicy"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(!OPERATIONS_SOURCE.contains(&format!("impl {capability} for BigIntNumberPolicy")));
    }

    let value_signature = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn emit_value_to_bigint_locals",
        ") -> Result<(), EmitError> {",
    );
    assert_eq!(
        without_whitespace(value_signature),
        without_whitespace(
            r#"(
                &mut self,
                input_tag_local: u32,
                input_payload_local: u32,
                number_policy: BigIntNumberPolicy,
                output_payload_local: u32,
                output_tag_local: u32,
                function: &mut Function,
            "#,
        ),
        "the value helper must require the closed policy in the fixed parameter position"
    );

    let primitive_signature = bounded(
        OPERATIONS_SOURCE,
        "\n    fn emit_primitive_to_bigint_locals",
        ") -> Result<(), EmitError> {",
    );
    assert_eq!(
        without_whitespace(primitive_signature),
        without_whitespace(
            r#"(
                &mut self,
                input_tag_local: u32,
                input_payload_local: u32,
                number_policy: BigIntNumberPolicy,
                output_payload_local: u32,
                output_tag_local: u32,
                function: &mut Function,
            "#,
        ),
        "the primitive helper must require the same closed policy"
    );

    let value_helper = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn emit_value_to_bigint_locals(",
        "\n    fn emit_primitive_to_bigint_locals(",
    );
    assert!(!value_helper.contains("allow_number"));
    assert_eq!(
        value_helper
            .matches("emit_tagged_to_primitive_locals(")
            .count(),
        1,
        "the value helper must perform ToPrimitive exactly once"
    );
    assert_eq!(
        value_helper
            .matches("emit_primitive_to_bigint_locals(")
            .count(),
        1,
        "the value helper must have one primitive-policy forwarding edge"
    );
    let normalized_value_helper = without_whitespace(value_helper);
    let conversion_then_forwarding = without_whitespace(
        r#"
        self.emit_tagged_to_primitive_locals(
            ToPrimitiveHint::Number,
            input_payload_local,
            input_tag_local,
            primitive_payload_local,
            primitive_tag_local,
            ToPrimitiveAbruptRoute::ReturnCurrentFunction,
            function,
        )?;
        self.emit_primitive_to_bigint_locals(
            primitive_tag_local,
            primitive_payload_local,
            number_policy,
            output_payload_local,
            output_tag_local,
            function,
        )?;
        "#,
    );
    assert_eq!(
        normalized_value_helper
            .matches(conversion_then_forwarding.as_str())
            .count(),
        1,
        "number-hinted ToPrimitive must return abrupt completion before forwarding the unchanged policy"
    );

    let primitive_helper = bounded(
        OPERATIONS_SOURCE,
        "\n    fn emit_primitive_to_bigint_locals(",
        "\n    pub(crate) fn emit_string_to_bigint_locals(",
    );
    assert!(!OPERATIONS_SOURCE.contains("pub(crate) fn emit_primitive_to_bigint_locals"));
    assert!(!primitive_helper.contains("allow_number"));
    assert_eq!(
        primitive_helper.matches("match number_policy {").count(),
        1,
        "the policy must be projected exactly once"
    );
    let normalized_primitive_helper = without_whitespace(primitive_helper);
    let number_branch_projection = without_whitespace(
        r#"
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        match number_policy {
        "#,
    );
    assert_eq!(
        normalized_primitive_helper
            .matches(number_branch_projection.as_str())
            .count(),
        1,
        "the policy projection must begin only after the Number-tag branch is entered"
    );
    let number_projection = bounded(
        primitive_helper,
        "match number_policy {",
        "\n        function.instruction(&Instruction::Else);",
    );
    assert_eq!(
        number_projection
            .matches("BigIntNumberPolicy::NumberToBigInt =>")
            .count(),
        1
    );
    assert_eq!(
        number_projection
            .matches("BigIntNumberPolicy::RejectNumber =>")
            .count(),
        1
    );
    assert_eq!(
        number_projection.matches("=>").count(),
        2,
        "the exhaustive policy projection must contain only its two named arms"
    );
    assert!(!number_projection.contains("_ =>"));
    assert!(!number_projection.contains("if number_policy"));

    let number_to_bigint = bounded(
        number_projection,
        "BigIntNumberPolicy::NumberToBigInt => {",
        "BigIntNumberPolicy::RejectNumber => {",
    );
    assert_eq!(number_to_bigint.matches("RANGE_ERROR_NAME").count(), 3);
    assert_eq!(
        number_to_bigint
            .matches("Instruction::I64TruncF64S")
            .count(),
        1
    );
    let nan_check = number_to_bigint
        .find("Instruction::F64Ne")
        .expect("missing NaN rejection");
    let infinity_checks = number_to_bigint
        .find("for infinite in [f64::INFINITY, f64::NEG_INFINITY]")
        .expect("missing infinity rejection");
    let integral_check = number_to_bigint
        .find("Instruction::F64Trunc)")
        .expect("missing integral check");
    let conversion = number_to_bigint
        .find("Instruction::I64TruncF64S")
        .expect("missing integral Number conversion");
    assert!(
        nan_check < infinity_checks
            && infinity_checks < integral_check
            && integral_check < conversion,
        "NumberToBigInt must reject NaN, infinities and fractions before conversion"
    );

    let reject_number = number_projection
        .split_once("BigIntNumberPolicy::RejectNumber => {")
        .expect("missing RejectNumber arm")
        .1;
    assert_eq!(reject_number.matches("TYPE_ERROR_NAME").count(), 1);
    assert_eq!(
        reject_number
            .matches("emit_return_current_completion(function)")
            .count(),
        1,
        "ToBigInt Number rejection must retain its immediate completion return"
    );
}

#[test]
fn bigint_number_policy_has_exactly_six_rejections_and_one_admission() {
    let payload_projection = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn compile_spec_operation_payload(",
        "\n    pub(crate) fn compile_spec_operation_to_locals(",
    );
    let locals_projection = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn compile_spec_operation_to_locals(",
        "\n    pub(crate) fn emit_to_boolean_payload_from_expr(",
    );
    let typed_data_word = bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_to_bigint_value_and_u64_word_from_value_locals(",
        "\n    pub(crate) fn emit_value_to_typed_array_element_payload(",
    );
    let bigint_builtin = bounded(
        BIGINT_SOURCE,
        "fn emit_bigint_builtin(",
        "\n    fn emit_bigint_exact_value_result(",
    );
    let zoned_date_time_constructor = bounded(
        TEMPORAL_SOURCE,
        "pub(crate) fn emit_temporal_zoned_date_time_constructor(",
        "\n    pub(crate) fn emit_temporal_zoned_date_time_time_zone(",
    );
    let instant_constructor = bounded(
        TEMPORAL_SOURCE,
        "pub(crate) fn emit_temporal_instant_constructor(",
        "\n    fn emit_temporal_parse_iso_string(",
    );
    let from_epoch_nanoseconds = bounded(
        TEMPORAL_INSTANT_SOURCE,
        "pub(crate) fn emit_temporal_instant_from_epoch_nanoseconds(",
        "\n    pub(crate) fn emit_temporal_instant_from_epoch_milliseconds(",
    );

    for (label, body) in [
        ("SpecOperationIr::ToBigInt payload", payload_projection),
        ("SpecOperationIr::ToBigInt locals", locals_projection),
        ("typed-data low-word ToBigInt", typed_data_word),
        (
            "Temporal.ZonedDateTime constructor",
            zoned_date_time_constructor,
        ),
        ("Temporal.Instant constructor", instant_constructor),
        (
            "Temporal.Instant.fromEpochNanoseconds",
            from_epoch_nanoseconds,
        ),
    ] {
        assert_one_value_call_with_policy(body, "BigIntNumberPolicy::RejectNumber", label);
    }
    assert_one_value_call_with_policy(
        bigint_builtin,
        "BigIntNumberPolicy::NumberToBigInt",
        "%BigInt% function",
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);

    let mut total_value_mentions = 0;
    let mut total_primitive_mentions = 0;
    for (path, source) in sources {
        let relative = path
            .strip_prefix(&source_root)
            .expect("collected source must remain below src")
            .to_string_lossy();
        let expected_value_mentions = match relative.as_ref() {
            "operations.rs" => 3,
            "objects.rs" | "builtins/bigint.rs" | "builtins/temporal_instant.rs" => 1,
            "builtins/temporal.rs" => 2,
            _ => 0,
        };
        let value_mentions = source.matches("emit_value_to_bigint_locals(").count();
        assert_eq!(
            value_mentions, expected_value_mentions,
            "unexpected value-helper definition or caller inventory in {relative}"
        );
        assert_eq!(
            source.matches("::emit_value_to_bigint_locals").count(),
            0,
            "the value helper must not escape the direct caller inventory as a method item in {relative}"
        );
        total_value_mentions += value_mentions;

        let expected_primitive_mentions = if relative.as_ref() == "operations.rs" {
            2
        } else {
            0
        };
        let primitive_mentions = source.matches("emit_primitive_to_bigint_locals").count();
        assert_eq!(
            primitive_mentions, expected_primitive_mentions,
            "the primitive helper must have only its definition and unique forwarding caller"
        );
        total_primitive_mentions += primitive_mentions;
    }

    assert_eq!(
        total_value_mentions, 8,
        "one definition plus exactly seven external callers must remain"
    );
    assert_eq!(
        total_primitive_mentions, 2,
        "one primitive-helper definition plus one forwarding call must remain"
    );
}
