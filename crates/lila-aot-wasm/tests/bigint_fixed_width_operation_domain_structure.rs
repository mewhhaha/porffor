const BIGINT_SOURCE: &str = include_str!("../src/builtins/bigint.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const BIGINT_AS_N_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_bigint_as_n_arbitrary_width.js");
const NUMERICS_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/language_numerics.rs");

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

#[test]
fn fixed_width_operation_is_the_exact_non_copy_domain() {
    let declaration = bounded(
        BIGINT_SOURCE,
        "enum BigIntFixedWidthOperation {",
        "enum BigIntBuiltin {",
    );
    assert_eq!(
        declaration
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        ["Signed,", "Unsigned,", "}"]
    );

    let declaration_offset = BIGINT_SOURCE
        .find("enum BigIntFixedWidthOperation {")
        .expect("fixed-width operation declaration");
    assert_eq!(
        BIGINT_SOURCE[..declaration_offset]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("}")
    );
    for forbidden in [
        "impl Clone for BigIntFixedWidthOperation",
        "impl Copy for BigIntFixedWidthOperation",
        "impl Default for BigIntFixedWidthOperation",
        "impl PartialEq for BigIntFixedWidthOperation",
        "impl Eq for BigIntFixedWidthOperation",
    ] {
        assert!(!BIGINT_SOURCE.contains(forbidden), "found `{forbidden}`");
    }

    let builtin_declaration = bounded(
        BIGINT_SOURCE,
        "enum BigIntBuiltin {",
        "#[allow(non_upper_case_globals)]",
    );
    assert!(builtin_declaration.contains("FixedWidth(BigIntFixedWidthOperation),"));
    assert!(!builtin_declaration.contains("AsIntN,"));
    assert!(!builtin_declaration.contains("AsUintN,"));
    assert!(!BIGINT_SOURCE.contains("impl Copy for BigIntBuiltin"));
    assert!(!BIGINT_SOURCE.contains("impl PartialEq for BigIntBuiltin"));
    assert!(!BIGINT_SOURCE.contains("impl Eq for BigIntBuiltin"));
    assert!(!BIGINT_SOURCE.contains("pub(super) enum BigIntFixedWidthOperation"));
    assert!(!BIGINT_SOURCE.contains("pub(super) enum BigIntBuiltin"));
}

#[test]
fn standard_dispatch_reaches_bigint_only_through_fixed_semantic_entries() {
    let dispatch = normalized(STANDARD_SOURCE);
    for mapping in [
        "StandardBuiltinId::BigIntConstructor=>{self.emit_bigint_constructor_builtin(function)?}",
        "StandardBuiltinId::BigIntAsIntN=>self.emit_bigint_as_int_n_builtin(function)?,",
        "StandardBuiltinId::BigIntAsUintN=>self.emit_bigint_as_uint_n_builtin(function)?,",
        "StandardBuiltinId::BigIntPrototypeToString=>{self.emit_bigint_prototype_to_string_builtin(function)?}",
        "StandardBuiltinId::BigIntPrototypeToLocaleString=>{self.emit_bigint_prototype_to_locale_string_builtin(function)?}",
        "StandardBuiltinId::BigIntPrototypeValueOf=>{self.emit_bigint_prototype_value_of_builtin(function)?}",
    ] {
        assert_eq!(dispatch.matches(mapping).count(), 1, "mapping `{mapping}`");
    }
    assert!(!STANDARD_SOURCE.contains("BigIntBuiltin"));
    assert!(!STANDARD_SOURCE.contains("BigIntFixedWidthOperation"));
    assert!(!STANDARD_SOURCE.contains("emit_bigint_builtin("));
    assert_eq!(
        BIGINT_SOURCE
            .matches("BigIntBuiltin::FixedWidth(BigIntFixedWidthOperation::Signed)")
            .count(),
        1
    );
    assert_eq!(
        BIGINT_SOURCE
            .matches("BigIntBuiltin::FixedWidth(BigIntFixedWidthOperation::Unsigned)")
            .count(),
        1
    );
    assert_eq!(
        BIGINT_SOURCE.matches("self.emit_bigint_builtin(").count(),
        6
    );
}

