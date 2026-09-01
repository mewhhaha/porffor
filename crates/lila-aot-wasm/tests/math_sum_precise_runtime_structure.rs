const MATH_SOURCE: &str = include_str!("../src/builtins/math.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/math-sum-precise-runtime.md");
const TASK: &str = include_str!("../../../tasks/20-number-bigint-math-json.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn sum_precise_proof_state_and_phase_witness_are_closed() {
    for proof in [
        "const MATH_SUM_PRECISE_MAX_COUNT: i64 = (1_i64 << 53) - 1;",
        "const MATH_SUM_PRECISE_MAX_EXACT_BITS: usize = 2_151;",
        "const MATH_SUM_PRECISE_LIMB_BITS: usize = 64;",
        "const _: () = assert!(MATH_SUM_PRECISE_LIMBS == 34);",
    ] {
        assert!(
            MATH_SOURCE.contains(proof),
            "missing proof invariant `{proof}`"
        );
    }

    let state = bounded(
        MATH_SOURCE,
        "enum MathSumPreciseState {",
        "enum MathSumPreciseLimbOperation",
    );
    assert_eq!(
        bounded(MATH_SOURCE, "enum MathSumPreciseState {", "\n}")
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>(),
        [
            "MinusZero,",
            "Finite,",
            "PlusInfinity,",
            "MinusInfinity,",
            "NotANumber,",
        ]
    );
    let state_declaration_prefix = MATH_SOURCE
        .split_once("enum MathSumPreciseState {")
        .expect("Math.sumPrecise state declaration")
        .0
        .rsplit_once("\n\n")
        .expect("Math.sumPrecise state attribute boundary")
        .1;
    assert!(state_declaration_prefix.trim().is_empty());
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
        assert!(!MATH_SOURCE.contains(&format!("impl {capability} for MathSumPreciseState")));
    }
    assert_eq!(state.matches("const fn abi_word(self) -> i64").count(), 1);
    assert_eq!(state.matches("match self {").count(), 1);
    for variant in [
        "MinusZero",
        "Finite",
        "PlusInfinity",
        "MinusInfinity",
        "NotANumber",
    ] {
        assert_eq!(
            state.matches(variant).count(),
            2,
            "state and ABI arm for {variant}"
        );
    }
    assert!(!state.contains("_ =>"));
    assert!(!state.contains(".clone()"));

    for evidence in [CONTRACT, TASK] {
        let words = evidence.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(words.contains("capability-free `MathSumPreciseState`"));
        assert!(words.contains("Batch AL"));
    }

    let declaration = MATH_SOURCE
        .split_once("struct CompletedMathSumPreciseReduction {")
        .expect("completed reduction witness")
        .0
        .rsplit_once("\n\n")
        .expect("witness attribute boundary")
        .1;
    assert!(declaration.contains("#[must_use"));
    assert!(!declaration.contains("derive"));
    assert!(!declaration.contains("pub"));
    assert_eq!(
        MATH_SOURCE
            .matches("CompletedMathSumPreciseReduction {")
            .count(),
        3,
        "one declaration, one producer and one consuming destructure"
    );
    assert!(!MATH_SOURCE.contains("impl Copy for CompletedMathSumPreciseReduction"));
}

#[test]
fn sum_precise_runtime_walk_orders_count_type_fold_and_close() {
    let producer = bounded(
        MATH_SOURCE,
        "    fn emit_math_sum_precise_reduction(",
        "    fn emit_math_sum_precise_make_magnitude(",
    );
    for operation in [
        "emit_builtin_arg_to_locals(",
        "emit_get_iterator_from_value_locals(",
        "let consumer = SyncIteratorConsumer::MathSumPrecise;",
        "emit_math_sum_precise_initialize_accumulator(&accumulator, function)",
        "emit_sync_iterator_step_value(",
        "MATH_SUM_PRECISE_MAX_COUNT",
        "emit_throw_current_function_realm_range_error(",
        "emit_throw_current_function_realm_type_error(",
        "emit_math_sum_precise_accept_number(",
    ] {
        assert!(producer.contains(operation), "missing `{operation}`");
    }
    assert_eq!(
        producer
            .matches("emit_iterator_close_preserving_current_throw(close, function)")
            .count(),
        2,
        "only count and exact-Number failures close"
    );
    assert_eq!(
        producer
            .matches("self.emit_return_current_completion(function)")
            .count(),
        2
    );
    let after_guard = producer
        .split_once("MATH_SUM_PRECISE_MAX_COUNT")
        .expect("count guard")
        .1;
    assert_before(
        after_guard,
        "ValueKind::Number.tag()",
        "emit_math_sum_precise_accept_number(",
    );
    assert_before(
        after_guard,
        "emit_math_sum_precise_accept_number(",
        "Instruction::LocalSet(count_local)",
    );

    assert_before(
        producer,
        "ptr_local: self.reserve_temp_local()",
        "let state_local = self.reserve_temp_local();",
    );
    for scratch in [
        "let method_payload_local = self.reserve_temp_local();",
        "let method_tag_local = self.reserve_temp_local();",
        "let iterator_locals = self.reserve_sync_iterator_locals();",
        "let done_local = self.reserve_temp_local();",
        "let count_local = self.reserve_temp_local();",
        "let close_saved_payload_local = self.reserve_temp_local();",
        "let close_saved_tag_local = self.reserve_temp_local();",
        "let close_saved_completion_local = self.reserve_temp_local();",
        "let close_saved_aux_local = self.reserve_temp_local();",
    ] {
        assert_before(
            producer,
            "let state_local = self.reserve_temp_local();",
            scratch,
        );
    }
    assert_before(
        producer,
        "emit_get_iterator_from_value_locals(",
        "emit_math_sum_precise_initialize_accumulator(&accumulator, function)",
    );
    assert_before(
        producer,
        "self.release_temp_local(method_payload_local);",
        "Ok(CompletedMathSumPreciseReduction {",
    );

    let arm = bounded(
        MATH_SOURCE,
        "            MathBuiltin::SumPrecise => {",
        "            MathBuiltin::Hypot => {",
    );
    assert_eq!(arm.matches("emit_math_sum_precise_reduction(").count(), 1);
    assert_eq!(arm.matches("emit_finish_math_sum_precise(").count(), 1);
    assert!(!arm.contains("emit_throw_runtime_error("));

    let finish = bounded(
        MATH_SOURCE,
        "    fn emit_finish_math_sum_precise(",
        "    fn emit_math_hypot_argument_reduction(",
    );
    assert_before(
        finish,
        "self.release_temp_local(state_local);",
        "self.release_temp_local(accumulator.ptr_local);",
    );
}

