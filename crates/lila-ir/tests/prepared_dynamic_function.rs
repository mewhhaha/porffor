use lila_front::{parse, ParseOptions};
use lila_ir::{lower, DynamicFunctionKind, PreparedDynamicFunctionOutcome};

#[test]
fn nested_global_constructor_calls_keep_finite_body_and_parameter_candidates() {
    for body in ["x = await 42", "x = yield"] {
        for caller in ["function", "async function*"] {
            let source = parse(
                format!(
                    "eval(null); var Constructor = Object.getPrototypeOf(async function*() {{}}).constructor;\n\
                     Constructor({body:?}); eval(null);\n\
                     {caller} reject() {{ Constructor({body:?}, ''); }}"
                ),
                ParseOptions::script(),
            )
            .unwrap();
            let program = lower(&source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.unwrap();
            for (arguments, syntax_error) in [(vec![body], false), (vec![body, ""], true)] {
                let prepared = script
                    .prepared_dynamic_functions
                    .iter()
                    .find(|prepared| {
                        prepared.kind == DynamicFunctionKind::AsyncGenerator
                            && prepared.arguments == arguments
                    })
                    .unwrap_or_else(|| panic!("missing {caller} candidate {arguments:?}"));
                assert_eq!(
                    matches!(
                        prepared.outcome,
                        PreparedDynamicFunctionOutcome::SyntaxError { .. }
                    ),
                    syntax_error,
                    "{caller}: {arguments:?}"
                );
            }
        }
    }
}

#[test]
fn nested_global_source_values_remain_finite_compilation_candidates() {
    let source = parse(
        "eval(null); var body = 'return 9;'; eval(null); function build() { return Function(body); } build();",
        ParseOptions::script(),
    )
    .unwrap();
    let program = lower(&source);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    assert!(program
        .script
        .unwrap()
        .prepared_dynamic_functions
        .iter()
        .any(|prepared| {
            prepared.kind == DynamicFunctionKind::Ordinary
                && prepared.arguments == ["return 9;"]
                && matches!(
                    prepared.outcome,
                    PreparedDynamicFunctionOutcome::Compiled { .. }
                )
        }));
}

#[test]
fn named_environment_calls_preserve_possible_derived_constructor_source_candidates() {
    for (expression, body, kind) in [
        ("function*() {}", "yield 1;", DynamicFunctionKind::Generator),
        (
            "async function() {}",
            "return 1;",
            DynamicFunctionKind::Async,
        ),
        (
            "async function*() {}",
            "yield 1;",
            DynamicFunctionKind::AsyncGenerator,
        ),
    ] {
        for initializer in [
            format!("Object.getPrototypeOf({expression}).constructor"),
            format!("Reflect.getPrototypeOf({expression}).constructor"),
            format!("({expression}).constructor"),
        ] {
            let source = parse(
                format!("eval(null); var Constructor = {initializer}; eval(null); Constructor('{body}');"),
                ParseOptions::script(),
            ).unwrap();
            let program = lower(&source);
            assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
            let script = program.script.unwrap();
            assert!(
                script.prepared_dynamic_functions.iter().any(|prepared| {
                    prepared.kind == kind
                        && prepared.arguments == [body]
                        && matches!(
                            prepared.outcome,
                            PreparedDynamicFunctionOutcome::Compiled { .. }
                        )
                }),
                "missing optional source for {kind:?}: {initializer}"
            );
        }
    }
}

#[test]
fn static_bodies_are_independent_functions_with_unique_nested_identities() {
    let source = parse(
        r#"let value = 99; function caller() { return Function('value', 'return () => value;'); } Function('return 1;'); Function('return 2;');"#,
        ParseOptions::script(),
    ).unwrap();
    let program = lower(&source);
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    assert_eq!(script.prepared_dynamic_functions.len(), 3);
    let ids = script
        .functions
        .iter()
        .map(|function| &function.id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), script.functions.len());
    for prepared in &script.prepared_dynamic_functions {
        assert_eq!(prepared.kind, DynamicFunctionKind::Ordinary);
        let PreparedDynamicFunctionOutcome::Compiled { function_id } = &prepared.outcome else {
            panic!("valid source must be compiled")
        };
        let function = script
            .functions
            .iter()
            .find(|function| &function.id == function_id)
            .unwrap();
        assert!(function.captured_bindings.is_empty());
        assert_eq!(function.name, "anonymous");
        assert!(!function.is_named_expression);
    }
}

#[test]
fn malformed_static_body_is_deferred_to_the_constructor_call() {
    let source = parse("if (false) Function('return }');", ParseOptions::script()).unwrap();
    let program = lower(&source);
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    // Dead source calls may disappear before capability preparation.
    for prepared in program.script.unwrap().prepared_dynamic_functions {
        assert!(matches!(
            prepared.outcome,
            PreparedDynamicFunctionOutcome::SyntaxError { .. }
        ));
    }
    let source = parse(
        "try { Function('return }'); } catch (error) {}",
        ParseOptions::script(),
    )
    .unwrap();
    let program = lower(&source);
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    assert!(matches!(
        program.script.unwrap().prepared_dynamic_functions[0].outcome,
        PreparedDynamicFunctionOutcome::SyntaxError { .. }
    ));
}

