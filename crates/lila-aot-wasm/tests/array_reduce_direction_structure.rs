const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-callback-receiver-kind.md");
const TASK: &str = include_str!("../../../tasks/16-arrays-and-array-builtins.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

#[test]
fn array_reduce_direction_is_a_private_two_variant_domain() {
    let variants = bounded(
        ARRAY_SOURCE,
        "enum ArrayReduceDirection {",
        "\n}\n\nimpl ArrayReduceDirection",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();
    let projections = bounded(
        ARRAY_SOURCE,
        "impl ArrayReduceDirection {",
        "\n}\n\nenum TypedArrayQuantifierKind",
    );

    assert_eq!(variants, ["LeftToRight,", "RightToLeft,"]);
    assert!(!ARRAY_SOURCE.contains("pub enum ArrayReduceDirection"));
    assert!(!ARRAY_SOURCE.contains("pub(crate) enum ArrayReduceDirection"));
    assert!(!ARRAY_SOURCE.contains("pub(super) enum ArrayReduceDirection"));
    let declaration_offset = ARRAY_SOURCE
        .find("enum ArrayReduceDirection {")
        .expect("Array reduce direction declaration");
    assert_eq!(
        ARRAY_SOURCE[..declaration_offset]
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
            !ARRAY_SOURCE.contains(&format!("impl {capability} for ArrayReduceDirection")),
            "Array reduce direction must not implement {capability}"
        );
    }
    for projection in [
        "const fn method_name(&self,",
        "const fn typed_array_receiver_error(&self)",
        "const fn callback_not_callable_error(\n        &self,",
    ] {
        assert_eq!(
            projections.matches(projection).count(),
            1,
            "Array reduce direction must borrow through `{projection}`"
        );
    }
    for forbidden in [
        "const fn method_name(self,",
        "const fn typed_array_receiver_error(self)",
        "direction.clone()",
        "direction ==",
        "direction !=",
        "matches!(direction",
        "if direction",
    ] {
        assert!(
            !ARRAY_SOURCE.contains(forbidden),
            "Array reduce direction must not escape through `{forbidden}`"
        );
    }
    assert!(!projections.contains("Default"));
    assert!(!projections.contains("-> bool"));
    assert!(!projections.contains("=> true"));
    assert!(!projections.contains("=> false"));
    assert!(!projections.contains("_ =>"));
    assert!(!projections.contains("unreachable!"));
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("capability-free `ArrayReduceDirection`"));
    }
}

#[test]
fn reducer_exhaustively_projects_all_nine_direction_decisions() {
    let reducer = bounded(
        ARRAY_SOURCE,
        "    fn emit_array_reduce_loop_entry(",
        "    fn emit_array_reduce_has_property(",
    );
    let loop_entry = bounded(
        ARRAY_SOURCE,
        "    fn emit_array_reduce_loop_entry(",
        "    fn emit_array_reduce_advance(",
    );
    let advance = bounded(
        ARRAY_SOURCE,
        "    fn emit_array_reduce_advance(",
        "    fn compile_array_like_reduce_builtin(",
    );
    let compile_builtin = bounded(
        ARRAY_SOURCE,
        "    fn compile_array_like_reduce_builtin(",
        "    pub(super) fn compile_array_reduce_builtin(",
    );
    let signature = bounded(
        reducer,
        "    fn compile_array_like_reduce_builtin(",
        "    ) -> Result<(), EmitError> {",
    );

    assert!(signature.contains("direction: ArrayReduceDirection,"));
    assert_eq!(
        reducer.matches("direction: &ArrayReduceDirection,").count(),
        2
    );
    assert!(!signature.contains("bool"));
    assert!(!reducer.contains("reverse"));
    assert!(!reducer.contains("matches!(direction"));
    assert!(!reducer.contains("if direction"));
    assert!(!reducer.contains("direction =="));
    assert!(!reducer.contains("=> true"));
    assert!(!reducer.contains("=> false"));
    assert!(!reducer.contains("_ =>"));
    assert!(!reducer.contains("unreachable!"));

    assert_eq!(
        reducer
            .matches("direction.method_name(&receiver_kind)")
            .count(),
        1
    );
    assert_eq!(
        reducer
            .matches("direction.typed_array_receiver_error()")
            .count(),
        1
    );
    assert_eq!(
        reducer
            .matches("direction.callback_not_callable_error(&receiver_kind)")
            .count(),
        1
    );
    assert_eq!(reducer.matches("match direction {").count(), 2);
    assert_eq!(compile_builtin.matches("match &direction {").count(), 1);
    assert_eq!(loop_entry.matches("match direction {").count(), 1);
    assert_eq!(advance.matches("match direction {").count(), 1);
    assert_eq!(
        reducer
            .matches("self.emit_array_reduce_loop_entry(&direction,")
            .count(),
        2
    );
    assert_eq!(
        reducer
            .matches("self.emit_array_reduce_advance(&direction,")
            .count(),
        3
    );

    let direction_decisions = reducer
        .matches("direction.method_name(&receiver_kind)")
        .count()
        + reducer
            .matches("direction.typed_array_receiver_error()")
            .count()
        + reducer
            .matches("direction.callback_not_callable_error(&receiver_kind)")
            .count()
        + compile_builtin.matches("match &direction {").count()
        + reducer
            .matches("self.emit_array_reduce_loop_entry(&direction,")
            .count()
        + reducer
            .matches("self.emit_array_reduce_advance(&direction,")
            .count();
    assert_eq!(direction_decisions, 9);
}

