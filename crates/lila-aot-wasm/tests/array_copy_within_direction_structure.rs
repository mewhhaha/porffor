const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const DIRECTION_SOURCE: &str = include_str!("../src/builtins/array/copy_within.rs");

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

fn copy_within_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "    pub(crate) fn compile_array_prototype_copy_within_builtin(",
        "    #[allow(clippy::too_many_arguments)]\n    pub(crate) fn emit_array_at_from_locals(",
    )
}

#[test]
fn copy_within_direction_is_a_capability_free_closed_domain() {
    let normalized = without_whitespace(DIRECTION_SOURCE);
    assert!(normalized.contains("pub(super)enumArrayCopyWithinDirection{Forward,Backward,}"));
    assert_eq!(normalized.matches("matchdirection{").count(), 1);
    assert_eq!(
        normalized
            .matches("ArrayCopyWithinDirection::Forward=>{")
            .count(),
        1
    );
    assert_eq!(
        normalized
            .matches("ArrayCopyWithinDirection::Backward=>{")
            .count(),
        1
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
            !normalized.contains(&format!("derive({capability}")),
            "copyWithin direction must not derive {capability}"
        );
        assert!(
            !normalized.contains(&format!("impl{capability}forArrayCopyWithinDirection")),
            "copyWithin direction must not implement {capability}"
        );
    }
    for forbidden in ["_=>", "==", "!=", "matches!(", "unreachable!"] {
        assert!(
            !normalized.contains(forbidden),
            "copyWithin direction must not use {forbidden}"
        );
    }
}

#[test]
fn one_projection_owns_cursor_start_and_step() {
    let forward = bounded(
        DIRECTION_SOURCE,
        "            ArrayCopyWithinDirection::Forward => {",
        "            ArrayCopyWithinDirection::Backward => {",
    );
    let backward = bounded(
        DIRECTION_SOURCE,
        "            ArrayCopyWithinDirection::Backward => {",
        "            }\n        }",
    );
    let normalized_forward = without_whitespace(forward);
    let normalized_backward = without_whitespace(backward);

    assert_eq!(
        normalized_forward,
        "function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::LocalSet(direction_local));}"
    );
    assert_eq!(
        normalized_backward,
        "function.instruction(&Instruction::LocalGet(from_local));function.instruction(&Instruction::LocalGet(count_local));function.instruction(&Instruction::I64Add);function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::I64Sub);function.instruction(&Instruction::LocalSet(from_local));function.instruction(&Instruction::LocalGet(to_local));function.instruction(&Instruction::LocalGet(count_local));function.instruction(&Instruction::I64Add);function.instruction(&Instruction::I64Const(1));function.instruction(&Instruction::I64Sub);function.instruction(&Instruction::LocalSet(to_local));function.instruction(&Instruction::I64Const(-1));function.instruction(&Instruction::LocalSet(direction_local));"
    );

    assert_eq!(
        DIRECTION_SOURCE
            .matches("LocalSet(direction_local)")
            .count(),
        2
    );
    assert_eq!(DIRECTION_SOURCE.matches("LocalSet(from_local)").count(), 1);
    assert_eq!(DIRECTION_SOURCE.matches("LocalSet(to_local)").count(), 1);
}

#[test]
fn forward_and_overlap_backward_are_the_only_producers() {
    let body = copy_within_body();
    let normalized = without_whitespace(body);
    let forward_call = "self.emit_array_copy_within_traversal_start(ArrayCopyWithinDirection::Forward,from_local,to_local,count_local,direction_local,function,);";
    let backward_call = "self.emit_array_copy_within_traversal_start(ArrayCopyWithinDirection::Backward,from_local,to_local,count_local,direction_local,function,);";

    assert_eq!(normalized.matches(forward_call).count(), 1);
    assert_eq!(normalized.matches(backward_call).count(), 1);
    assert_eq!(body.matches("LocalSet(direction_local)").count(), 0);
    assert_eq!(
        body.matches("let direction_local = self.reserve_temp_local();")
            .count(),
        1
    );
    assert_eq!(
        body.matches("self.release_temp_local(direction_local);")
            .count(),
        1
    );

    let forward_position = normalized.find(forward_call).expect("forward producer");
    let backward_position = normalized.find(backward_call).expect("backward producer");
    let overlap_guard = "function.instruction(&Instruction::LocalGet(from_local));function.instruction(&Instruction::LocalGet(to_local));function.instruction(&Instruction::I64LtU);function.instruction(&Instruction::If(BlockType::Empty));function.instruction(&Instruction::LocalGet(to_local));function.instruction(&Instruction::LocalGet(from_local));function.instruction(&Instruction::LocalGet(count_local));function.instruction(&Instruction::I64Add);function.instruction(&Instruction::I64LtU);function.instruction(&Instruction::If(BlockType::Empty));";
    let direction_selection = format!(
        "{forward_call}{overlap_guard}{backward_call}function.instruction(&Instruction::End);function.instruction(&Instruction::End);"
    );
    let overlap_position = normalized.find(overlap_guard).expect("overlap guard");
    assert!(forward_position < overlap_position);
    assert!(overlap_position < backward_position);
    assert_eq!(normalized.matches(&direction_selection).count(), 1);

    assert_eq!(
        ARRAY_SOURCE
            .matches("ArrayCopyWithinDirection::Forward")
            .count(),
        1
    );
    assert_eq!(
        ARRAY_SOURCE
            .matches("ArrayCopyWithinDirection::Backward")
            .count(),
        1
    );
}
