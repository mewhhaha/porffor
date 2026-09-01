const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const REFLECT_SOURCE: &str = include_str!("../src/builtins/reflect.rs");
const CLI_OBJECT_SOURCE: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const FIXTURE_SOURCE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_reflect_optional_argument_presence.js");
const MODULE_BOUNDARY_SOURCE: &str = include_str!("../../../scripts/check-module-boundaries.sh");
const CONTRACT_SOURCE: &str =
    include_str!("../../../docs/rust-rewrite/contracts/reflect-optional-argument-presence.md");
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
fn builtin_argument_presence_has_one_argc_authority() {
    let presence = braced_rust_function(
        FUNCTIONS_SOURCE,
        "pub(crate) fn emit_builtin_arg_is_present_i32(",
    );
    for instruction in [
        "Instruction::LocalGet(self.argc_param_local())",
        "Instruction::I64Const(index as i64)",
        "Instruction::I64GtU",
    ] {
        assert_eq!(
            presence.matches(instruction).count(),
            1,
            "presence instruction `{instruction}`"
        );
    }
    for forbidden in [
        "argv_param_local",
        "ValueKind::Undefined",
        "emit_array_read",
    ] {
        assert!(
            !presence.contains(forbidden),
            "presence must not inspect argument value through `{forbidden}`"
        );
    }

    let loader = braced_rust_function(
        FUNCTIONS_SOURCE,
        "pub(crate) fn emit_builtin_arg_to_locals(",
    );
    assert_eq!(
        loader
            .matches("self.emit_builtin_arg_is_present_i32(index, function);")
            .count(),
        1
    );
    assert!(!loader.contains("argc_param_local"));
    assert!(!loader.contains("Instruction::I64GtU"));
    assert_eq!(
        FUNCTIONS_SOURCE
            .matches("emit_builtin_arg_is_present_i32(")
            .count(),
        2
    );
    assert_eq!(
        REFLECT_SOURCE
            .matches("emit_builtin_arg_is_present_i32(")
            .count(),
        3
    );
}

#[test]
fn reflect_construct_defaults_new_target_only_when_index_two_is_absent() {
    let owner = reflect_owner("pub(crate) fn compile_reflect_construct_builtin(");

    assert_eq!(
        owner
            .matches("self.emit_builtin_arg_is_present_i32(2, function);")
            .count(),
        1
    );
    assert!(!owner.contains("ValueKind::Undefined.tag()"));
    assert!(owner.contains(
        "self.emit_builtin_arg_is_present_i32(2, function);\n        function.instruction(&Instruction::I32Eqz);\n        function.instruction(&Instruction::If(BlockType::Empty));\n        function.instruction(&Instruction::LocalGet(target_payload_local));\n        function.instruction(&Instruction::LocalSet(new_target_payload_local));\n        function.instruction(&Instruction::LocalGet(target_tag_local));\n        function.instruction(&Instruction::LocalSet(new_target_tag_local));"
    ));
    assert_before(
        owner,
        "self.emit_builtin_arg_to_locals(\n            2,\n            new_target_payload_local,",
        "self.emit_is_constructor_i32(target_tag_local, target_payload_local, function)?;",
    );
    assert_before(
        owner,
        "self.emit_is_constructor_i32(target_tag_local, target_payload_local, function)?;",
        "self.emit_builtin_arg_is_present_i32(2, function);",
    );
    assert_before(
        owner,
        "Reflect.construct target is not a constructor",
        "self.emit_builtin_arg_is_present_i32(2, function);",
    );
    assert_before(
        owner,
        "self.emit_builtin_arg_is_present_i32(2, function);",
        "self.emit_is_constructor_i32(new_target_tag_local, new_target_payload_local, function)?;",
    );
}

