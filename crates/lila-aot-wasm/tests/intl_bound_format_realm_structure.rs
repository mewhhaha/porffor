const FUNCTIONS: &str = include_str!("../src/functions.rs");
const CLOSURE: &str = include_str!("../src/functions/current_builtin_realm_closure.rs");
const RENDERER: &str = include_str!("../src/builtins/intl_datetimeformat/provider_render.rs");
const FORMATTER: &str = include_str!("../src/builtins/intl_datetimeformat.rs");

#[test]
fn escaping_builtin_closure_keeps_capture_and_realm_identity_separate() {
    assert!(FUNCTIONS.contains("mod current_builtin_realm_closure;"));
    assert!(CLOSURE.contains("RealmFunctionMaterializationContext {"));
    assert!(CLOSURE.contains("HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET"));
    assert!(CLOSURE.contains("self.emit_function_value_payload_in_realm("));
    assert!(!CLOSURE.contains("HEAP_PROTOTYPE_OFFSET"));
    let compact = CLOSURE.split_whitespace().collect::<String>();
    assert!(compact.contains(
        "function_object_local,HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET,capture_local,"
    ));
    assert!(compact
        .contains("function_object_local,HEAP_FUNCTION_ENV_HANDLE_OFFSET,function_object_local,"));
    assert!(compact.contains("self.release_realm_function_materialization_context(context);self.release_temp_local(realm_local);"));
}

#[test]
fn format_cache_materializes_in_getter_realm_and_reads_canonical_capture() {
    let getter = FORMATTER
        .split_once("pub(crate) fn emit_intl_date_time_format_format_getter(")
        .expect("format getter")
        .1
        .split_once("fn emit_dtf_if_code_eq(")
        .expect("next formatter implementation")
        .0;
    assert!(getter.contains("self.emit_current_builtin_realm_closure_value("));
    assert!(!getter.contains("self.emit_function_value_payload("));
    assert!(!getter.contains("HEAP_FUNCTION_ENV_HANDLE_OFFSET"));
    let cached_read = getter.find("HEAP_INTL_DTF_BOUND_FORMAT_OFFSET").unwrap();
    let materialize = getter
        .find("self.emit_current_builtin_realm_closure_value(")
        .unwrap();
    let cached_write = getter.rfind("HEAP_INTL_DTF_BOUND_FORMAT_OFFSET").unwrap();
    assert!(cached_read < materialize && materialize < cached_write);

    let body = RENDERER
        .split_once("pub(crate) fn emit_intl_date_time_format_bound_format(")
        .expect("bound format body")
        .1
        .split_once("pub(crate) fn emit_intl_date_time_format_format_to_parts(")
        .expect("next formatter body")
        .0;
    assert!(body.contains("HEAP_FUNCTION_BUILTIN_CLOSURE_CONTEXT_OFFSET"));
    assert!(!body.contains("Instruction::LocalGet(self.current_env_local)"));
}