#[test]
fn fixed_width_consumer_has_four_exhaustive_borrowed_decisions() {
    let consumer = bounded(
        BIGINT_SOURCE,
        "BigIntBuiltin::FixedWidth(operation) => {",
        "BigIntBuiltin::Prototype(result_policy) => {",
    );
    assert_eq!(consumer.matches("match &operation {").count(), 4);
    assert_eq!(
        consumer
            .matches("BigIntFixedWidthOperation::Signed =>")
            .count(),
        4
    );
    assert_eq!(
        consumer
            .matches("BigIntFixedWidthOperation::Unsigned =>")
            .count(),
        4
    );
    assert_eq!(consumer.matches("=>").count(), 8);
    for forbidden in [
        "operation ==",
        "operation !=",
        "builtin ==",
        "builtin !=",
        "signed: bool",
        "unsigned: bool",
        "_ =>",
        "unreachable!",
        "debug_assert!",
        "matches!",
    ] {
        assert!(
            !consumer.contains(forbidden),
            "consumer contains `{forbidden}`"
        );
    }

    let consumer = normalized(consumer);
    let sub_64 = bounded(
        &consumer,
        "function.instruction(&Instruction::I64And);function.instruction(&Instruction::LocalSet(word_payload_local));match&operation{",
        "}function.instruction(&Instruction::LocalGet(word_payload_local));function.instruction(&Instruction::LocalSet(self.result_local));",
    );
    assert_eq!(
        sub_64,
        concat!(
            "BigIntFixedWidthOperation::Signed=>{",
            "function.instruction(&Instruction::I64Const(1));",
            "function.instruction(&Instruction::LocalGet(index_local));",
            "function.instruction(&Instruction::I64Const(1));",
            "function.instruction(&Instruction::I64Sub);",
            "function.instruction(&Instruction::I64Shl);",
            "function.instruction(&Instruction::LocalSet(sign_local));",
            "function.instruction(&Instruction::LocalGet(word_payload_local));",
            "function.instruction(&Instruction::LocalGet(sign_local));",
            "function.instruction(&Instruction::I64And);",
            "function.instruction(&Instruction::I64Const(0));",
            "function.instruction(&Instruction::I64Ne);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "function.instruction(&Instruction::LocalGet(word_payload_local));",
            "function.instruction(&Instruction::LocalGet(mask_local));",
            "function.instruction(&Instruction::I64Const(1));",
            "function.instruction(&Instruction::I64Add);",
            "function.instruction(&Instruction::I64Sub);",
            "function.instruction(&Instruction::LocalSet(word_payload_local));",
            "function.instruction(&Instruction::End);",
            "}BigIntFixedWidthOperation::Unsigned=>{}",
        )
    );

    let exact_64 = bounded(
        &consumer,
        "function.instruction(&Instruction::I64Const(64));function.instruction(&Instruction::I64Eq);function.instruction(&Instruction::If(BlockType::Empty));match&operation{",
        "}function.instruction(&Instruction::Else);function.instruction(&Instruction::LocalGet(bigint_tag_local));",
    );
    assert_eq!(
        exact_64,
        concat!(
            "BigIntFixedWidthOperation::Signed=>{",
            "function.instruction(&Instruction::LocalGet(word_payload_local));",
            "function.instruction(&Instruction::LocalSet(self.result_local));",
            "function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag()asi64));",
            "function.instruction(&Instruction::LocalSet(self.result_tag_local));",
            "}BigIntFixedWidthOperation::Unsigned=>{",
            "function.instruction(&Instruction::LocalGet(word_payload_local));",
            "function.instruction(&Instruction::I64Const(0));",
            "function.instruction(&Instruction::I64LtS);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "self.emit_alloc_one_limb_bigint(1,word_payload_local,function)?;",
            "function.instruction(&Instruction::LocalSet(self.result_local));",
            "function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));",
            "function.instruction(&Instruction::LocalSet(self.result_tag_local));",
            "function.instruction(&Instruction::Else);",
            "function.instruction(&Instruction::LocalGet(word_payload_local));",
            "function.instruction(&Instruction::LocalSet(self.result_local));",
            "function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag()asi64));",
            "function.instruction(&Instruction::LocalSet(self.result_tag_local));",
            "function.instruction(&Instruction::End);",
            "}",
        )
    );

    let wide_passthrough = bounded(
        &consumer,
        "function.instruction(&Instruction::LocalGet(input_limb_count_local));function.instruction(&Instruction::LocalGet(result_capacity_local));function.instruction(&Instruction::I64LtU);function.instruction(&Instruction::I32And);match&operation{",
        "}function.instruction(&Instruction::If(BlockType::Empty));function.instruction(&Instruction::LocalGet(bigint_payload_local));",
    );
    assert_eq!(
        wide_passthrough,
        concat!(
            "BigIntFixedWidthOperation::Signed=>{}",
            "BigIntFixedWidthOperation::Unsigned=>{",
            "function.instruction(&Instruction::LocalGet(input_sign_local));",
            "function.instruction(&Instruction::I64Const(0));",
            "function.instruction(&Instruction::I64GeS);",
            "function.instruction(&Instruction::I32And);",
            "}",
        )
    );

    let wide_signed = bounded(
        &consumer,
        "function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::LocalSet(result_sign_local));match&operation{BigIntFixedWidthOperation::Signed=>{",
        "}BigIntFixedWidthOperation::Unsigned=>{}}function.instruction(&Instruction::LocalGet(result_capacity_local));",
    );
    assert_eq!(wide_signed.matches("function.instruction(").count(), 84);
    assert_eq!(wide_signed.matches("Instruction::Loop").count(), 1);
    assert_eq!(wide_signed.matches("Instruction::I64Xor").count(), 1);
    assert_eq!(wide_signed.matches("Instruction::I64Store").count(), 2);
    assert_eq!(wide_signed.matches("Instruction::End").count(), 3);
    assert!(wide_signed.starts_with(concat!(
        "function.instruction(&Instruction::I64Const(1));",
        "function.instruction(&Instruction::LocalGet(index_local));",
        "function.instruction(&Instruction::I64Const(1));",
        "function.instruction(&Instruction::I64Sub);",
        "function.instruction(&Instruction::I64Const(63));",
        "function.instruction(&Instruction::I64And);",
        "function.instruction(&Instruction::I64Shl);",
        "function.instruction(&Instruction::LocalSet(sign_local));",
    )));
    assert!(wide_signed.contains(concat!(
        "function.instruction(&Instruction::I64Load(self.buffer_memarg64(0)));",
        "function.instruction(&Instruction::I64Const(-1));",
        "function.instruction(&Instruction::I64Xor);",
        "function.instruction(&Instruction::LocalGet(carry_local));",
        "function.instruction(&Instruction::I64Add);",
        "function.instruction(&Instruction::LocalSet(word_payload_local));",
    )));
    assert!(wide_signed.ends_with(concat!(
        "function.instruction(&Instruction::I64Load(self.buffer_memarg64(0)));",
        "function.instruction(&Instruction::LocalGet(mask_local));",
        "function.instruction(&Instruction::I64And);",
        "function.instruction(&Instruction::LocalSet(word_payload_local));",
        "function.instruction(&Instruction::LocalGet(result_limbs_local));",
        "function.instruction(&Instruction::LocalGet(result_capacity_local));",
        "function.instruction(&Instruction::I64Const(1));",
        "function.instruction(&Instruction::I64Sub);",
        "function.instruction(&Instruction::I64Const(8));",
        "function.instruction(&Instruction::I64Mul);",
        "function.instruction(&Instruction::I64Add);",
        "function.instruction(&Instruction::I32WrapI64);",
        "function.instruction(&Instruction::LocalGet(word_payload_local));",
        "function.instruction(&Instruction::I64Store(self.buffer_memarg64(0)));",
        "function.instruction(&Instruction::End);",
    )));
}