#[test]
fn reflect_get_and_set_default_receivers_after_property_key_conversion() {
    let get = reflect_owner("pub(crate) fn compile_reflect_get_builtin(");
    assert_eq!(
        get.matches("self.emit_builtin_arg_is_present_i32(2, function);")
            .count(),
        1
    );
    assert!(!get.contains("ValueKind::Undefined.tag()"));
    assert!(!get.contains(
        "Instruction::LocalGet(receiver_tag_local));\n        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag()"
    ));
    assert_before(
        get,
        "self.emit_value_to_property_key_locals(key_payload_local, key_tag_local, function)?;",
        "self.emit_builtin_arg_is_present_i32(2, function);",
    );
    assert!(get.contains(
        "self.emit_builtin_arg_is_present_i32(2, function);\n        function.instruction(&Instruction::I32Eqz);\n        function.instruction(&Instruction::If(BlockType::Empty));\n        function.instruction(&Instruction::LocalGet(target_payload_local));\n        function.instruction(&Instruction::LocalSet(receiver_payload_local));\n        function.instruction(&Instruction::LocalGet(target_tag_local));\n        function.instruction(&Instruction::LocalSet(receiver_tag_local));"
    ));

    let set = reflect_owner("pub(crate) fn compile_reflect_set_builtin(");
    assert_eq!(
        set.matches("self.emit_builtin_arg_is_present_i32(3, function);")
            .count(),
        1
    );
    assert_eq!(set.matches("ValueKind::Undefined.tag()").count(), 1);
    assert!(!set.contains(
        "Instruction::LocalGet(receiver_tag_local));\n        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag()"
    ));
    assert_before(
        set,
        "self.emit_property_key_value_payload_to_local(",
        "self.emit_builtin_arg_is_present_i32(3, function);",
    );
    assert_before(
        set,
        "self.emit_builtin_arg_is_present_i32(3, function);",
        "function.instruction(&Instruction::LocalSet(handled_local));",
    );
    assert!(set.contains(
        "self.emit_builtin_arg_is_present_i32(3, function);\n        function.instruction(&Instruction::I32Eqz);\n        function.instruction(&Instruction::If(BlockType::Empty));\n        function.instruction(&Instruction::LocalGet(target_payload_local));\n        function.instruction(&Instruction::LocalSet(receiver_payload_local));\n        function.instruction(&Instruction::LocalGet(target_tag_local));\n        function.instruction(&Instruction::LocalSet(receiver_tag_local));"
    ));
}

#[test]
fn runtime_fixture_observes_omitted_and_explicit_undefined_as_distinct() {
    for cli_marker in [
        "fn run_wasm_backend_distinguishes_omitted_reflect_optional_arguments()",
        "\"wasm_reflect_optional_argument_presence.js\"",
    ] {
        assert!(
            CLI_OBJECT_SOURCE.contains(cli_marker),
            "CLI marker `{cli_marker}`"
        );
    }
    for fixture_marker in [
        "Reflect.get(getProxy, \"value\", undefined)",
        "getReceiver === undefined",
        "Reflect.get(getProxy, \"value\")",
        "getReceiver === getProxy",
        "Reflect.set(setProxy, \"value\", 23, undefined)",
        "setReceiver === undefined",
        "Reflect.set(setProxy, \"value\", 23)",
        "setReceiver === setProxy",
        "Reflect.set(ordinaryExplicitUndefinedTarget, \"value\", 29, undefined) === false",
        "Reflect.construct(constructProxy, [], undefined)",
        "explicitUndefinedNewTargetError instanceof TypeError",
        "constructTrapCalls === 0",
        "Reflect.construct(constructProxy, [])",
        "constructNewTarget === constructProxy",
    ] {
        assert!(
            FIXTURE_SOURCE.contains(fixture_marker),
            "fixture marker `{fixture_marker}`"
        );
    }
    assert!(FIXTURE_SOURCE.trim_end().ends_with("true;"));
}

#[test]
fn module_guard_contract_and_task_record_the_presence_boundary() {
    for marker in [
        "'builtin optional-argument presence authority definition/use'",
        "'three Reflect optional-argument presence consumers'",
        "run_wasm_backend_distinguishes_omitted_reflect_optional_arguments",
        "'Reflect optional-argument presence fixture wiring'",
    ] {
        assert!(
            MODULE_BOUNDARY_SOURCE.contains(marker),
            "module-boundary marker `{marker}`"
        );
    }
    for marker in [
        "Reflect optional-argument presence",
        "argc > index",
        "wasm_reflect_optional_argument_presence.js",
        "Verification pending",
    ] {
        assert!(
            CONTRACT_SOURCE.contains(marker),
            "contract marker `{marker}`"
        );
    }
    for marker in [
        "Reflect optional-argument defaults",
        "reflect_optional_argument_presence_structure",
        "wasm_reflect_optional_argument_presence.js",
    ] {
        assert!(TASK_SOURCE.contains(marker), "T11 marker `{marker}`");
    }
}
