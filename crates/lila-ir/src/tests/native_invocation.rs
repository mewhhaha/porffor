#[test]
fn typed_array_factory_aliases_keep_the_acquired_key_and_full_arguments() {
    for (method, builtin, arguments) in [
        (
            "from",
            StandardBuiltinId::TypedArrayFrom,
            "(receiver.from = function replacement() { return 99; }, [1]), undefined, ...[2]",
        ),
        (
            "of",
            StandardBuiltinId::TypedArrayOf,
            "(receiver.of = function replacement() { return 99; }, 1), ...[2], 3",
        ),
    ] {
        // A transferred native must reach its own receiver validation.
        // A same-named own function is not the function that was acquired.
        let source = format!(
            "function run() {{ const receiver = {{ saved: Uint8Array.{method}, \
                 {method}() {{ return 7; }} }}; return receiver.saved({arguments}); }} run();"
        );
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let run = program
            .script
            .as_ref()
            .expect("script")
            .functions
            .iter()
            .find(|function| function.name == "run")
            .expect("run");
        let result = function_return(run).expect("factory call");
        let (callee, args) = retained_indexed_collection_call(result, "saved");
        assert_eq!(
            callee.function_targets.exact_single_target(),
            Some(&builtin.function_id())
        );
        assert_eq!(args.len(), 3);
        let ExprIr::Comma { lhs, .. } = &args[0].expr else {
            panic!("{args:?}")
        };
        assert!(matches!(
            lhs.expr,
            ExprIr::OrdinaryPropertyAssignment(_) | ExprIr::PropertyWrite { .. }
        ));
        let spread_index = if method == "from" { 2 } else { 1 };
        assert!(
            matches!(&args[spread_index].expr, ExprIr::SpreadArgument(spread)
                if spread.protocol == SpreadArgumentProtocol::ARGUMENT_LIST)
        );
        assert_eq!(result.kind, ValueKind::Object);
        assert_eq!(result.possible_kinds, KindSet::from_kind(ValueKind::Object));
        assert!(result.heap_shape.is_none());
        assert!(result.function_targets.known_targets().is_empty());
    }
}

#[test]
fn typed_array_factory_direct_and_spread_results_have_no_invented_shape() {
    for (method, builtin, arguments, argument_count) in [
        ("from", StandardBuiltinId::TypedArrayFrom, "[1]", 1),
        ("from", StandardBuiltinId::TypedArrayFrom, "...[ [1] ]", 1),
        ("of", StandardBuiltinId::TypedArrayOf, "1, 2", 2),
        ("of", StandardBuiltinId::TypedArrayOf, "...[1, 2]", 1),
    ] {
        let source =
            format!("function run() {{ return Uint8Array.{method}({arguments}); }} run();");
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let run = program
            .script
            .as_ref()
            .expect("script")
            .functions
            .iter()
            .find(|function| function.name == "run")
            .expect("run");
        let result = function_return(run).expect("native factory call");
        let (callee, args) = retained_indexed_collection_call(result, method);
        assert_eq!(
            callee.function_targets.exact_single_target(),
            Some(&builtin.function_id())
        );
        assert_eq!(args.len(), argument_count);
        assert_eq!(result.kind, ValueKind::Object);
        assert_eq!(result.possible_kinds, KindSet::from_kind(ValueKind::Object));
        assert!(result.heap_shape.is_none());
        assert!(result.function_targets.known_targets().is_empty());
    }
}

