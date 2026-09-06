use std::fs;
use std::path::Path;

const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const REALM_OWNER_SOURCE: &str = include_str!("../src/functions/proxy_execution_realm.rs");
const PROXY_BUILTIN_SOURCE: &str = include_str!("../src/builtins/proxy.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn proxy_execution_realm_source_is_closed_and_builder_owned() {
    let variants = bounded(
        EMIT_SOURCE,
        "pub(crate) enum ProxyExecutionRealmSource {",
        "\n}\n\nimpl ProxyExecutionRealmSource",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "MainRealmFallback,",
            "StandardBuiltinEnvironment,",
            "ObjectReadHelperArgument,",
            "ProxyDispatchHelperArgument,",
        ]
    );

    let builder = bounded(
        EMIT_SOURCE,
        "pub(crate) struct FunctionBuilder<'a> {",
        "\n}\n\npub fn emit(",
    );
    assert_eq!(
        builder
            .matches("proxy_execution_realm_source: ProxyExecutionRealmSource,")
            .count(),
        1
    );

    let accessor = normalized(bounded(
        EMIT_SOURCE,
        "pub(crate) const fn proxy_execution_realm_source(&self) -> ProxyExecutionRealmSource {",
        "\n    }\n\n    pub(crate) const fn object_read_error_realm_source",
    ));
    assert_eq!(accessor, "self.proxy_execution_realm_source");
    assert_eq!(
        EMIT_SOURCE
            .matches("ProxyExecutionRealmSource::for_initial_body(numeric_error_realm_source)")
            .count(),
        1
    );

    assert_eq!(
        FUNCTIONS_SOURCE
            .matches("\nmod proxy_execution_realm;\n")
            .count(),
        1
    );
    assert!(!FUNCTIONS_SOURCE.contains("\npub mod proxy_execution_realm;\n"));
}

#[test]
fn initial_and_helper_body_sources_project_exhaustively() {
    let source_domain = bounded(
        EMIT_SOURCE,
        "pub(crate) enum ProxyExecutionRealmSource {",
        "/// What `current_env_local` is allowed to mean when proxy `[[Get]]` detects a",
    );
    let initial_projection = normalized(bounded(
        source_domain,
        "const fn for_initial_body(numeric_source: NumericErrorRealmSource) -> Self {",
        "\n    }\n\n    /// The closed body-domain transition",
    ));
    assert!(initial_projection.contains(concat!(
        "NumericErrorRealmSource::StandardBuiltinEnvironment=>",
        "Self::StandardBuiltinEnvironment,"
    )));
    assert!(initial_projection.contains(concat!(
        "NumericErrorRealmSource::GlobalFallback|",
        "NumericErrorRealmSource::NumericConversionHelperArgument=>",
        "Self::MainRealmFallback"
    )));
    assert!(!initial_projection.contains("_=>"));

    let runtime_projection = bounded(
        source_domain,
        "pub(crate) const fn for_runtime_helper(helper: RuntimeHelperId) -> Self {",
        "\n    }\n}",
    );
    let normalized_runtime_projection = normalized(runtime_projection);
    for (source, helper_arm) in [
        (
            "Self::ObjectReadHelperArgument",
            concat!("RuntimeHelperId::ObjectRead|RuntimeHelperId::ObjectReadProxy|RuntimeHelperId::IndexedElementRead=>",),
        ),
        (
            "Self::ProxyDispatchHelperArgument",
            concat!("RuntimeHelperId::ProxyCall|RuntimeHelperId::ProxyConstruct=>{",),
        ),
    ] {
        assert_eq!(
            normalized_runtime_projection.matches(helper_arm).count(),
            1,
            "{helper_arm}"
        );
        assert_eq!(runtime_projection.matches(source).count(), 1, "{source}");
    }
    assert!(runtime_projection.contains("=> Self::MainRealmFallback,"));
    assert!(!runtime_projection.contains("_ =>"));

    let body_entry = bounded(
        EMIT_SOURCE,
        "pub(crate) fn begin_helper_body(&mut self, helper: RuntimeHelperId) -> Function {",
        "match helper {",
    );
    assert_eq!(
        body_entry
            .matches("ProxyExecutionRealmSource::for_runtime_helper(helper)")
            .count(),
        1
    );
}