#[test]
fn nested_function_source_registers_its_own_independent_body() {
    let source = parse(
        "Function(\"return Function('return 9;');\");",
        ParseOptions::script(),
    )
    .unwrap();
    let program = lower(&source);
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    assert_eq!(script.prepared_dynamic_functions.len(), 2);
    let ids = script
        .functions
        .iter()
        .map(|function| &function.id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), script.functions.len());
}

#[test]
fn member_constructor_candidates_preserve_unknown_callable_dispatch() {
    let source = parse(
        r#"var other = $262.createRealm().global; other.Function('return 1;'); new other['Function']('return 2;'); var replacement = { Function() { return 3; } }; replacement.Function('return }');"#,
        ParseOptions::script(),
    ).unwrap();
    let program = lower(&source);
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    let prepared = program.script.unwrap().prepared_dynamic_functions;
    assert!(prepared
        .iter()
        .any(|entry| entry.arguments == ["return 1;"]));
    assert!(prepared
        .iter()
        .any(|entry| entry.arguments == ["return 2;"]));
    assert!(prepared.iter().any(|entry| matches!(
        entry.outcome,
        PreparedDynamicFunctionOutcome::SyntaxError { .. }
    )));
}

#[test]
fn an_unproven_member_candidate_does_not_reject_a_runtime_dependent_source() {
    let source = parse(
        r#"var replacement = { Function() { return 3; } }; replacement.Function('return eval(runtimeSource);');"#,
        ParseOptions::script(),
    ).unwrap();
    let program = lower(&source);
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
}

#[test]
fn primitive_and_binding_candidates_retain_runtime_constructor_invocation() {
    for (source, expected) in [
        ("var body = 'return 9;'; Function(body);", "return 9;"),
        ("Function(null);", "null"),
        ("Function(undefined);", "undefined"),
        ("Function(void 0);", "undefined"),
        ("Function(hoisted); var hoisted;", "undefined"),
        ("Function(1);", "1"),
    ] {
        let source = parse(source, ParseOptions::script()).unwrap();
        let program = lower(&source);
        assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
        let prepared = program.script.unwrap().prepared_dynamic_functions;
        assert!(
            prepared.iter().any(|entry| entry.arguments == [expected]),
            "missing {expected}: {prepared:?}"
        );
    }
    let source = parse("Function(runtimeSource);", ParseOptions::script()).unwrap();
    let program = lower(&source);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    assert!(program
        .script
        .unwrap()
        .prepared_dynamic_functions
        .is_empty());
}

#[test]
fn boxed_and_object_coercion_candidates_keep_primitive_facts_separate() {
    for (source, expected) in [
        (
            "var body = Object('return 9;'); Function(body);",
            "return 9;",
        ),
        (
            "const body = new String('return 8;'); Function(body);",
            "return 8;",
        ),
        ("let body = Object(1); Function(body);", "1"),
        (
            "var body = {toString: function () { return 'return 7;'; }}; Function(body);",
            "return 7;",
        ),
        (
            "const body = {toString() { return 'return 6;'; }}; Function(body);",
            "return 6;",
        ),
        ("Function({});", "[object Object]"),
    ] {
        let parsed = parse(source, ParseOptions::script()).unwrap();
        let program = lower(&parsed);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let prepared = program.script.unwrap().prepared_dynamic_functions;
        assert!(
            prepared.iter().any(|entry| entry.arguments == [expected]),
            "{source}: {prepared:?}"
        );
    }
}

#[test]
fn finite_literal_records_prepare_each_callback_source_without_replacing_the_call() {
    for source in [
        "const entries = [{source: '41'}, {source: '42'}]; entries.forEach(entry => Function('return ' + entry.source)());",
        "[{source: '41'}, {source: '42'}].map(function (entry) { return Function('return ' + entry.source)(); });",
        "const entries = ['41', '42']; entries.forEach(entry => Function('return ' + entry)());",
        "const entries = [{source: '41'}, {source: '42'}]; Object.defineProperty(entries[0], 'source', {get() { return '42'; }}); entries.forEach(entry => print(Function('return ' + entry.source)())); entries.forEach = function(callback) {}; entries.forEach(entry => Function('return ' + entry.source)());",
    ] {
        let parsed = parse(source, ParseOptions::script()).unwrap();
        let program = lower(&parsed);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let prepared = program.script.unwrap().prepared_dynamic_functions;
        for expected in ["return 41", "return 42"] {
            assert!(
                prepared.iter().any(|entry| entry.arguments == [expected]),
                "{source}: {prepared:?}"
            );
        }
    }
}

#[test]
fn source_candidate_expansion_is_bounded_without_claiming_runtime_support() {
    let entries = (0..257)
        .map(|number| format!("'{number}'"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!("[{entries}].forEach(entry => Function('return ' + entry)());");
    let parsed = parse(&source, ParseOptions::script()).unwrap();
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    assert!(program
        .script
        .unwrap()
        .prepared_dynamic_functions
        .is_empty());
}
