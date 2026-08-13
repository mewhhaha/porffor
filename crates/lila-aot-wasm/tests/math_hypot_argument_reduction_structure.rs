const MATH_SOURCE: &str = include_str!("../src/builtins/math.rs");

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
fn hypot_reduction_witness_is_private_linear_and_completed_by_one_consumer() {
    let declaration = MATH_SOURCE
        .split_once("struct CompletedMathHypotReduction {")
        .expect("completed reduction witness")
        .0
        .rsplit_once("\n\n")
        .expect("witness attribute boundary")
        .1;
    assert!(declaration.contains("#[must_use"));
    assert!(!declaration.contains("derive"));
    assert!(!declaration.contains("pub"));
    assert!(!MATH_SOURCE.contains("impl Copy for CompletedMathHypotReduction"));

    let fields = MATH_SOURCE
        .split_once("struct CompletedMathHypotReduction {")
        .expect("completed reduction fields")
        .1
        .split_once('}')
        .expect("completed reduction fields end")
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        fields,
        [
            "scale_local: u32,",
            "scaled_sum_local: u32,",
            "saw_infinity_local: u32,",
            "saw_nan_local: u32,",
        ]
    );
    assert_eq!(
        MATH_SOURCE.matches("CompletedMathHypotReduction {").count(),
        3,
        "the witness must have one declaration, producer and consuming destructure"
    );

    let hypot_arm = bounded(
        MATH_SOURCE,
        "            MathBuiltin::Hypot => {",
        "            MathBuiltin::Atan2 => {",
    );
    assert_eq!(
        hypot_arm
            .matches("emit_math_hypot_argument_reduction(")
            .count(),
        1
    );
    assert_eq!(hypot_arm.matches("emit_finish_math_hypot(").count(), 1);
    assert!(!hypot_arm.contains("emit_builtin_arg_to_locals("));
    assert!(!hypot_arm.contains("for arg_index in"));
}

#[test]
fn hypot_producer_walks_every_runtime_argument_before_finishing() {
    let producer = bounded(
        MATH_SOURCE,
        "    fn emit_math_hypot_argument_reduction(",
        "    fn emit_finish_math_hypot(",
    );

    for operation in [
        "Instruction::Loop(BlockType::Empty)",
        "self.argv_param_local(),",
        "self.argc_param_local()",
        "Instruction::I64GeU",
        "Instruction::BrIf(1)",
        "emit_value_to_number_payload(",
        "emit_return_current_completion_if_throw(function)",
        "Instruction::Br(0)",
    ] {
        assert!(producer.contains(operation), "missing `{operation}`");
    }
    assert_eq!(producer.matches("emit_array_read(").count(), 1);
    assert_eq!(producer.matches("emit_value_to_number_payload(").count(), 1);
    assert_eq!(
        producer
            .matches("emit_return_current_completion_if_throw(function)")
            .count(),
        1
    );
    assert_eq!(producer.matches("Instruction::Br(0)").count(), 1);
    assert_eq!(producer.matches("Instruction::BrIf(").count(), 1);
    assert!(!producer.contains("emit_builtin_arg_to_locals("));
    assert!(!producer.contains("for arg_index in"));

    assert_before(
        producer,
        "emit_array_read(",
        "emit_value_to_number_payload(",
    );
    assert_before(
        producer,
        "emit_value_to_number_payload(",
        "emit_return_current_completion_if_throw(function)",
    );
    assert_before(
        producer,
        "emit_return_current_completion_if_throw(function)",
        "Instruction::F64Abs",
    );
    assert_before(
        producer,
        "Instruction::LocalSet(saw_infinity_local)",
        "Instruction::LocalGet(argument_index_local));\n        function.instruction(&Instruction::I64Const(1))",
    );
    assert_before(
        producer,
        "Instruction::LocalSet(saw_nan_local)",
        "Instruction::LocalGet(argument_index_local));\n        function.instruction(&Instruction::I64Const(1))",
    );
}

#[test]
fn hypot_finite_fold_is_scaled_and_finish_precedence_is_closed() {
    let producer = bounded(
        MATH_SOURCE,
        "    fn emit_math_hypot_argument_reduction(",
        "    fn emit_finish_math_hypot(",
    );
    assert_eq!(producer.matches("Instruction::F64Div").count(), 2);
    assert_eq!(
        producer
            .matches("Instruction::LocalSet(ratio_local)")
            .count(),
        2
    );
    assert_eq!(producer.matches("Instruction::F64Mul").count(), 3);
    assert!(producer.contains("Instruction::LocalSet(scale_local)"));
    assert!(producer.contains("Instruction::LocalSet(scaled_sum_local)"));
    assert!(!producer.contains("Instruction::F64Sqrt"));

    let finish = bounded(
        MATH_SOURCE,
        "    fn emit_finish_math_hypot(",
        "    fn emit_math_extremum_builtin(",
    );
    assert_eq!(finish.matches("Instruction::F64Sqrt").count(), 1);
    assert_eq!(finish.matches("Instruction::F64Mul").count(), 1);
    assert_before(
        finish,
        "Instruction::LocalGet(saw_infinity_local)",
        "Instruction::LocalGet(saw_nan_local)",
    );
    assert_before(
        finish,
        "Instruction::LocalGet(saw_nan_local)",
        "Instruction::LocalGet(scale_local)",
    );
    assert_before(finish, "Ieee64::from(0.0)", "Ieee64::from(f64::NAN)");
    assert_before(
        finish,
        "Ieee64::from(f64::NAN)",
        "Ieee64::from(f64::INFINITY)",
    );
}