#[test]
fn iterator_next_alias_keeps_the_runtime_reference_and_full_arguments() {
    let program = lower_script(
        "function run() { const iterator = [1].values(); iterator.saved = iterator.next; \
             return iterator.saved((iterator.next = function replacement() { return 99; }, 1), \
             ...[2], 3); } run();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let run = program
        .script
        .as_ref()
        .expect("script")
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run");
    let result = function_return(run).expect("runtime iterator call");
    let (callee, args) = retained_indexed_collection_call(result, "saved");
    // Iterator factories have no immutable inherited-method shape. This
    // read cannot authenticate an exact native next target.
    assert!(callee.function_targets.known_targets().is_empty());
    assert_eq!(args.len(), 3);
    assert!(matches!(args[0].expr, ExprIr::Comma { .. }));
    assert!(matches!(&args[1].expr, ExprIr::SpreadArgument(spread)
            if spread.protocol == SpreadArgumentProtocol::ARGUMENT_LIST));
    assert!(matches!(args[2].expr, ExprIr::Number(bits) if f64::from_bits(bits) == 3.0));
    assert_eq!(result.kind, ValueKind::Dynamic);
    assert!(result.heap_shape.is_none());
}

#[test]
fn species_getter_results_keep_all_raw_this_tags_on_direct_and_spread_calls() {
    for constructor in [
        "Array",
        "Object.getPrototypeOf(Uint8Array)",
        "ArrayBuffer",
        "RegExp",
    ] {
        for arguments in ["", "...[1], 2"] {
            let source = format!(
                    "function run() {{ const receiver = {{ saved: Object.getOwnPropertyDescriptor(\
                     {constructor}, Symbol.species).get }}; return receiver.saved({arguments}); }} run();"
                );
            let program = lower_script(&source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let run = program
                .script
                .as_ref()
                .expect("script")
                .functions
                .iter()
                .find(|function| function.name == "run")
                .expect("run");
            let result = function_return(run).expect("species getter call");
            let (_, args) = retained_indexed_collection_call(result, "saved");
            assert_eq!(args.len(), if arguments.is_empty() { 0 } else { 2 });
            if !arguments.is_empty() {
                assert!(matches!(&args[0].expr, ExprIr::SpreadArgument(spread)
                        if spread.protocol == SpreadArgumentProtocol::ARGUMENT_LIST));
            }
            assert_eq!(result.kind, ValueKind::Dynamic);
            assert_eq!(result.possible_kinds, KindSet::all_runtime_tags());
            assert!(result.heap_shape.is_none());
            assert_eq!(result.function_targets, FunctionTargetKnowledge::unknown());
        }
    }
}

#[test]
fn array_buffer_species_forwarding_retains_call_and_raw_this_operands() {
    for (raw_this, kind) in [
        ("undefined", ValueKind::Undefined),
        ("null", ValueKind::Null),
        ("0", ValueKind::Number),
        ("false", ValueKind::Boolean),
        ("'raw'", ValueKind::String),
        ("1n", ValueKind::BigInt),
        ("Symbol('raw')", ValueKind::Symbol),
    ] {
        let source = format!(
                "function run() {{ const getter = Object.getOwnPropertyDescriptor(\
                 ArrayBuffer, Symbol.species).get; return getter.call({raw_this}, ...[1], 2); }} run();"
            );
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let run = program
            .script
            .as_ref()
            .expect("script")
            .functions
            .iter()
            .find(|function| function.name == "run")
            .expect("run");
        let result = function_return(run).expect("forwarded species getter call");
        let ExprIr::MaterializeBinding { value, .. } = &result.expr else {
            panic!("forwarding retains its actual receiver: {result:?}");
        };
        assert!(matches!(&value.expr, ExprIr::Identifier(name) if name == "getter"));
        let (callee, args) = retained_indexed_collection_call(result, "call");
        assert!(callee
            .function_targets
            .known_targets()
            .contains(&StandardBuiltinId::FunctionPrototypeCall.function_id()));
        assert!(callee.function_targets.exact_targets().is_none());
        assert_eq!(args.len(), 3);
        if kind == ValueKind::Symbol {
            // The preceding live `.call` Get may replace the mutable Symbol
            // global. Keep its actual invocation instead of asserting a type
            // fact from the source name.
            assert!(args[0].possible_kinds.contains(ValueKind::Symbol));
            let ExprIr::CallIndirect {
                args: symbol_args, ..
            } = &args[0].expr
            else {
                panic!(
                    "the original Symbol argument call is retained: {:?}",
                    args[0]
                );
            };
            assert_eq!(symbol_args.len(), 1);
            assert!(matches!(&symbol_args[0].expr, ExprIr::String(value) if value == "raw"));
        } else {
            assert_eq!(args[0].kind, kind);
            assert_eq!(args[0].possible_kinds, KindSet::from_kind(kind));
        }
        assert!(matches!(&args[1].expr, ExprIr::SpreadArgument(spread)
                if spread.protocol == SpreadArgumentProtocol::ARGUMENT_LIST));
        assert!(matches!(args[2].expr, ExprIr::Number(bits) if f64::from_bits(bits) == 2.0));
        assert_eq!(result.kind, ValueKind::Dynamic);
        assert_eq!(result.possible_kinds, KindSet::all_runtime_tags());
        assert!(result.heap_shape.is_none());
        assert_eq!(result.function_targets, FunctionTargetKnowledge::unknown());
    }
}

#[test]
fn literal_builtin_calls_retain_the_callee_and_every_effectful_operand() {
    for (callee_source, builtin, input, result_kind) in [
        (
            "unescape",
            StandardBuiltinId::Unescape,
            "'%41'",
            ValueKind::String,
        ),
        (
            "Number.isFinite",
            StandardBuiltinId::NumberIsFinite,
            "effect = 'changed'",
            ValueKind::Boolean,
        ),
        (
            "Number.isNaN",
            StandardBuiltinId::NumberIsNaN,
            "effect = 'changed'",
            ValueKind::Boolean,
        ),
        (
            "Number.isSafeInteger",
            StandardBuiltinId::NumberIsSafeInteger,
            "effect = 'changed'",
            ValueKind::Boolean,
        ),
    ] {
        for property in [false, true] {
            let setup = if property {
                format!("const receiver = {{ saved: {callee_source} }};")
            } else {
                String::new()
            };
            let invocation = if property {
                "receiver.saved"
            } else {
                callee_source
            };
            let source = format!(
                "function run() {{ let effect = 0; let extra = 0; {setup} \
                     return {invocation}({input}, (extra = 1, 9), ...[77]); }} run();"
            );
            let program = lower_script(&source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let run = program
                .script
                .as_ref()
                .expect("script")
                .functions
                .iter()
                .find(|function| function.name == "run")
                .expect("run");
            let result = function_return(run).expect("literal builtin call");
            let (callee, args) = if property {
                retained_indexed_collection_call(result, "saved")
            } else if callee_source.starts_with("Number.") {
                retained_indexed_collection_call(
                    result,
                    callee_source
                        .strip_prefix("Number.")
                        .expect("Number method"),
                )
            } else {
                let ExprIr::CallIndirect {
                    callee,
                    this_arg: None,
                    args,
                    ..
                } = &result.expr
                else {
                    panic!("{result:?}")
                };
                (callee.as_ref(), args.as_slice())
            };
            assert_eq!(
                callee.function_targets.exact_single_target(),
                Some(&builtin.function_id())
            );
            assert_eq!(args.len(), 3);
            if builtin == StandardBuiltinId::Unescape {
                assert!(matches!(&args[0].expr, ExprIr::String(value) if value == "%41"));
            } else {
                assert!(matches!(args[0].expr, ExprIr::AssignIdentifier { .. }));
                assert_eq!(args[0].kind, ValueKind::String);
            }
            let ExprIr::Comma { lhs, rhs } = &args[1].expr else {
                panic!("{args:?}")
            };
            assert!(matches!(lhs.expr, ExprIr::AssignIdentifier { .. }));
            assert!(matches!(rhs.expr, ExprIr::Number(bits) if f64::from_bits(bits) == 9.0));
            assert!(matches!(&args[2].expr, ExprIr::SpreadArgument(spread)
                    if spread.protocol == SpreadArgumentProtocol::ARGUMENT_LIST));
            assert_eq!(result.kind, result_kind);
        }
    }
}

#[test]
fn remaining_native_invocations_invalidate_observable_caller_facts() {
    for invocation in [
            "function mutate() { captured.value = 'changed'; } mutate.call(null);",
            "function target() {} Object.defineProperty(target, 'name', { \
             get() { captured.value = 'changed'; return 'target'; } }); \
             captured = { value: 1 }; target.bind(null);",
            "const values = [1]; Object.defineProperty(values, '0', { \
             get() { captured.value = 'changed'; return 1; } }); const iterator = values.values(); \
             captured = { value: 1 }; iterator.next();",
            "const element = { valueOf() { captured.value = 'changed'; return 1; } }; Uint8Array.of(element);",
            "const input = { toString() { captured.value = 'changed'; return '%41'; } }; unescape(input);",
        ] {
            let source = format!("let captured = {{ value: 1 }}; {invocation} captured.value + 1;");
            assert_caller_flow_invalidation_reaches_final_addition(&source);
        }
}

#[test]
fn codec_native_aliases_retain_acquired_references_and_full_operands() {
    for (source_name, builtin) in [
        ("escape", StandardBuiltinId::Escape),
        ("unescape", StandardBuiltinId::Unescape),
        ("encodeURI", StandardBuiltinId::EncodeUri),
        ("encodeURIComponent", StandardBuiltinId::EncodeUriComponent),
        ("decodeURI", StandardBuiltinId::DecodeUri),
        ("decodeURIComponent", StandardBuiltinId::DecodeUriComponent),
    ] {
        let source = format!(
                "function run() {{ let extra = 0; const input = {{ toString() {{ return 'text'; }} }}; \
                 const receiver = {{ saved: {source_name} }}; \
                 return receiver.saved((receiver.saved = function replacement() {{ return 99; }}, input), \
                 (extra = 1, 9), ...[77]); }} run();"
            );
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let run = program
            .script
            .as_ref()
            .expect("script")
            .functions
            .iter()
            .find(|function| function.name == "run")
            .expect("run");
        let result = function_return(run).expect("native codec call");
        let (callee, args) = retained_indexed_collection_call(result, "saved");
        assert_eq!(
            callee.function_targets.exact_single_target(),
            Some(&builtin.function_id())
        );
        let [input, ignored, spread] = args else {
            panic!("{args:?}");
        };
        let ExprIr::Comma { lhs, rhs } = &input.expr else {
            panic!("{input:?}");
        };
        assert!(matches!(
            lhs.expr,
            ExprIr::OrdinaryPropertyAssignment(_) | ExprIr::PropertyWrite { .. }
        ));
        assert!(matches!(rhs.expr, ExprIr::Identifier(_)));
        let ExprIr::Comma { lhs, rhs } = &ignored.expr else {
            panic!("{ignored:?}");
        };
        assert!(matches!(lhs.expr, ExprIr::AssignIdentifier { .. }));
        assert!(matches!(rhs.expr, ExprIr::Number(bits) if f64::from_bits(bits) == 9.0));
        assert!(matches!(&spread.expr, ExprIr::SpreadArgument(argument)
                if argument.protocol == SpreadArgumentProtocol::ARGUMENT_LIST));
        assert_eq!(result.kind, ValueKind::String);
    }
}

#[test]
fn error_constructor_aliases_retain_each_native_argument_domain() {
    for (source_name, builtin, operands, message_index, spread_index) in [
        (
            "Error",
            StandardBuiltinId::ErrorConstructor,
            "MESSAGE, options, ...[9], (extra = 1, 10)",
            0,
            2,
        ),
        (
            "EvalError",
            StandardBuiltinId::EvalErrorConstructor,
            "MESSAGE, options, ...[9], (extra = 1, 10)",
            0,
            2,
        ),
        (
            "RangeError",
            StandardBuiltinId::RangeErrorConstructor,
            "MESSAGE, options, ...[9], (extra = 1, 10)",
            0,
            2,
        ),
        (
            "SyntaxError",
            StandardBuiltinId::SyntaxErrorConstructor,
            "MESSAGE, options, ...[9], (extra = 1, 10)",
            0,
            2,
        ),
        (
            "TypeError",
            StandardBuiltinId::TypeErrorConstructor,
            "MESSAGE, options, ...[9], (extra = 1, 10)",
            0,
            2,
        ),
        (
            "URIError",
            StandardBuiltinId::URIErrorConstructor,
            "MESSAGE, options, ...[9], (extra = 1, 10)",
            0,
            2,
        ),
        (
            "ReferenceError",
            StandardBuiltinId::ReferenceErrorConstructor,
            "MESSAGE, options, ...[9], (extra = 1, 10)",
            0,
            2,
        ),
        (
            "AggregateError",
            StandardBuiltinId::AggregateErrorConstructor,
            "errors, MESSAGE, options, ...[9], (extra = 1, 10)",
            1,
            3,
        ),
        (
            "SuppressedError",
            StandardBuiltinId::SuppressedErrorConstructor,
            "original, null, MESSAGE, ...[options], (extra = 1, 10)",
            2,
            3,
        ),
    ] {
        let operands = operands.replace(
            "MESSAGE",
            "(receiver.saved = function replacement() { return 99; }, message)",
        );
        let source = format!(
                "function run() {{ let extra = 0; const message = {{ toString() {{ return 'text'; }} }}; \
                 const options = {{ cause: 7 }}; const errors = []; const original = Symbol.iterator; \
                 const receiver = {{ saved: {source_name} }}; return receiver.saved({operands}); }} run();"
            );
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let run = program
            .script
            .as_ref()
            .expect("script")
            .functions
            .iter()
            .find(|function| function.name == "run")
            .expect("run");
        let result = function_return(run).expect("native Error constructor call");
        let (callee, args) = retained_indexed_collection_call(result, "saved");
        assert_eq!(
            callee.function_targets.exact_single_target(),
            Some(&builtin.function_id())
        );
        assert_eq!(
            args.len(),
            if source_name == "AggregateError" || source_name == "SuppressedError" {
                5
            } else {
                4
            }
        );
        let ExprIr::Comma { lhs, rhs } = &args[message_index].expr else {
            panic!("{args:?}");
        };
        assert!(matches!(
            lhs.expr,
            ExprIr::OrdinaryPropertyAssignment(_) | ExprIr::PropertyWrite { .. }
        ));
        assert!(matches!(rhs.expr, ExprIr::Identifier(_)));
        assert!(
            matches!(&args[spread_index].expr, ExprIr::SpreadArgument(argument)
                if argument.protocol == SpreadArgumentProtocol::ARGUMENT_LIST)
        );
        let ExprIr::Comma { lhs, rhs } = &args.last().expect("ignored extra").expr else {
            panic!("{args:?}");
        };
        assert!(matches!(lhs.expr, ExprIr::AssignIdentifier { .. }));
        assert!(matches!(rhs.expr, ExprIr::Number(bits) if f64::from_bits(bits) == 10.0));
        if source_name == "SuppressedError" {
            assert_eq!(args[0].kind, ValueKind::Symbol);
            assert_eq!(args[1].kind, ValueKind::Null);
        }
        assert_eq!(result.kind, ValueKind::Object);
    }
}

#[test]
fn error_to_string_alias_keeps_its_raw_receiver_and_ignored_operands() {
    let program = lower_script(
            "function run() { const receiver = { name: 'N', message: 'M', saved: Error.prototype.toString }; \
             return receiver.saved((receiver.saved = function replacement() { return 99; }, 1), ...[2], 3); } run();"
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let run = program
        .script
        .as_ref()
        .expect("script")
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run");
    let result = function_return(run).expect("native Error.toString call");
    let (callee, args) = retained_indexed_collection_call(result, "saved");
    assert_eq!(
        callee.function_targets.exact_single_target(),
        Some(&StandardBuiltinId::ErrorPrototypeToString.function_id())
    );
    assert_eq!(args.len(), 3);
    assert!(matches!(args[0].expr, ExprIr::Comma { .. }));
    assert!(matches!(&args[1].expr, ExprIr::SpreadArgument(argument)
            if argument.protocol == SpreadArgumentProtocol::ARGUMENT_LIST));
    assert!(matches!(args[2].expr, ExprIr::Number(bits) if f64::from_bits(bits) == 3.0));
    assert_eq!(result.kind, ValueKind::String);
}

#[test]
fn codec_direct_and_native_candidate_calls_invalidate_captured_facts() {
    for codec in [
        "escape",
        "unescape",
        "encodeURI",
        "encodeURIComponent",
        "decodeURI",
        "decodeURIComponent",
    ] {
        for candidate in [false, true] {
            let setup = if candidate {
                format!("const native = unknown ? {codec} : Number.isFinite;")
            } else {
                String::new()
            };
            let invocation = if candidate { "native" } else { codec };
            for (declaration, mutation, observation) in [
                ("let captured = 1;", "captured = 'changed';", "captured"),
                (
                    "let captured = { value: 1 };",
                    "captured.value = 'changed';",
                    "captured.value",
                ),
                (
                    "let captured = [1];",
                    "captured[0] = 'changed';",
                    "captured[0]",
                ),
            ] {
                let source = format!(
                    "{setup} const input = {{ toString() {{ {mutation} return 'text'; }} }}; \
                         {declaration} {invocation}(input); {observation} + 1;"
                );
                assert_caller_flow_invalidation_reaches_final_addition(&source);
            }
        }
    }
}

#[test]
fn error_message_direct_construct_and_candidate_calls_invalidate_caller_facts() {
    for (constructor, operands) in [
        ("Error", "message"),
        ("EvalError", "message"),
        ("RangeError", "message"),
        ("SyntaxError", "message"),
        ("TypeError", "message"),
        ("URIError", "message"),
        ("ReferenceError", "message"),
        ("AggregateError", "[], message"),
        ("SuppressedError", "7, null, message"),
    ] {
        for form in ["call", "construct", "candidate"] {
            let setup = if form == "candidate" {
                let alternative = if constructor == "Error" {
                    "EvalError"
                } else {
                    "Error"
                };
                format!("const native = unknown ? {constructor} : {alternative};")
            } else {
                String::new()
            };
            let invocation = match form {
                "call" => format!("{constructor}({operands})"),
                "construct" => format!("new {constructor}({operands})"),
                "candidate" => format!("native({operands})"),
                _ => unreachable!(),
            };
            let source = format!(
                    "{setup} const message = {{ toString() {{ captured.value = 'changed'; return 'text'; }} }}; \
                     let captured = {{ value: 1 }}; {invocation}; captured.value + 1;"
                );
            assert_caller_flow_invalidation_reaches_final_addition(&source);
        }
    }
}

#[test]
fn error_native_property_iteration_and_new_target_hooks_invalidate_caller_facts() {
    for (setup, invocation, observation) in [
            ("const options = { get cause() { captured = 'changed'; return 7; } }; let captured = 1;",
             "Error(undefined, options)", "captured"),
            ("const options = new Proxy({}, { has(target, key) { captured[0] = 'changed'; return false; } }); let captured = [1];",
             "TypeError(undefined, options)", "captured[0]"),
            ("const iterable = { [Symbol.iterator]() { captured.value = 'changed'; return [][Symbol.iterator](); } }; let captured = { value: 1 };",
             "AggregateError(iterable)", "captured.value"),
            ("const receiver = { get name() { captured.value = 'changed'; return 'Name'; }, message: 'text', saved: Error.prototype.toString }; let captured = { value: 1 };",
             "receiver.saved()", "captured.value"),
            ("const message = { toString() { captured[0] = 'changed'; return 'text'; } }; const receiver = { name: 'Name', message, saved: Error.prototype.toString }; let captured = [1];",
             "receiver.saved()", "captured[0]"),
            ("function Target() {} const newTarget = new Proxy(Target, { get(target, key, receiver) { if (key === 'prototype') captured = 'changed'; return Reflect.get(target, key, receiver); } }); let captured = 1;",
             "Reflect.construct(Error, [], newTarget)", "captured"),
        ] {
            let source = format!("{setup} {invocation}; {observation} + 1;");
            assert_caller_flow_invalidation_reaches_final_addition(&source);
        }
}

#[test]
fn numeric_native_aliases_retain_acquired_callees_and_full_operands() {
    for (native, builtin, first, result_kind) in [
        (
            "Number",
            StandardBuiltinId::NumberConstructor,
            "input",
            ValueKind::Number,
        ),
        (
            "BigInt",
            StandardBuiltinId::BigIntConstructor,
            "input",
            ValueKind::BigInt,
        ),
        (
            "isFinite",
            StandardBuiltinId::GlobalIsFinite,
            "input",
            ValueKind::Boolean,
        ),
        (
            "isNaN",
            StandardBuiltinId::GlobalIsNaN,
            "input",
            ValueKind::Boolean,
        ),
        (
            "Math.abs",
            StandardBuiltinId::MathAbs,
            "input",
            ValueKind::Number,
        ),
        (
            "Math.pow",
            StandardBuiltinId::MathPow,
            "input",
            ValueKind::Number,
        ),
        (
            "Math.sumPrecise",
            StandardBuiltinId::MathSumPrecise,
            "[1, 2]",
            ValueKind::Number,
        ),
        (
            "Atomics.isLockFree",
            StandardBuiltinId::AtomicsIsLockFree,
            "input",
            ValueKind::Boolean,
        ),
    ] {
        let source = format!(
            "function run() {{ let extra = 0; const input = {{ valueOf() {{ return 2; }} }}; \
                 const receiver = {{ saved: {native} }}; return receiver.saved(\
                 (receiver.saved = function replacement() {{ return 99; }}, {first}), \
                 (extra = 1, 9), ...[77]); }} run();"
        );
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let run = program
            .script
            .as_ref()
            .expect("script")
            .functions
            .iter()
            .find(|function| function.name == "run")
            .expect("run");
        let result = function_return(run).expect("native numeric call");
        let (callee, args) = retained_indexed_collection_call(result, "saved");
        assert_eq!(
            callee.function_targets.exact_single_target(),
            Some(&builtin.function_id())
        );
        let [first, ignored, spread] = args else {
            panic!("{args:?}");
        };
        let ExprIr::Comma { lhs, .. } = &first.expr else {
            panic!("{first:?}");
        };
        assert!(matches!(
            lhs.expr,
            ExprIr::OrdinaryPropertyAssignment(_) | ExprIr::PropertyWrite { .. }
        ));
        let ExprIr::Comma { lhs, rhs } = &ignored.expr else {
            panic!("{ignored:?}");
        };
        assert!(matches!(lhs.expr, ExprIr::AssignIdentifier { .. }));
        assert!(matches!(rhs.expr, ExprIr::Number(bits) if f64::from_bits(bits) == 9.0));
        assert!(matches!(&spread.expr, ExprIr::SpreadArgument(argument)
                if argument.protocol == SpreadArgumentProtocol::ARGUMENT_LIST));
        assert_eq!(result.kind, result_kind);
    }
}

#[test]
fn numeric_conversion_direct_and_candidate_calls_invalidate_captured_facts() {
    for (native, operands, coercion) in [
        ("Number", "input", "valueOf"),
        ("BigInt", "input", "valueOf"),
        ("BigInt.asIntN", "input, 7n", "valueOf"),
        ("BigInt.asUintN", "input, 7n", "valueOf"),
        ("isFinite", "input", "valueOf"),
        ("isNaN", "input", "valueOf"),
    ] {
        for candidate in [false, true] {
            let setup = if candidate {
                format!("const native = unknown ? {native} : Number.isFinite;")
            } else {
                String::new()
            };
            let invocation = if candidate { "native" } else { native };
            for (declaration, mutation, observation) in [
                ("let captured = 1;", "captured = 'changed';", "captured"),
                (
                    "let captured = { value: 1 };",
                    "captured.value = 'changed';",
                    "captured.value",
                ),
                (
                    "let captured = [1];",
                    "captured[0] = 'changed';",
                    "captured[0]",
                ),
            ] {
                let source = format!(
                    "{setup} const input = {{ {coercion}() {{ {mutation} return 2; }} }}; \
                        {declaration} {invocation}({operands}); {observation} + 1;"
                );
                assert_caller_flow_invalidation_reaches_final_addition(&source);
            }
        }
    }
}

#[test]
fn numeric_formatters_invalidate_facts_through_arguments_and_locale_options() {
    for invocation in [
        "(7).toExponential(input)",
        "(7).toFixed(input)",
        "(7).toPrecision(input)",
        "(7).toString(input)",
        "(7n).toString(input)",
    ] {
        let source = format!(
            "const input = {{ valueOf() {{ captured.value = 'changed'; return 2; }} }}; \
                 let captured = {{ value: 1 }}; {invocation}; captured.value + 1;"
        );
        assert_caller_flow_invalidation_reaches_final_addition(&source);
    }
    for receiver in ["7", "7n"] {
        let source = format!(
                "const options = {{ get useGrouping() {{ captured[0] = 'changed'; return false; }} }}; \
                 let captured = [1]; ({receiver}).toLocaleString('en', options); captured[0] + 1;"
            );
        assert_caller_flow_invalidation_reaches_final_addition(&source);
    }
    assert_caller_flow_invalidation_reaches_final_addition(
        "const input = { valueOf() { captured = 'changed'; return 2; } }; \
             let captured = 1; new Number(input); captured + 1;",
    );
}

#[test]
fn math_coercion_families_and_iterator_gets_invalidate_caller_facts() {
    for method in [
        "abs", "acos", "acosh", "asin", "asinh", "atan", "atanh", "cbrt", "ceil", "clz32", "cos",
        "cosh", "exp", "expm1", "f16round", "floor", "fround", "log", "log10", "log1p", "log2",
        "round", "sign", "sin", "sinh", "sqrt", "tan", "tanh", "trunc", "atan2", "hypot", "imul",
        "pow", "min", "max",
    ] {
        for candidate in [false, true] {
            let setup = if candidate {
                format!("const native = unknown ? Math.{method} : Number.isFinite;")
            } else {
                String::new()
            };
            let invocation = if candidate {
                "native".to_string()
            } else {
                format!("Math.{method}")
            };
            let source = format!(
                    "{setup} const input = {{ valueOf() {{ captured.value = 'changed'; return 1; }} }}; \
                     let captured = {{ value: 1 }}; {invocation}(input, 2); captured.value + 1;"
                );
            assert_caller_flow_invalidation_reaches_final_addition(&source);
        }
    }
    assert_caller_flow_invalidation_reaches_final_addition(
        "const iterable = { get [Symbol.iterator]() { captured[0] = 'changed'; \
             return function() { return [1, 2][Symbol.iterator](); }; } }; \
             let captured = [1]; Math.sumPrecise(iterable); captured[0] + 1;",
    );
}

#[test]
fn atomics_preparation_and_wait_arguments_invalidate_caller_facts() {
    for (method, operands) in [
        ("add", "view, input, 1"),
        ("and", "view, input, 1"),
        ("compareExchange", "view, input, 0, 1"),
        ("exchange", "view, input, 1"),
        ("load", "view, input"),
        ("notify", "view, input, 0"),
        ("or", "view, input, 1"),
        ("store", "view, input, 1"),
        ("sub", "view, input, 1"),
        ("wait", "view, input, 0, 0"),
        ("waitAsync", "view, input, 0, 0"),
        ("xor", "view, input, 1"),
        ("isLockFree", "input"),
    ] {
        for candidate in [false, true] {
            let setup = if candidate {
                format!("const native = unknown ? Atomics.{method} : Number.isFinite;")
            } else {
                String::new()
            };
            let invocation = if candidate {
                "native".to_string()
            } else {
                format!("Atomics.{method}")
            };
            let source = format!(
                "{setup} const view = new Int32Array(new SharedArrayBuffer(4)); \
                     const input = {{ valueOf() {{ captured.value = 'changed'; return 0; }} }}; \
                     let captured = {{ value: 1 }}; {invocation}({operands}); captured.value + 1;"
            );
            assert_caller_flow_invalidation_reaches_final_addition(&source);
        }
    }
    for invocation in [
        "Atomics.store(view, 0, input)",
        "Atomics.compareExchange(view, 0, input, input)",
        "Atomics.notify(view, 0, input)",
        "Atomics.wait(view, 0, 0, input)",
        "Atomics.waitAsync(view, 0, 0, input)",
    ] {
        let source = format!(
            "const view = new Int32Array(new SharedArrayBuffer(4)); \
                 const input = {{ valueOf() {{ captured[0] = 'changed'; return 0; }} }}; \
                 let captured = [1]; {invocation}; captured[0] + 1;"
        );
        assert_caller_flow_invalidation_reaches_final_addition(&source);
    }
    assert_caller_flow_invalidation_reaches_final_addition(
        "const view = new BigInt64Array(new SharedArrayBuffer(8)); \
             const input = { valueOf() { captured = 'changed'; return 2n; } }; \
             let captured = 1; Atomics.store(view, 0, input); captured + 1;",
    );
}

#[test]
fn noncoercing_numeric_entries_preserve_caller_facts() {
    for invocation in [
        "Number.isInteger(input)",
        "Number.isSafeInteger(input)",
        "Number.isFinite(input)",
        "Number.isNaN(input)",
        "(7).valueOf(input)",
        "(7n).valueOf(input)",
        "Math.random(input)",
        "Atomics.pause(0, input)",
    ] {
        let source = format!(
            "const input = {{ valueOf() {{ captured.value = 'changed'; return 2; }} }}; \
                 let captured = {{ value: 1 }}; {invocation}; captured.value + 1;"
        );
        assert_caller_flow_preservation_reaches_final_addition(&source);
    }
}