#[test]
fn proxy_realm_methods_share_one_trusted_access_projection() {
    let access_projection = normalized(bounded(
        REALM_OWNER_SOURCE,
        "const fn proxy_execution_realm_access(",
        "\n}\n\nimpl FunctionBuilder",
    ));
    assert!(access_projection.contains(concat!(
        "ProxyExecutionRealmSource::MainRealmFallback=>{",
        "ProxyExecutionRealmAccess::MainRealmFallback}"
    )));
    assert!(access_projection.contains(concat!(
        "ProxyExecutionRealmSource::StandardBuiltinEnvironment|",
        "ProxyExecutionRealmSource::ObjectReadHelperArgument|",
        "ProxyExecutionRealmSource::ProxyDispatchHelperArgument=>{",
        "ProxyExecutionRealmAccess::TrustedCurrentEnvironment}"
    )));
    assert!(!access_projection.contains("_=>"));

    for method in [
        "emit_proxy_execution_realm_argument",
        "emit_proxy_execution_realm_type_error",
        "emit_install_proxy_execution_realm_array_prototype",
    ] {
        assert_eq!(
            REALM_OWNER_SOURCE
                .matches(&format!("pub(crate) fn {method}("))
                .count(),
            1,
            "{method}"
        );
        assert!(
            !FUNCTIONS_SOURCE.contains(&format!("pub(crate) fn {method}(")),
            "{method} must stay in its private owner module"
        );
    }
    assert_eq!(
        REALM_OWNER_SOURCE
            .matches("match proxy_execution_realm_access(self.proxy_execution_realm_source())")
            .count(),
        3
    );

    let realm_argument = bounded(
        REALM_OWNER_SOURCE,
        "pub(crate) fn emit_proxy_execution_realm_argument",
        "pub(crate) fn emit_proxy_execution_realm_type_error",
    );
    assert!(realm_argument.contains("Instruction::LocalGet(self.current_env_local)"));
    assert!(realm_argument.contains("Instruction::I64Const(0)"));

    let type_error = bounded(
        REALM_OWNER_SOURCE,
        "pub(crate) fn emit_proxy_execution_realm_type_error",
        "pub(crate) fn emit_install_proxy_execution_realm_array_prototype",
    );
    assert!(type_error.contains("emit_throw_current_function_realm_type_error("));
    assert!(type_error.contains("emit_throw_runtime_error("));

    let array_prototype = bounded(
        REALM_OWNER_SOURCE,
        "pub(crate) fn emit_install_proxy_execution_realm_array_prototype",
        "\n    }\n}\n\n#[cfg(test)]",
    );
    assert!(array_prototype.contains("emit_load_current_function_realm_array_prototype("));
    assert!(array_prototype.contains("emit_install_current_function_realm_array_prototype("));
    assert!(array_prototype.contains("ARRAY_PROTOTYPE_GLOBAL_INDEX"));
    assert!(!array_prototype.contains("_ =>"));
}