#[test]
fn sum_precise_finite_fold_is_fixed_width_and_rounds_once() {
    let exact = bounded(
        MATH_SOURCE,
        "    fn emit_math_sum_precise_load_limb(",
        "    fn emit_math_hypot_argument_reduction(",
    );
    for operation in [
        "Instruction::I64Load(Self::memarg64(0))",
        "Instruction::I64Store(Self::memarg64(0))",
        "MathSumPreciseLimbOperation::Add",
        "MathSumPreciseLimbOperation::Subtract",
        "Instruction::I64Clz",
        "emit_math_sum_precise_extract_bit(",
        "emit_math_sum_precise_sticky_below(",
        "Instruction::I64Const(2_098)",
        "Instruction::Unreachable",
    ] {
        assert!(
            exact.contains(operation),
            "missing exact operation `{operation}`"
        );
    }
    assert!(!exact.contains("Instruction::F64Add"));
    assert!(!exact.contains("emit_value_to_number"));

    let limb_fold = bounded(
        exact,
        "    fn emit_math_sum_precise_fold_limbs(",
        "    fn emit_math_sum_precise_add_finite(",
    );
    let carry_advance = concat!(
        "Instruction::LocalGet(next_carry_local));\n",
        "        function.instruction(&Instruction::LocalSet(carry_local));\n",
        "        function.instruction(&Instruction::LocalGet(index_local))"
    );
    assert_eq!(limb_fold.matches(carry_advance).count(), 1);
    assert_eq!(
        limb_fold
            .matches("function.instruction(&Instruction::LocalSet(carry_local));")
            .count(),
        2,
        "carry is initialized once and assigned once per loop iteration"
    );

    let sign_branch = concat!(
        "Instruction::I64ShrU);\n",
        "        function.instruction(&Instruction::I32WrapI64);\n",
        "        function.instruction(&Instruction::If(BlockType::Empty))"
    );
    assert_eq!(
        exact.matches(sign_branch).count(),
        2,
        "the finite-term and infinity-sign i64 flags must become i32 conditions"
    );
    let magnitude = bounded(
        exact,
        "    fn emit_math_sum_precise_make_magnitude(",
        "    fn emit_math_sum_precise_extract_bit(",
    );
    assert!(magnitude.contains(concat!(
        "Instruction::LocalGet(negative_local));\n",
        "        function.instruction(&Instruction::I32WrapI64);\n",
        "        function.instruction(&Instruction::If(BlockType::Empty))"
    )));

    let allocation = bounded(
        exact,
        "    fn emit_math_sum_precise_initialize_accumulator(",
        "    #[allow(clippy::too_many_arguments)]",
    );
    assert!(allocation.contains("MATH_SUM_PRECISE_BYTES"));
    assert!(allocation.contains("for index in 0..MATH_SUM_PRECISE_LIMBS"));
    assert!(!allocation.contains("reserve_temp_local"));

    let finish = bounded(
        MATH_SOURCE,
        "    fn emit_finish_math_sum_precise(",
        "    fn emit_math_hypot_argument_reduction(",
    );
    for state in [
        "MathSumPreciseState::MinusZero",
        "MathSumPreciseState::Finite",
        "MathSumPreciseState::PlusInfinity",
        "MathSumPreciseState::MinusInfinity",
        "MathSumPreciseState::NotANumber",
    ] {
        assert!(finish.contains(state), "finisher omits `{state}`");
    }
}