#[test]
fn fixed_width_arm_closes_six_blocks_then_releases_locals_in_reverse_order() {
    let consumer = bounded(
        BIGINT_SOURCE,
        "BigIntBuiltin::FixedWidth(operation) => {",
        "BigIntBuiltin::Prototype(result_policy) => {",
    );
    let tail_start = consumer
        .rfind("function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));")
        .expect("fixed-width heap result tag");
    let tail = normalized(&consumer[tail_start..]);
    assert_eq!(
        tail,
        concat!(
            "function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));",
            "function.instruction(&Instruction::LocalSet(self.result_tag_local));",
            "function.instruction(&Instruction::End);",
            "function.instruction(&Instruction::End);",
            "function.instruction(&Instruction::End);",
            "function.instruction(&Instruction::End);",
            "function.instruction(&Instruction::End);",
            "function.instruction(&Instruction::End);",
            "self.release_temp_local(fits_immediate_local);",
            "self.release_temp_local(record_local);",
            "self.release_temp_local(result_sign_local);",
            "self.release_temp_local(partial_bits_local);",
            "self.release_temp_local(carry_local);",
            "self.release_temp_local(limb_local);",
            "self.release_temp_local(limb_index_local);",
            "self.release_temp_local(result_capacity_local);",
            "self.release_temp_local(result_limb_count_local);",
            "self.release_temp_local(result_limbs_local);",
            "self.release_temp_local(input_magnitude_word_local);",
            "self.release_temp_local(input_limb_count_local);",
            "self.release_temp_local(input_limbs_local);",
            "self.release_temp_local(input_sign_local);",
            "self.release_temp_local(word_payload_local);",
            "self.release_temp_local(sign_local);",
            "self.release_temp_local(mask_local);",
            "self.release_temp_local(index_local);",
            "self.release_temp_local(bigint_tag_local);",
            "self.release_temp_local(bigint_payload_local);",
            "self.release_temp_local(bits_tag_local);",
            "self.release_temp_local(bits_payload_local);",
            "}",
        )
    );
}

#[test]
fn arbitrary_width_fixture_witnesses_both_operations_and_conversion_order() {
    assert_eq!(
        NUMERICS_CLI_TESTS
            .matches("fn run_wasm_backend_truncates_bigints_at_arbitrary_widths()")
            .count(),
        1
    );
    assert_eq!(
        NUMERICS_CLI_TESTS
            .matches("fixture_path(\"wasm_bigint_as_n_arbitrary_width.js\")")
            .count(),
        1
    );
    for witness in [
        "BigInt.asUintN(0, zeroWidthInput)",
        "BigInt.asIntN(0, zeroWidthInput)",
        "BigInt.asUintN(64, -1n)",
        "BigInt.asIntN(64, 0x8000000000000000n)",
        "BigInt.asUintN(65, positive)",
        "BigInt.asIntN(65, positive)",
        "BigInt.asIntN(200, wide)",
        "BigInt.asUintN(200, unsignedWide)",
        "conversionOrder !== \"bits,bigint\"",
        "if (bigintConversionReached)",
    ] {
        assert!(
            BIGINT_AS_N_FIXTURE.contains(witness),
            "missing fixture witness `{witness}`"
        );
    }
}