#[test]
fn proxy_and_object_read_helpers_restore_parameter_six_and_forward_it() {
    for (runtime_body_id, start, end, dispatch) in [
        (
            "RuntimeHelperId::ProxyCall",
            "fn compile_proxy_call_helper(&mut self)",
            "fn compile_proxy_construct_helper(&mut self)",
            "self.emit_function_or_proxy_call_with_argv_leave_throw_completion(",
        ),
        (
            "RuntimeHelperId::ProxyConstruct",
            "fn compile_proxy_construct_helper(&mut self)",
            "fn compile_string_equality_helper(&mut self)",
            "self.emit_function_or_proxy_construct_with_argv(",
        ),
    ] {
        let compiled_body = bounded(EMIT_SOURCE, start, end);
        assert_eq!(compiled_body.matches("Instruction::LocalGet(6)").count(), 1);
        assert_eq!(
            compiled_body
                .matches("Instruction::LocalSet(self.current_env_local)")
                .count(),
            1
        );
        let load = compiled_body
            .find("Instruction::LocalGet(6)")
            .expect("parameter 6 load");
        let store = compiled_body
            .find("Instruction::LocalSet(self.current_env_local)")
            .expect("current environment store");
        let dispatch = compiled_body.find(dispatch).expect("Proxy dispatch body");
        assert!(load < store && store < dispatch, "{runtime_body_id}");
        assert!(compiled_body.contains(runtime_body_id));
    }

    for (runtime_body_id, start, end, dispatch) in [
        (
            "RuntimeHelperId::ObjectRead",
            "fn compile_object_read_helper(&mut self)",
            "fn compile_object_write_helper(&mut self)",
            "self.emit_object_read_ordinary_inner(",
        ),
        (
            "RuntimeHelperId::ObjectReadProxy",
            "fn compile_object_read_proxy_helper(&mut self)",
            "// `compile_indexed_element_read_helper`",
            "self.emit_object_read_with_key_tag(",
        ),
    ] {
        let compiled_body = bounded(EMIT_SOURCE, start, end);
        let load = compiled_body
            .find("Instruction::LocalGet(6)")
            .expect("parameter 6 load");
        let store = compiled_body
            .find("Instruction::LocalSet(self.current_env_local)")
            .expect("current environment store");
        let dispatch = compiled_body.find(dispatch).expect("object-read body");
        assert!(load < store && store < dispatch, "{runtime_body_id}");
        assert_eq!(compiled_body.matches("Instruction::LocalGet(6)").count(), 1);
        assert_eq!(
            compiled_body
                .matches("Instruction::LocalSet(self.current_env_local)")
                .count(),
            1
        );
        assert!(compiled_body.contains(runtime_body_id));
    }

    let ordinary_read_call = bounded(
        OBJECTS_SOURCE,
        "fn emit_object_read_ordinary_via_helper(",
        "pub(crate) fn emit_object_read_ordinary(",
    );
    assert_eq!(
        ordinary_read_call
            .matches("Instruction::I64Const(0)")
            .count(),
        1
    );
    assert_eq!(
        ordinary_read_call
            .matches("self.emit_proxy_execution_realm_argument(function);")
            .count(),
        1
    );
    assert!(
        ordinary_read_call
            .find("self.emit_proxy_execution_realm_argument(function);")
            .unwrap()
            < ordinary_read_call
                .find("Instruction::Call(helper)")
                .unwrap()
    );

    let construct = bounded(
        FUNCTIONS_SOURCE,
        "pub(crate) fn emit_function_or_proxy_construct_with_argv(",
        "pub(crate) fn emit_function_handle_construct_with_argv(",
    );
    let call = bounded(
        FUNCTIONS_SOURCE,
        "fn emit_function_or_proxy_call_with_argv_inner(",
        "pub(crate) fn emit_function_handle_call_with_argv_inner(",
    );
    for (state_machine, outline_start) in [
        (construct, "if self.outline_proxy_construct {"),
        (call, "if self.outline_proxy_call {"),
    ] {
        let outlined_dispatch = bounded(state_machine, outline_start, "let current_payload_local");
        assert_eq!(
            outlined_dispatch
                .matches("self.emit_proxy_execution_realm_argument(function);")
                .count(),
            1
        );
        assert!(!outlined_dispatch.contains("Instruction::I64Const(0)"));
        assert!(
            outlined_dispatch
                .find("self.emit_proxy_execution_realm_argument(function);")
                .unwrap()
                < outlined_dispatch.find("Instruction::Call(helper)").unwrap()
        );
        assert_eq!(
            state_machine
                .matches("self.emit_proxy_call_helper_leave_throw_completion(")
                .count(),
            1
        );
    }

    let nested_trap_call = bounded(
        FUNCTIONS_SOURCE,
        "fn emit_proxy_call_helper_leave_throw_completion(",
        "/// `emit_function_or_proxy_call_leave_throw_completion` plus the throw",
    );
    assert_eq!(
        nested_trap_call
            .matches("self.emit_proxy_execution_realm_argument(function);")
            .count(),
        1
    );
    assert!(!nested_trap_call.contains("Instruction::I64Const(0)"));
    assert!(
        nested_trap_call
            .find("self.emit_proxy_execution_realm_argument(function);")
            .unwrap()
            < nested_trap_call.find("Instruction::Call(helper)").unwrap()
    );

    let tail_dispatch = bounded(
        FUNCTIONS_SOURCE,
        "pub(crate) fn emit_tail_indirect_call(",
        "fn emit_custom_array_named_method_call(",
    );
    let plain_function_tail = normalized(bounded(
        tail_dispatch,
        "function.instruction(&Instruction::If(BlockType::Empty));",
        "function.instruction(&Instruction::Else);",
    ));
    assert!(plain_function_tail.ends_with(concat!(
        "function.instruction(&Instruction::LocalGet(argv_local));",
        "function.instruction(&Instruction::I64Const(0));",
        "function.instruction(&Instruction::ReturnCall(function_helper));"
    )));
    assert!(!plain_function_tail.contains("emit_proxy_execution_realm_argument"));

    let proxy_tail = normalized(bounded(
        tail_dispatch,
        "function.instruction(&Instruction::Else);",
        "function.instruction(&Instruction::End);",
    ));
    assert!(proxy_tail.starts_with(concat!(
        "function.instruction(&Instruction::LocalGet(callee_payload_local));",
        "function.instruction(&Instruction::LocalGet(callee_tag_local));",
        "function.instruction(&Instruction::LocalGet(this_payload_local));",
        "function.instruction(&Instruction::LocalGet(this_tag_local));",
        "function.instruction(&Instruction::LocalGet(argc_local));",
        "function.instruction(&Instruction::LocalGet(argv_local));",
        "self.emit_proxy_execution_realm_argument(function);",
        "function.instruction(&Instruction::ReturnCall(proxy_helper));"
    )));
    assert!(!proxy_tail.contains("Instruction::I64Const(0)"));
}

