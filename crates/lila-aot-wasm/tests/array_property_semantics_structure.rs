const DESCRIPTOR_SOURCE: &str =
    include_str!("../src/builtins/object/get_own_property_descriptor.rs");
const OBJECT_SOURCE: &str = include_str!("../src/objects.rs");

fn normalize(source: &str) -> String {
    source.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn virtual_string_keys_use_property_key_equality_not_payload_identity() {
    let source = normalize(DESCRIPTOR_SOURCE);
    for (key, count) in [("length", 4), ("callee", 1), ("prototype", 2)] {
        let wiring = format!(
            "function.instruction(&Instruction::I64Const(self.strings.payload(\"{key}\")));\
             function.instruction(&Instruction::LocalSet(key_constant_local));\
             self.emit_property_key_payload_equality_i32(\
             key_string_local,key_constant_local,function);"
        );
        assert_eq!(source.matches(&wiring).count(), count, "{key}");
        let pointer_comparison = format!(
            "function.instruction(&Instruction::LocalGet(key_string_local));\
             function.instruction(&Instruction::I64Const(self.strings.payload(\"{key}\")));\
             function.instruction(&Instruction::I64Eq);"
        );
        assert!(!source.contains(&pointer_comparison), "{key}");
    }
    assert_eq!(
        source
            .matches("letkey_constant_local=self.reserve_temp_local();")
            .count(),
        1
    );
    assert_eq!(
        source
            .matches("self.release_temp_local(key_constant_local);")
            .count(),
        1
    );
    assert!(source.contains(concat!(
        "self.release_temp_local(key_constant_local);",
        "self.release_temp_local(proxy_target_extensible_local);"
    )));
}

#[test]
fn ordinary_indexed_setters_share_proxy_aware_call_and_abrupt_propagation() {
    let owner = OBJECT_SOURCE
        .split_once("    pub(crate) fn emit_ordinary_set_result_with_receiver_fallback(")
        .expect("OrdinarySet owner")
        .1;
    let setter = owner
        .split_once("        self.emit_array_accessor_setter_for_index(")
        .expect("indexed setter branch")
        .1;
    let setter = setter
        .split_once("        function.instruction(&Instruction::LocalSet(result_local));")
        .expect("normal-success publication")
        .0;
    let setter = normalize(setter);
    assert!(!setter.contains("self.emit_function_handle_call("));
    let call = setter
        .find(concat!(
            "self.emit_function_or_proxy_call_leave_throw_completion(",
            "setter_payload_local,setter_tag_local,receiver_payload_local,receiver_tag_local,",
            "&[(value_payload_local,value_tag_local)],setter_result_payload_local,",
            "setter_result_tag_local,function,)?;"
        ))
        .expect("Proxy-aware Call retains the explicit Receiver and assigned value");
    let abrupt = setter
        .find(concat!(
            "self.emit_propagate_throw_from_locals_if_needed(",
            "setter_result_payload_local,setter_result_tag_local,function,)?;"
        ))
        .expect("setter abrupt completion is preserved");
    let success = setter
        .rfind("function.instruction(&Instruction::I64Const(1));")
        .expect("success value");
    assert!(call < abrupt && abrupt < success);
}
