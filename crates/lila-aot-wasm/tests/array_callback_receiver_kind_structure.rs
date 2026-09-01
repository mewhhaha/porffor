const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/array-callback-receiver-kind.md");
const TASK_T02: &str = include_str!("../../../tasks/02-modularize-ir-and-wasm-backend.md");
const TASK_T16: &str = include_str!("../../../tasks/16-arrays-and-array-builtins.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn reducer_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "    fn compile_array_like_reduce_builtin(",
        "    pub(super) fn compile_array_reduce_builtin(",
    )
}

fn for_each_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "    fn compile_array_like_for_each_builtin(",
        "    pub(crate) fn emit_alloc_array_payload_with_length(",
    )
}

#[test]
fn callback_receiver_kind_is_a_capability_free_two_variant_authority() {
    let variants = bounded(
        ARRAY_SOURCE,
        "enum ArrayCallbackReceiverKind {",
        "\n}\n\nenum ArrayReduceDirection",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();
    assert_eq!(variants, ["ArrayLike,", "TypedArray,"]);
    let declaration_offset = ARRAY_SOURCE
        .find("enum ArrayCallbackReceiverKind {")
        .expect("Array callback receiver kind declaration");
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
            !ARRAY_SOURCE.contains(&format!("impl {capability} for ArrayCallbackReceiverKind")),
            "Array callback receiver kind must not implement {capability}"
        );
    }
    assert!(!ARRAY_SOURCE.contains("pub(crate) enum ArrayCallbackReceiverKind"));
    assert!(!ARRAY_SOURCE.contains("pub(super) enum ArrayCallbackReceiverKind"));

    let reducer = reducer_body();
    let for_each = for_each_body();
    for forbidden in [
        "typed_array_only",
        "receiver_kind ==",
        "receiver_kind !=",
        "matches!(receiver_kind",
        "receiver_kind.is_",
        "receiver_kind.clone()",
        "=> true",
        "=> false",
    ] {
        assert!(
            !reducer.contains(forbidden) && !for_each.contains(forbidden),
            "callback receiver semantics must not collapse to `{forbidden}`"
        );
    }
    for evidence in [CONTRACT, TASK_T02, TASK_T16] {
        assert!(evidence.contains("capability-free `ArrayCallbackReceiverKind`"));
        assert!(
            evidence.contains("c073b0a9449fae68b12f82e43fc0bf7dc52a0a0bc98b1a6eb2bf6d5b0bce3ea1")
        );
        assert!(
            evidence.contains("ea047de76bef8b4c5fbc8eb440c42329e7693feecf848cef753011cf2a541c26")
        );
        assert!(evidence.contains("no new Array behavior"));
    }
    assert!(CONTRACT.contains("two fixed forEach entries"));
    assert!(CONTRACT.contains("does not close T16"));
}

#[test]
fn all_thirteen_callback_receiver_decisions_are_exhaustive() {
    let direction_projections = bounded(
        ARRAY_SOURCE,
        "impl ArrayReduceDirection {",
        "\n}\n\nenum TypedArrayQuantifierKind",
    );
    let reducer = reducer_body();
    let for_each = for_each_body();

    assert_eq!(
        direction_projections
            .matches("receiver_kind: &ArrayCallbackReceiverKind")
            .count(),
        2
    );
    assert_eq!(
        reducer
            .matches("receiver_kind: ArrayCallbackReceiverKind,")
            .count(),
        1
    );
    assert_eq!(
        for_each
            .matches("receiver_kind: ArrayCallbackReceiverKind,")
            .count(),
        1
    );
    assert_eq!(
        direction_projections
            .matches("match (receiver_kind, self) {")
            .count(),
        2
    );
    assert_eq!(reducer.matches("match &receiver_kind {").count(), 6);
    assert_eq!(for_each.matches("match &receiver_kind {").count(), 5);
    assert_eq!(
        reducer
            .matches("direction.method_name(&receiver_kind)")
            .count(),
        1
    );
    assert_eq!(
        reducer
            .matches("direction.callback_not_callable_error(&receiver_kind)")
            .count(),
        1
    );
    assert_eq!(
        direction_projections
            .matches("match (receiver_kind, self) {")
            .count()
            + reducer.matches("match &receiver_kind {").count()
            + for_each.matches("match &receiver_kind {").count(),
        13
    );
    assert!(!direction_projections.contains("receiver_kind: ArrayCallbackReceiverKind"));
    assert!(!reducer.contains("match receiver_kind {"));
    assert!(!for_each.contains("match receiver_kind {"));

    assert_eq!(
        reducer
            .matches("ArrayCallbackReceiverKind::ArrayLike =>")
            .count(),
        6
    );
    assert_eq!(
        reducer
            .matches("ArrayCallbackReceiverKind::TypedArray =>")
            .count(),
        6
    );
    assert_eq!(
        for_each
            .matches("ArrayCallbackReceiverKind::ArrayLike =>")
            .count(),
        5
    );
    assert_eq!(
        for_each
            .matches("ArrayCallbackReceiverKind::TypedArray =>")
            .count(),
        5
    );
    assert!(!direction_projections.contains("_ =>"));
    assert!(!reducer.contains("_ =>"));
    assert!(!for_each.contains("_ =>"));
}

