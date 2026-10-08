const VALUE: &str = include_str!("../src/builtins/json/module_value.rs");
const OWNER: &str = include_str!("../../lila-ir/src/modules/json.rs");
const SOURCE: &str = include_str!("../../lila-ir/src/modules/synchronous_source.rs");
const DEFINITION: &str = include_str!("../../lila-ir/src/modules/synchronous_definition.rs");
const REALM: &str = include_str!("../src/modules/synchronous.rs");
const EXPRESSION: &str = include_str!("../src/expressions.rs");

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn native_json_evaluation_requires_the_actual_record_and_original_allocation_algorithms() {
    let owner = compact(OWNER);
    let source = compact(SOURCE);
    let definition = compact(DEFINITION);
    let expression = compact(EXPRESSION);
    let realm = compact(REALM);
    assert!(owner.contains("pub(super)fnnew("));
    assert!(!owner.contains("pubfnnew("));
    assert!(source.contains("unit.record.json_source()"));
    assert!(source.contains("JsonModuleValueIr::new(module,source.clone())"));
    assert!(definition.contains("module_execution.json_values.insert("));
    assert!(expression.contains(
        "ExprIr::JsonModuleValue(plan)=>self.emit_json_module_value(plan,output,function)?"
    ));
    assert!(realm.contains("owner.protocol()==FunctionProtocolIr::ModuleActivation"));
    assert!(realm.contains("field(ModuleRecordSchema::REALM)"));
    for operation in [
        "emit_json_plain_object",
        "emit_alloc_array_payload_with_length_and_prototype",
        "emit_json_define",
        "encode_js_string_utf16",
    ] {
        assert!(VALUE.contains(operation));
    }
    assert!(!VALUE.contains("emit_json_parse"));
    assert!(!VALUE.contains("JSON.parse"));
    assert!(!VALUE.contains("emit_user_function"));
    assert!(!VALUE.contains("load_current_realm"));
}
