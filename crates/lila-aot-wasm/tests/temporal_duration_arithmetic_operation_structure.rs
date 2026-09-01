const DURATION_METHODS_SOURCE: &str = include_str!("../src/builtins/temporal_duration_methods.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn duration_arithmetic_operation_is_a_closed_domain() {
    let declaration = bounded(
        DURATION_METHODS_SOURCE,
        "enum TemporalDurationArithmeticOperation {",
        "\n}\n\nimpl<'a> FunctionBuilder<'a>",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(variants, ["Add,", "Subtract,"]);
    assert!(!declaration.contains("Default"));
}

#[test]
fn duration_arithmetic_selects_exhaustively_after_rhs_coercion() {
    let emitter = bounded(
        DURATION_METHODS_SOURCE,
        "    fn emit_temporal_duration_arithmetic(",
        "    pub(crate) fn emit_temporal_duration_renormalize(",
    );

    assert!(emitter.contains("operation: TemporalDurationArithmeticOperation"));
    assert!(emitter.contains("match operation {"));
    assert_eq!(
        emitter
            .matches("TemporalDurationArithmeticOperation::Add =>")
            .count(),
        1
    );
    assert_eq!(
        emitter
            .matches("TemporalDurationArithmeticOperation::Subtract =>")
            .count(),
        1
    );
    assert!(
        emitter.find("self.emit_to_temporal_duration(").unwrap()
            < emitter.find("match operation {").unwrap(),
        "RHS coercion must remain observable before arithmetic selection"
    );
    assert!(!emitter.contains("_ =>"));
    assert!(!emitter.contains("unreachable!"));
    assert!(!emitter.contains(": bool"));
    assert!(!emitter.contains("if subtract"));
}

#[test]
fn duration_arithmetic_wrappers_and_dispatch_pin_both_operations() {
    let add = bounded(
        DURATION_METHODS_SOURCE,
        "    pub(crate) fn emit_temporal_duration_add(",
        "    pub(crate) fn emit_temporal_duration_subtract(",
    );
    assert_eq!(
        add.matches("TemporalDurationArithmeticOperation::Add")
            .count(),
        1
    );
    assert!(!add.contains("::Subtract"));

    let subtract = bounded(
        DURATION_METHODS_SOURCE,
        "    pub(crate) fn emit_temporal_duration_subtract(",
        "    fn emit_temporal_duration_arithmetic(",
    );
    assert_eq!(
        subtract
            .matches("TemporalDurationArithmeticOperation::Subtract")
            .count(),
        1
    );
    assert!(!subtract.contains("::Add"));

    let dispatch = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::TemporalDurationPrototypeAdd => {",
        "            StandardBuiltinId::TemporalDurationPrototypeRound => {",
    );
    assert_eq!(dispatch.matches("emit_temporal_duration_add(").count(), 1);
    assert_eq!(
        dispatch.matches("emit_temporal_duration_subtract(").count(),
        1
    );
    assert!(!dispatch.contains("emit_temporal_duration_add_or_subtract"));
    assert!(!dispatch.contains("true, function"));
    assert!(!dispatch.contains("false, function"));

    assert_eq!(
        DURATION_METHODS_SOURCE
            .matches("emit_temporal_duration_arithmetic(")
            .count(),
        3,
        "one private emitter and two named wrappers own arithmetic selection"
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("emit_temporal_duration_add(")
            .count(),
        1
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("emit_temporal_duration_subtract(")
            .count(),
        1
    );
}
