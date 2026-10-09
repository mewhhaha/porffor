const OBJECTS: &str = include_str!("../../src/objects.rs");
const HELPERS: &str = include_str!("../../src/runtime_helpers.rs");
const EMIT: &str = include_str!("../../src/emit.rs");
fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing {end}"))
        .0
}
fn normalized(source: &str) -> String {
    source.chars().filter(|c| !c.is_whitespace()).collect()
}
#[test]
fn indexed_get_forwards_one_typed_environment_and_whole_completion() {
    assert!(HELPERS.contains(r#"IndexedElementRead / IndexedElementReadArguments / IndexedElementReadParameters / "indexed_element_read" { target:Value,index:I64,caller_environment:(Ref Environment Nullable) } => Completion;"#));
    let seam = normalized(bounded(
        OBJECTS,
        "pub(crate) fn emit_typed_array_or_object_index_read_from_locals(",
        "pub(crate) fn compile_indexed_element_read_helper(",
    ));
    assert_eq!(
        seam.matches("IndexedElementReadArguments::new(target,index,self.current_environment(),)")
            .count(),
        1
    );
    assert!(seam.contains(".store(result,function)"));
    assert!(seam.contains("result:&CompletionLocals"));
    assert!(!seam.contains("Instruction::Call("));
    assert!(!seam.contains("GlobalGet"));
}
#[test]
fn indexed_get_enters_its_typed_helper_domain_before_property_emission() {
    let body = bounded(
        OBJECTS,
        "pub(crate) fn compile_indexed_element_read_helper(",
        "pub(crate) fn emit_typed_array_or_object_index_write_from_locals(",
    );
    let start = body
        .find("self.begin_helper_body(RuntimeHelperId::IndexedElementRead)")
        .unwrap();
    let bind = body
        .find("helper_parameters::<crate::runtime_helpers::IndexedElementReadParameters>")
        .unwrap();
    let read = body
        .find("self.emit_object_read_with_throw_routing(")
        .unwrap();
    let emit = body.find("result.emit(&mut function)").unwrap();
    assert!(start < bind && bind < read && read < emit);
    assert!(body.contains("AccessorThrowRouting::LeaveInCompletion"));
    assert_eq!(body.matches("&parameters.target,").count(), 2);
    let parameters = normalized(bounded(
        EMIT,
        "pub(crate) fn helper_parameters<P:",
        "\n    }",
    ));
    assert!(parameters.contains("parameters.caller_environment()"));
    assert!(parameters.contains(
        "self.current_environment.replace(environment.load(self.schema,function),function)"
    ));
}