#[test]
fn six_fixed_entries_select_callback_receiver_kinds_inside_array_owner() {
    let producers = bounded(
        STANDARD_SOURCE,
        "            StandardBuiltinId::ArrayPrototypeMap => {",
        "            StandardBuiltinId::ArrayPrototypeFilter => {",
    );
    assert_eq!(
        producers
            .matches("self.compile_array_prototype_for_each_builtin(function)?;")
            .count(),
        1
    );
    assert_eq!(
        producers
            .matches("self.compile_typed_array_prototype_for_each_builtin(function)?;")
            .count(),
        1
    );
    for fixed_entry in [
        "self.compile_array_reduce_builtin(function)?;",
        "self.compile_array_reduce_right_builtin(function)?;",
        "self.compile_typed_array_reduce_builtin(function)?;",
        "self.compile_typed_array_reduce_right_builtin(function)?;",
    ] {
        assert_eq!(producers.matches(fixed_entry).count(), 1, "{fixed_entry}");
    }
    assert!(!producers.contains("ArrayReduceDirection"));
    assert!(!producers.contains("ArrayCallbackReceiverKind"));
    assert!(!producers.contains("compile_array_like_for_each_builtin("));

    for (start, end, receiver_kind) in [
        (
            "    pub(super) fn compile_array_reduce_builtin(",
            "    pub(super) fn compile_array_reduce_right_builtin(",
            "ArrayCallbackReceiverKind::ArrayLike",
        ),
        (
            "    pub(super) fn compile_array_reduce_right_builtin(",
            "    pub(super) fn compile_typed_array_reduce_builtin(",
            "ArrayCallbackReceiverKind::ArrayLike",
        ),
        (
            "    pub(super) fn compile_typed_array_reduce_builtin(",
            "    pub(super) fn compile_typed_array_reduce_right_builtin(",
            "ArrayCallbackReceiverKind::TypedArray",
        ),
        (
            "    pub(super) fn compile_typed_array_reduce_right_builtin(",
            "    fn emit_array_reduce_has_property(",
            "ArrayCallbackReceiverKind::TypedArray",
        ),
    ] {
        let producer = bounded(ARRAY_SOURCE, start, end);
        let normalized = without_whitespace(producer);
        assert_eq!(normalized.matches(receiver_kind).count(), 1, "{start}");
        assert_eq!(
            normalized
                .matches("compile_array_like_reduce_builtin(")
                .count(),
            1,
            "{start}"
        );
    }

    for (start, end, receiver_kind) in [
        (
            "    pub(super) fn compile_array_prototype_for_each_builtin(",
            "    pub(super) fn compile_typed_array_prototype_for_each_builtin(",
            "ArrayCallbackReceiverKind::ArrayLike",
        ),
        (
            "    pub(super) fn compile_typed_array_prototype_for_each_builtin(",
            "    fn compile_array_like_for_each_builtin(",
            "ArrayCallbackReceiverKind::TypedArray",
        ),
    ] {
        let producer = bounded(ARRAY_SOURCE, start, end);
        assert_eq!(producer.matches(receiver_kind).count(), 1, "{start}");
        assert_eq!(
            producer
                .matches("compile_array_like_for_each_builtin(")
                .count(),
            1,
            "{start}"
        );
    }
}

#[test]
fn callback_receiver_projection_preserves_typed_array_witnesses() {
    let reducer = reducer_body();
    let reducer_property = bounded(
        ARRAY_SOURCE,
        "    fn emit_array_reduce_has_property(",
        "    fn emit_array_reduce_get_index(",
    );
    let for_each = for_each_body();

    assert_eq!(
        reducer
            .matches("TypedArrayWitnessUse::ValidatedMethodEntry")
            .count(),
        1
    );
    assert_eq!(
        reducer
            .matches("self.emit_array_reduce_has_property(")
            .count(),
        2
    );
    assert_eq!(
        reducer_property
            .matches("TypedArrayWitnessUse::IntegerIndexedProperty")
            .count(),
        1
    );
    assert_eq!(
        for_each
            .matches("TypedArrayWitnessUse::ValidatedMethodEntry")
            .count(),
        1
    );
    assert_eq!(
        for_each
            .matches("TypedArrayWitnessUse::IntegerIndexedProperty")
            .count(),
        1
    );
}
