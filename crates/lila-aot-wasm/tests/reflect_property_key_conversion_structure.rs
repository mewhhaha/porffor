const REFLECT_SOURCE: &str = include_str!("../src/builtins/reflect.rs");
const CLI_OBJECT_SOURCE: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const FIXTURE_SOURCE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_reflect_property_key_conversion.js");
const MODULE_BOUNDARY_SOURCE: &str = include_str!("../../../scripts/check-module-boundaries.sh");
const CONTRACT_SOURCE: &str =
    include_str!("../../../docs/rust-rewrite/contracts/reflect-property-key-conversion.md");
const TASK_SOURCE: &str = include_str!("../../../tasks/11-proxy-reflect-metaobject.md");

fn anchored_offsets(source: &str, declaration: &str) -> Vec<usize> {
    source
        .match_indices(declaration)
        .filter_map(|(offset, _)| {
            let line_start = source[..offset]
                .rfind('\n')
                .map_or(0, |newline| newline + 1);
            source[line_start..offset]
                .chars()
                .all(char::is_whitespace)
                .then_some(offset)
        })
        .collect()
}

fn braced_rust_function<'a>(source: &'a str, declaration: &str) -> &'a str {
    let offsets = anchored_offsets(source, declaration);
    assert_eq!(offsets.len(), 1, "exact Rust owner `{declaration}`");
    let start = offsets[0];
    let mut depth = 0;
    let mut body_started = false;
    for (relative_offset, character) in source[start..].char_indices() {
        match character {
            '{' => {
                depth += 1;
                body_started = true;
            }
            '}' => {
                depth -= 1;
                if body_started && depth == 0 {
                    return &source[start..start + relative_offset + character.len_utf8()];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated Rust owner `{declaration}`");
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn reflect_owner(declaration: &str) -> &'static str {
    braced_rust_function(REFLECT_SOURCE, declaration)
}

#[test]
fn five_reflect_key_boundaries_use_the_full_in_place_conversion() {
    for (declaration, target_error) in [
        (
            "pub(crate) fn compile_reflect_get_builtin(",
            "Reflect.get target must be object",
        ),
        (
            "pub(crate) fn compile_reflect_set_builtin(",
            "Reflect.set target must be object",
        ),
        (
            "pub(crate) fn compile_reflect_has_builtin(",
            "Reflect.has target must be object",
        ),
        (
            "pub(crate) fn compile_reflect_define_property_builtin(",
            "Reflect.defineProperty target must be object",
        ),
        (
            "pub(crate) fn compile_reflect_delete_property_builtin(",
            "Reflect.deleteProperty target must be object",
        ),
    ] {
        let owner = reflect_owner(declaration);
        assert_eq!(
            owner
                .matches("self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;")
                .count(),
            1,
            "one full ToPropertyKey conversion in `{declaration}`"
        );
        assert!(
            !owner.contains("emit_value_to_property_key_payload("),
            "`{declaration}` must not discard the converted tag"
        );
        assert_before(
            owner,
            target_error,
            "self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;",
        );
    }

    assert_eq!(
        REFLECT_SOURCE
            .matches("emit_value_to_property_key_locals(")
            .count(),
        5
    );
    assert_eq!(
        REFLECT_SOURCE
            .matches("emit_value_to_property_key_payload(")
            .count(),
        0
    );
}

#[test]
fn converted_symbol_tag_reaches_tagged_consumers() {
    let get = reflect_owner("pub(crate) fn compile_reflect_get_builtin(");
    assert!(get.contains(
        "function.instruction(&Instruction::LocalGet(key_payload_local));\n        function.instruction(&Instruction::LocalSet(key_string_local));"
    ));

    let set = reflect_owner("pub(crate) fn compile_reflect_set_builtin(");
    assert!(set.contains(
        "function.instruction(&Instruction::LocalGet(key_tag_local));\n        function.instruction(&Instruction::LocalSet(key_property_tag_local));"
    ));
    assert!(!set.contains("emit_property_key_tag_from_source_tag("));
    assert_before(
        set,
        "self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;",
        "self.emit_property_key_value_payload_to_local(",
    );

    let has = reflect_owner("pub(crate) fn compile_reflect_has_builtin(");
    assert!(has.contains(
        "self.emit_object_has_property_with_key_tag_i32(\n            target_payload_local,\n            target_tag_local,\n            key_string_local,\n            key_tag_local,"
    ));
    assert!(!has.contains("emit_property_key_tag_from_source_tag("));

    let define = reflect_owner("pub(crate) fn compile_reflect_define_property_builtin(");
    assert!(define.contains(
        "function.instruction(&Instruction::LocalGet(key_tag_local));\n        function.instruction(&Instruction::LocalSet(proxy_key_tag_local));"
    ));
    assert_before(
        define,
        "function.instruction(&Instruction::LocalSet(proxy_key_tag_local));",
        "self.emit_is_heap_object_like_tag_i32(descriptor_tag_local, function);",
    );

    let delete = reflect_owner("pub(crate) fn compile_reflect_delete_property_builtin(");
    assert_before(
        delete,
        "self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;",
        "self.emit_object_delete(",
    );
}

#[test]
fn runtime_fixture_observes_exact_symbols_and_abrupt_conversion() {
    for cli_marker in [
        "fn run_wasm_backend_preserves_reflect_property_key_conversion()",
        "\"wasm_reflect_property_key_conversion.js\"",
    ] {
        assert!(
            CLI_OBJECT_SOURCE.contains(cli_marker),
            "CLI marker `{cli_marker}`"
        );
    }
    for fixture_marker in [
        "Reflect.get(getTarget, Object(getSymbol))",
        "Reflect.get(getTarget, symbolCoercible(getSymbol))",
        "Reflect.has(hasTarget, symbolCoercible(hasSymbol))",
        "Reflect.defineProperty(defineTarget, symbolCoercible(defineSymbol)",
        "Reflect.deleteProperty(deleteTarget, symbolCoercible(deleteSymbol))",
        "Reflect.set(setProxy, symbolCoercible(setSymbol), 45, setReceiver)",
        "assert(key === setSymbol, \"Reflect.set exact converted Symbol\");",
        "assert(target === setTarget, \"Reflect.set exact Function target\");",
        "assert(typeof target === \"function\", \"Reflect.set Function target tag\");",
        "assert(receiver === setReceiver, \"Reflect.set exact Array receiver\");",
        "assert(Array.isArray(receiver), \"Reflect.set Array receiver tag\");",
        "assert(symbolKeyConversions === 5, \"Reflect.set conversion count\");",
        "assert(setTrapCalls === 1, \"Reflect.set trap count\");",
        "assert(abruptKeyConversions === 5, \"abrupt key conversion count\");",
        "assert(abruptTrapCalls === 0, \"abrupt key reached target internal method\");",
    ] {
        assert!(
            FIXTURE_SOURCE.contains(fixture_marker),
            "fixture marker `{fixture_marker}`"
        );
    }
    assert_eq!(FIXTURE_SOURCE.matches("=== abruptSentinel").count(), 5);
    assert!(FIXTURE_SOURCE.trim_end().ends_with("true;"));
}

#[test]
fn module_guard_contract_and_task_pin_the_five_boundaries() {
    for marker in [
        "'five Reflect full ToPropertyKey consumers'",
        "'legacy payload-only Reflect ToPropertyKey consumers'",
        "run_wasm_backend_preserves_reflect_property_key_conversion",
        "'Reflect property-key conversion fixture wiring'",
    ] {
        assert!(
            MODULE_BOUNDARY_SOURCE.contains(marker),
            "module-boundary marker `{marker}`"
        );
    }
    for marker in [
        "Reflect property-key conversion",
        "converted payload and tag",
        "wasm_reflect_property_key_conversion.js",
        "Verification pending",
    ] {
        assert!(
            CONTRACT_SOURCE.contains(marker),
            "contract marker `{marker}`"
        );
    }
    for marker in [
        "five Reflect property-key boundaries",
        "reflect_property_key_conversion_structure",
        "wasm_reflect_property_key_conversion.js",
    ] {
        assert!(TASK_SOURCE.contains(marker), "T11 marker `{marker}`");
    }
}
