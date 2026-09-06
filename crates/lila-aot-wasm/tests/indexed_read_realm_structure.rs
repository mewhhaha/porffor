const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .expect("start boundary")
        .1
        .split_once(end)
        .expect("end boundary")
        .0
}

#[test]
fn indexed_get_forwards_only_the_trusted_read_realm_in_argument_six() {
    let seam = bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_typed_array_or_object_index_read_from_locals(",
        "pub(crate) fn compile_indexed_element_read_helper(",
    );
    let normalized: String = seam.chars().filter(|ch| !ch.is_whitespace()).collect();
    let call = concat!(
        "function.instruction(&Instruction::LocalGet(target_local));",
        "function.instruction(&Instruction::LocalGet(target_tag_local));",
        "function.instruction(&Instruction::LocalGet(index_local));",
        "function.instruction(&Instruction::I64Const(0));",
        "function.instruction(&Instruction::I64Const(0));",
        "function.instruction(&Instruction::I64Const(0));",
        "self.emit_outlined_object_read_realm_argument(function);",
        "function.instruction(&Instruction::Call(helper));",
    );
    assert_eq!(normalized.matches(call).count(), 1);
    assert_eq!(
        seam.matches("emit_outlined_object_read_realm_argument(")
            .count(),
        1
    );
    assert!(
        !seam.contains("LocalGet(self.current_env_local)"),
        "never pass an unclassified lexical environment"
    );
    assert!(seam.contains("emit_propagate_throw_from_locals_if_needed("));
}

#[test]
fn indexed_get_body_enters_the_classified_helper_domain_before_emission() {
    let body = bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn compile_indexed_element_read_helper(",
        "fn emit_typed_array_or_object_index_read_from_locals_inner(",
    );
    let classify = body
        .find("self.begin_helper_body(RuntimeHelperId::IndexedElementRead)")
        .unwrap();
    let read = body
        .find("self.emit_typed_array_or_object_index_read_from_locals_inner(")
        .unwrap();
    let normalized: String = body.chars().filter(|ch| !ch.is_whitespace()).collect();
    let binding = concat!(
        "function.instruction(&Instruction::LocalGet(6));",
        "function.instruction(&Instruction::LocalSet(self.current_env_local));",
    );
    assert_eq!(normalized.matches(binding).count(), 1);
    let bind = body
        .find("function.instruction(&Instruction::LocalSet(self.current_env_local))")
        .unwrap();
    assert!(classify < bind && bind < read);
    assert_eq!(body.matches("LocalSet(self.current_env_local)").count(), 1);
    assert_eq!(body.matches("begin_helper_body(").count(), 1);
    assert!(!body.contains("current_env_local ="));
}