#[test]
fn sum_precise_iterator_consumer_is_exhaustive_and_does_not_close_steps() {
    let consumer = bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) enum SyncIteratorConsumer {",
        "/// Whether `Iterator.prototype.flatMap`",
    );
    assert_eq!(
        consumer
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

    let route = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn emit_sync_iterator_protocol_type_error(",
        "    fn compile_array_destructuring_element(",
    );
    for error in [
        "NotIterable",
        "MethodResultNotObject",
        "NextNotCallable",
        "NextResultNotObject",
    ] {
        assert_eq!(
            route.matches(error).count(),
            4,
            "all four consumers route {error}"
        );
    }
    assert!(!route.contains("_ =>"));
    assert!(route.contains("emit_throw_current_function_realm_type_error("));
    assert!(route.contains("emit_throw_runtime_error("));
    assert!(route.contains("match self.numeric_error_realm_source()"));
    assert!(route.contains("NumericErrorRealmSource::StandardBuiltinEnvironment"));
    assert!(route.contains("NumericErrorRealmSource::GlobalFallback"));
    assert!(route.contains("NumericErrorRealmSource::NumericConversionHelperArgument"));
    assert!(!route.contains("_ =>"));

    let step = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_sync_iterator_step_value(",
        "    fn prepare_destructuring_target<'b>(",
    );
    assert!(step.contains("SyncIteratorProtocolError::NextNotCallable"));
    assert!(step.contains("SyncIteratorProtocolError::NextResultNotObject"));
    assert!(!step.contains("emit_iterator_close"));

    let get_iterator = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_get_iterator_from_value_locals(",
        "    fn finish_get_iterator_from_method(",
    );
    assert!(get_iterator.contains(
        "let has_runtime_arguments_arm = value_info.possible_kinds.contains(ValueKind::Arguments);"
    ));
    assert!(get_iterator.contains("Instruction::I64Const(ValueKind::Arguments.tag() as i64)"));
    assert!(get_iterator.contains("emit_arguments_iterator_method_to_locals("));
    let after_runtime_arguments_guard = get_iterator
        .split_once(
            "let has_runtime_arguments_arm = value_info.possible_kinds.contains(ValueKind::Arguments);",
        )
        .expect("dynamic Arguments guard")
        .1;
    assert!(after_runtime_arguments_guard.contains("self.finish_get_iterator_from_method("));
    assert_eq!(
        get_iterator
            .matches("self.emit_value_to_current_function_realm_object_locals(")
            .count(),
        1
    );
    assert_eq!(
        get_iterator
            .matches("self.emit_value_to_object_locals(")
            .count(),
        0
    );
    assert!(!get_iterator.contains("match consumer"));

    let reduction = bounded(
        MATH_SOURCE,
        "    fn emit_math_sum_precise_reduction(",
        "    fn emit_math_sum_precise_make_magnitude(",
    );
    assert_eq!(
        reduction
            .matches("let consumer = SyncIteratorConsumer::MathSumPrecise;")
            .count(),
        1
    );
    assert_eq!(reduction.matches("&consumer").count(), 2);
}

#[test]
fn sum_precise_is_runtime_only_and_roots_sync_iterators() {
    for removed in [
        "static_sum_precise_literal_array",
        "static_sum_precise_iterable_arg",
        "static_sum_precise_values",
        "static_precise_sum",
        "round_exact_power_two_sum_to_f64",
        "static_array_iterator_overrides",
    ] {
        assert!(
            !LOWERING_SOURCE.contains(removed),
            "static route remains: {removed}"
        );
    }

    let dependencies = bounded(
        PLANNING_SOURCE,
        "        if matches!(\n            builtin,\n            StandardBuiltinId::MapConstructor",
        "        if builtin.string_prototype_method_name().is_some()",
    );
    assert!(dependencies.contains("StandardBuiltinId::MathSumPrecise"));
    for builtin in [
        "ArrayPrototypeValues",
        "ArrayIteratorNext",
        "ArrayIteratorIdentity",
        "StringPrototypeIterator",
        "StringIteratorNext",
    ] {
        assert!(dependencies.contains(builtin));
    }
    assert!(PLANNING_SOURCE.contains("fn math_sum_precise_roots_sync_iterator_machinery()"));
}

#[test]
fn sum_precise_messages_and_created_realm_math_metadata_are_complete() {
    for message in [
        "Math.sumPrecise input is not iterable",
        "Math.sumPrecise iterable contains too many values",
        "Math.sumPrecise iterator method must return an object",
        "Math.sumPrecise iterator next method is not callable",
        "Math.sumPrecise iterator next result must be an object",
        "Math.sumPrecise non-number element",
    ] {
        assert!(
            DATA_SOURCE.contains(message),
            "message is not interned: {message}"
        );
    }

    let math_methods = bounded(
        HOST_SOURCE,
        "        for (name, meta) in &math_static_method_metas {",
        "        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;",
    );
    for offset in [
        "HEAP_FUNCTION_ENV_HANDLE_OFFSET",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
        "HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert!(
            math_methods.contains(offset),
            "created-realm Math omits `{offset}`"
        );
    }
}