#[test]
fn exactly_four_fixed_array_and_typed_array_entries_select_a_direction() {
    let producers = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypeMap => {",
        "            StandardBuiltinId::ArrayPrototypeEvery => {",
    );

    for (builtin, fixed_entry) in [
        (
            "StandardBuiltinId::ArrayPrototypeReduce => {",
            "self.compile_array_reduce_builtin(function)?;",
        ),
        (
            "StandardBuiltinId::ArrayPrototypeReduceRight => {",
            "self.compile_array_reduce_right_builtin(function)?;",
        ),
        (
            "StandardBuiltinId::TypedArrayPrototypeReduce => {",
            "self.compile_typed_array_reduce_builtin(function)?;",
        ),
        (
            "StandardBuiltinId::TypedArrayPrototypeReduceRight => {",
            "self.compile_typed_array_reduce_right_builtin(function)?;",
        ),
    ] {
        assert_eq!(producers.matches(builtin).count(), 1, "builtin `{builtin}`");
        assert_eq!(
            producers.matches(fixed_entry).count(),
            1,
            "entry `{fixed_entry}`"
        );
    }
    assert!(!producers.contains("ArrayReduceDirection"));
    assert!(!producers.contains("compile_array_like_reduce_builtin"));

    let fixed_entries = bounded(
        ARRAY_SOURCE,
        "    pub(super) fn compile_array_reduce_builtin(",
        "    fn emit_array_reduce_has_property(",
    );
    assert_eq!(
        fixed_entries
            .matches("self.compile_array_like_reduce_builtin(")
            .count(),
        4
    );
    for (start, end, receiver_kind, direction) in [
        (
            "    pub(super) fn compile_array_reduce_builtin(",
            "    pub(super) fn compile_array_reduce_right_builtin(",
            "ArrayCallbackReceiverKind::ArrayLike",
            "ArrayReduceDirection::LeftToRight",
        ),
        (
            "    pub(super) fn compile_array_reduce_right_builtin(",
            "    pub(super) fn compile_typed_array_reduce_builtin(",
            "ArrayCallbackReceiverKind::ArrayLike",
            "ArrayReduceDirection::RightToLeft",
        ),
        (
            "    pub(super) fn compile_typed_array_reduce_builtin(",
            "    pub(super) fn compile_typed_array_reduce_right_builtin(",
            "ArrayCallbackReceiverKind::TypedArray",
            "ArrayReduceDirection::LeftToRight",
        ),
        (
            "    pub(super) fn compile_typed_array_reduce_right_builtin(",
            "    fn emit_array_reduce_has_property(",
            "ArrayCallbackReceiverKind::TypedArray",
            "ArrayReduceDirection::RightToLeft",
        ),
    ] {
        let producer = bounded(ARRAY_SOURCE, start, end);
        assert!(
            producer.contains(receiver_kind),
            "producer `{start}` receiver"
        );
        assert!(producer.contains(direction), "producer `{start}` direction");
    }
}

#[test]
fn raw_reduce_direction_is_private_to_the_four_fixed_entries() {
    assert!(!STANDARD_SOURCE.contains("ArrayReduceDirection"));
    assert!(!STANDARD_SOURCE.contains("compile_array_like_reduce_builtin"));
    assert_eq!(
        ARRAY_SOURCE
            .matches("fn compile_array_like_reduce_builtin(")
            .count(),
        1
    );
    assert_eq!(
        ARRAY_SOURCE
            .matches("self.compile_array_like_reduce_builtin(")
            .count(),
        4
    );
}