#[test]
fn call_and_construct_errors_and_argument_arrays_use_the_execution_realm() {
    let construct = bounded(
        FUNCTIONS_SOURCE,
        "pub(crate) fn emit_function_or_proxy_construct_with_argv(",
        "pub(crate) fn emit_function_handle_construct_with_argv(",
    );
    let call = bounded(
        FUNCTIONS_SOURCE,
        "fn emit_function_or_proxy_call_with_argv_inner(",
        "pub(crate) fn emit_function_handle_call_with_argv_inner(",
    );
    for (state_machine, expected_type_error_calls, generated_errors) in [
        (
            construct,
            5,
            &[
                "target is not a constructor",
                "Proxy handler is null",
                "Proxy construct trap returned non-object",
                "Proxy construct trap is not callable",
            ][..],
        ),
        (
            call,
            4,
            &[
                "value is not callable",
                "Proxy handler is null",
                "Proxy apply trap is not callable",
            ][..],
        ),
    ] {
        for generated_error in generated_errors {
            assert!(
                state_machine.contains(generated_error),
                "missing generated error `{generated_error}`"
            );
        }
        assert_eq!(
            state_machine
                .matches("self.emit_proxy_execution_realm_type_error(")
                .count(),
            expected_type_error_calls
        );
        for forbidden in [
            "self.emit_throw_runtime_error(",
            "self.emit_throw_runtime_error_with_prototype_local(",
            "self.emit_throw_current_function_realm_type_error(",
            "TYPE_ERROR_NAME",
            "TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX",
            "HEAP_PROXY_TYPE_ERROR_PROTOTYPE_OFFSET",
        ] {
            assert!(!state_machine.contains(forbidden), "found `{forbidden}`");
        }

        assert_eq!(
            state_machine
                .matches("self.emit_array_like_snapshot_payload(")
                .count(),
            1
        );
        assert_eq!(
            state_machine
                .matches("self.emit_install_proxy_execution_realm_array_prototype(")
                .count(),
            1
        );
        let after_snapshot = state_machine
            .split_once("self.emit_array_like_snapshot_payload(")
            .expect("argument Array snapshot")
            .1
            .split_once(")?;")
            .expect("completed argument Array snapshot")
            .1;
        assert!(normalized(after_snapshot).starts_with(concat!(
            "self.emit_install_proxy_execution_realm_array_prototype(",
            "trap_args_payload_local,function);"
        )));
    }
}

#[test]
fn proxy_objects_retain_no_creation_realm_error_snapshot() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "HEAP_PROXY_TYPE_ERROR_PROTOTYPE_OFFSET"),
        0
    );

    let revocable = bounded(
        PROXY_BUILTIN_SOURCE,
        "pub(super) fn compile_proxy_revocable_builtin(",
        "pub(super) fn compile_proxy_revoke_builtin(",
    );
    for creation_realm_snapshot in [
        "type_error_prototype_local",
        "TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX",
        "emit_load_function_defining_realm_type_error_prototype(",
        "HEAP_PROXY_TYPE_ERROR_PROTOTYPE_OFFSET",
        "self.this_payload_local",
        "self.this_tag_local",
    ] {
        assert!(
            !revocable.contains(creation_realm_snapshot),
            "found creation-Realm snapshot `{creation_realm_snapshot}`"
        );
    }
}
