use lila_front::{parse, ParseOptions};
use lila_ir::{lower, DynamicFunctionKind, PreparedDynamicFunctionOutcome};

#[test]
fn hoisted_hints_preserve_all_256_existing_body_candidates() {
    let bodies = (0..256)
        .map(|index| format!("return {index};"))
        .collect::<Vec<_>>();
    let literals = bodies
        .iter()
        .map(|body| format!("{body:?}"))
        .collect::<Vec<_>>()
        .join(",");
    for prefix in ["", "'use strict';"] {
        for use_source in [
            "var body; Function(body);",
            "outerBody = body; Function(outerBody);",
        ] {
            let parsed = parse(
                format!("{prefix} var outerBody; [{literals}].forEach(function(body) {{ {use_source} }});"),
                ParseOptions::script(),
            ).unwrap();
            let program = lower(&parsed);
            assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
            let script = program.script.unwrap();
            for body in &bodies {
                assert!(
                    script.prepared_dynamic_functions.iter().any(|prepared| {
                        prepared.kind == DynamicFunctionKind::Ordinary
                            && prepared.arguments == [body.as_str()]
                            && matches!(
                                prepared.outcome,
                                PreparedDynamicFunctionOutcome::Compiled { .. }
                            )
                    }),
                    "lost existing body {body:?} in {use_source}"
                );
            }
        }
    }
}

#[test]
fn supplemental_hints_preserve_all_256_existing_complete_tuples() {
    let parameters = (0..16).map(|index| format!("p{index}")).collect::<Vec<_>>();
    let bodies = (0..16)
        .map(|index| format!("return {index};"))
        .collect::<Vec<_>>();
    let choices = |values: &[String]| {
        let (last, previous) = values.split_last().unwrap();
        previous
            .iter()
            .map(|value| format!("choose() ? {value:?} : "))
            .collect::<String>()
            + &format!("{last:?}")
    };
    for prefix in ["", "'use strict';"] {
        let parsed = parse(
            format!("{prefix} function choose() {{ return false; }} var parameter = {}; var body = {}; var body; Function(parameter, body);",
                choices(&parameters), choices(&bodies)),
            ParseOptions::script(),
        ).unwrap();
        let program = lower(&parsed);
        assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        for parameter in &parameters {
            for body in &bodies {
                assert!(
                    script.prepared_dynamic_functions.iter().any(|prepared| {
                        prepared.kind == DynamicFunctionKind::Ordinary
                            && prepared.arguments == [parameter.as_str(), body.as_str()]
                            && matches!(
                                prepared.outcome,
                                PreparedDynamicFunctionOutcome::Compiled { .. }
                            )
                    }),
                    "lost existing tuple {parameter:?}, {body:?}"
                );
            }
        }
    }
}

#[test]
fn body_references_hint_unknown_record_formals_in_each_source_grammar() {
    for (constructor, kind, body) in [
        (
            "Function",
            DynamicFunctionKind::Ordinary,
            "return key1+key2+key3;",
        ),
        (
            "(function*() {}).constructor",
            DynamicFunctionKind::Generator,
            "yield key1+key2+key3;",
        ),
        (
            "(async function() {}).constructor",
            DynamicFunctionKind::Async,
            "return key1+key2+key3;",
        ),
        (
            "(async function*() {}).constructor",
            DynamicFunctionKind::AsyncGenerator,
            "yield key1+key2+key3;",
        ),
    ] {
        for (arguments, expected) in [
            ("p,p,p", vec!["key1", "key2", "key3", body]),
            ("p+','+p,p", vec!["key1,key2", "key3", body]),
            ("p+','+p+','+p", vec!["key1,key2,key3", body]),
        ] {
            let parsed = parse(
                format!("var counter=0; var p={{toString:function(){{return 'key'+(++counter);}}}}; {constructor}({arguments}, {body:?});"),
                ParseOptions::script(),
            ).unwrap();
            let program = lower(&parsed);
            assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
            assert!(
                program
                    .script
                    .unwrap()
                    .prepared_dynamic_functions
                    .iter()
                    .any(|prepared| {
                        prepared.kind == kind
                            && prepared.arguments == expected
                            && matches!(
                                prepared.outcome,
                                PreparedDynamicFunctionOutcome::Compiled { .. }
                            )
                    }),
                "missing {kind:?} body-reference tuple {expected:?}"
            );
        }
    }
}

#[test]
fn supplemental_source_hints_do_not_create_eager_module_requests() {
    use lila_ir::{HostSurfacePolicy, LoweringSession, ModuleRequestKeyIr};
    let parsed = parse(
        r#"
var realm = new ShadowRealm();
function load(source) { var source; realm.importValue(source, 'value'); }
load('./live.js');
var counter=0;
var p={toString:function(){return 'parameter'+(++counter);}};
Function(p, "return parameter1; new ShadowRealm().importValue('./unrequested.js', 'value');");
Function(p, counter ? "return parameter1;" : "return parameter1; new ShadowRealm().importValue('./also-unrequested.js', 'value');");
Function(p, "return parameter1; Function(\"return new ShadowRealm().importValue('./nested-unrequested.js', 'value');\")();");
"#,
        ParseOptions::script(),
    ).unwrap();
    let session = LoweringSession::default();
    let program = session.lower_source(&parsed, HostSurfacePolicy::default());
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    assert_eq!(
        session.realm_module_requests(),
        [ModuleRequestKeyIr::plain("./live.js")]
    );
}

#[test]
fn forward_var_declarations_retain_undefined_source_candidates_after_calls() {
    for (source, arguments) in [
        ("try { new Function(body); } catch (error) {} var body;", vec!["undefined"]),
        ("var parameters = 'a,b,c'; try { new Function(parameters, body); } catch (error) {} var body;", vec!["a,b,c", "undefined"]),
        ("var body = 'return 1.1;'; try { new Function(parameters, body); } catch (error) {} var parameters;", vec!["undefined", "return 1.1;"]),
    ] {
        for prefix in ["", "'use strict';"] {
          for source in [
            format!("{prefix} eval(null); {source}"),
            format!("{prefix} function create() {{ eval(null); {source} }} create();"),
          ] {
            let parsed = parse(
                source,
                ParseOptions::script(),
            )
            .unwrap();
            let program = lower(&parsed);
            assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
            assert!(program.script.unwrap().prepared_dynamic_functions.iter().any(|prepared| {
                prepared.kind == DynamicFunctionKind::Ordinary
                    && prepared.arguments == arguments
                    && matches!(prepared.outcome, PreparedDynamicFunctionOutcome::Compiled { .. })
            }), "missing forward var candidate {arguments:?}");
          }
        }
    }
}

#[test]
fn declared_function_constructor_properties_retain_each_source_grammar() {
    for (declaration, kind) in [
        ("function f() {}", DynamicFunctionKind::Ordinary),
        ("function* f() {}", DynamicFunctionKind::Generator),
        ("async function f() {}", DynamicFunctionKind::Async),
        (
            "async function* f() {}",
            DynamicFunctionKind::AsyncGenerator,
        ),
    ] {
        let parsed = parse(
            format!("{declaration} var Constructor = f.constructor; Constructor('a', ' /* a */ b, c /* b */ //', '/* c */ ; /* d */ //');"),
            ParseOptions::script(),
        ).unwrap();
        let program = lower(&parsed);
        assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
        assert!(
            program
                .script
                .unwrap()
                .prepared_dynamic_functions
                .iter()
                .any(|prepared| {
                    prepared.kind == kind
                        && prepared.arguments
                            == ["a", " /* a */ b, c /* b */ //", "/* c */ ; /* d */ //"]
                        && matches!(
                            prepared.outcome,
                            PreparedDynamicFunctionOutcome::Compiled { .. }
                        )
                }),
            "missing declared constructor candidate for {kind:?}"
        );
    }
}

#[test]
fn final_binding_patterns_prepare_compiled_function_units_for_each_constructor_kind() {
    for (constructor, kind) in [
        ("Function", DynamicFunctionKind::Ordinary),
        (
            "(function* () {}).constructor",
            DynamicFunctionKind::Generator,
        ),
        (
            "(async function () {}).constructor",
            DynamicFunctionKind::Async,
        ),
        (
            "(async function* () {}).constructor",
            DynamicFunctionKind::AsyncGenerator,
        ),
    ] {
        for parameters in ["{value}", "[value]"] {
            let parsed = parse(
                format!("var Constructor = {constructor}; Constructor('{parameters}', 'return value;');"),
                ParseOptions::script(),
            )
            .unwrap();
            let program = lower(&parsed);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            assert!(
                program
                    .script
                    .unwrap()
                    .prepared_dynamic_functions
                    .iter()
                    .any(|prepared| {
                        prepared.kind == kind
                            && prepared.arguments == [parameters, "return value;"]
                            && matches!(
                                prepared.outcome,
                                PreparedDynamicFunctionOutcome::Compiled { .. }
                            )
                    }),
                "{kind:?}({parameters}) must compile instead of deferring a SyntaxError"
            );
        }
    }
}

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

#[test]
fn captured_derived_body_candidates_keep_every_original_strict_rejection() {
    let source = include_str!(
        "../../lila-engine/tests/fixtures/finite_capture_sources/escaped_strict_bodies.js"
    );
    let bodies = [
        "l\\u0065t: 42;",
        "if (1) l\\u0065t: 42;",
        "l\\u0065t = 42;",
        "if (1) l\\u0065t = 42;",
        "st\\u0061tic: 42;",
        "if (1) st\\u0061tic: 42;",
        "st\\u0061tic = 42;",
        "if (1) st\\u0061tic = 42;",
    ];
    for prefix in ["", "'use strict';"] {
        let parsed = parse(format!("{prefix}{source}"), ParseOptions::script()).unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let prepared = program.script.unwrap().prepared_dynamic_functions;
        for body in bodies {
            let plain = prepared
                .iter()
                .find(|entry| entry.arguments == [body])
                .unwrap_or_else(|| panic!("missing original body {body:?}"));
            assert!(matches!(
                plain.outcome,
                PreparedDynamicFunctionOutcome::Compiled { .. }
            ));
            for directive in [" 'use strict'; ", " 'use strict' \n "] {
                let strict = format!("{directive}{body}");
                let rejected = prepared
                    .iter()
                    .find(|entry| entry.arguments == [strict.as_str()])
                    .unwrap_or_else(|| panic!("missing captured strict body {strict:?}"));
                assert!(matches!(
                    rejected.outcome,
                    PreparedDynamicFunctionOutcome::SyntaxError { .. }
                ));
            }
        }
    }
}

#[test]
fn captured_source_hints_do_not_seed_unobserved_shadowing_parameters() {
    for source in [
        "var source = 'return 997;'; function unobserved(source) { return () => Function(source); }",
        "function unused() { var source = 'return 997;'; return () => source; } function unobserved(source) { return () => Function(source); }",
    ] {
        let parsed = parse(source, ParseOptions::script()).unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        assert!(!program.script.unwrap().prepared_dynamic_functions.iter()
            .any(|entry| entry.arguments == ["return 997;"]), "unrelated binding seeded a constructor: {source}");
    }
}

#[test]
fn parameter_and_body_cells_keep_separate_captured_source_candidates() {
    let source = r#"
function build(source = 'return 71;', early = () => Function(source)()) {
  var source = 'return 72;';
  return [early, () => Function(source)()];
}
var reads = build(); reads[0](); reads[1]();
"#;
    let parsed = parse(source, ParseOptions::script()).unwrap();
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let prepared = program.script.unwrap().prepared_dynamic_functions;
    for body in ["return 71;", "return 72;"] {
        assert!(
            prepared.iter().any(|entry| entry.arguments == [body]
                && matches!(
                    entry.outcome,
                    PreparedDynamicFunctionOutcome::Compiled { .. }
                )),
            "missing separate cell {body:?}"
        );
    }
}

#[test]
fn captured_source_hints_retain_the_existing_256_body_boundary() {
    let bodies = (0..256)
        .map(|index| format!("return {index};"))
        .collect::<Vec<_>>();
    let literals = bodies
        .iter()
        .map(|body| format!("{body:?}"))
        .collect::<Vec<_>>()
        .join(",");
    let parsed = parse(format!("[{literals}].forEach(function(body) {{ var captured = body; var read = () => Function(captured); read(); }});"), ParseOptions::script()).unwrap();
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let prepared = program.script.unwrap().prepared_dynamic_functions;
    for body in bodies {
        assert!(
            prepared
                .iter()
                .any(|entry| entry.arguments == [body.as_str()]
                    && matches!(
                        entry.outcome,
                        PreparedDynamicFunctionOutcome::Compiled { .. }
                    )),
            "lost captured boundary body {body:?}"
        );
    }
}

#[test]
fn global_recursive_and_mutual_calls_keep_each_captured_body_candidate() {
    for (source, expected) in [
        (
            r#"
function recursive(depth, source) {
  var text = source; var read = () => Function(text)();
  if (depth) { var next = recursive(0, 'return 82;'); return [read, next[0]]; }
  return [read];
}
var reads = recursive(1, 'return 81;'); reads[0](); reads[1]();
"#,
            vec!["return 81;", "return 82;"],
        ),
        (
            r#"
function first(depth, source) {
  var text = source; var read = () => Function(text)();
  if (depth) { var next = second(depth - 1, 'return 92;'); return [read, next[0], next[1]]; }
  return [read];
}
function second(depth, source) {
  var text = source; var read = () => Function(text)();
  if (depth) { var next = first(depth - 1, 'return 93;'); return [read, next[0]]; }
  return [read];
}
var reads = first(2, 'return 91;'); reads[0](); reads[1](); reads[2]();
"#,
            vec!["return 91;", "return 92;", "return 93;"],
        ),
    ] {
        for directive in ["", "'use strict';"] {
            let parsed = parse(format!("{directive}{source}"), ParseOptions::script()).unwrap();
            let program = lower(&parsed);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let prepared = program.script.unwrap().prepared_dynamic_functions;
            for body in &expected {
                assert!(
                    prepared.iter().any(|entry| entry.arguments == [*body]
                        && matches!(
                            entry.outcome,
                            PreparedDynamicFunctionOutcome::Compiled { .. }
                        )),
                    "missing recursive candidate {body:?}"
                );
            }
        }
    }
}

#[test]
fn global_source_target_discovery_does_not_cross_shadowed_or_dynamic_references() {
    let definitions = r#"
function build(body) { return () => Function(body)(); }
build('return 91;')();
"#;
    for source in [
        "function local(build) { build('return 997;'); } local(function ignored(value) { return value; });",
        "function local() { let build = function ignored(value) { return value; }; build('return 998;'); } local();",
        "function local() { var build = function ignored(value) { return value; }; return () => build('return 999;'); } local()();",
        "function local() { with ({ build: function ignored(value) { return value; } }) { build('return 996;'); } } local();",
        "function local() { var build = function ignored(value) { return value; }; eval(''); build('return 995;'); } local();",
    ] {
        let parsed = parse(format!("{definitions}{source}"), ParseOptions::script()).unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let prepared = program.script.unwrap().prepared_dynamic_functions;
        assert!(prepared.iter().any(|entry|entry.arguments == ["return 91;"]));
        for body in ["return 997;","return 998;","return 999;","return 996;","return 995;"] {
            assert!(!prepared.iter().any(|entry|entry.arguments == [body]),
                "shadowed reference seeded unused global body {body:?}: {source}");
        }
    }
}

#[test]
fn source_call_targets_do_not_cross_current_or_captured_with_references() {
    let definitions = r#"
function build(body) { return () => Function(body)(); }
build('return 91;')();
"#;
    for source in [
        "function local() { with ({ build: function ignored(value) { return value; } }) { return () => build('return 771;'); } } local()();",
        "with (Object.create({ build: function ignored(value) { return value; } })) { build('return 771;'); }",
        "function local(build) { with ({ build: function ignored(value) { return value; } }) { build('return 771;'); } } local(function selected(body) { return Function(body); });",
        "with ({}) { const build = function selected(body) { return Function(body); }; with ({ build: function ignored(value) { return value; } }) { build('return 771;'); } }",
    ] {
        let parsed = parse(format!("{definitions}{source}"), ParseOptions::script()).unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let prepared = program.script.unwrap().prepared_dynamic_functions;
        assert!(prepared.iter().any(|entry| entry.arguments == ["return 91;"]));
        assert!(
            !prepared.iter().any(|entry| entry.arguments == ["return 771;"]),
            "an intercepted Reference forwarded source to a fallback target: {source}"
        );
    }
}

#[test]
fn source_property_targets_do_not_borrow_with_shadowed_binding_shapes() {
    let definitions = r#"
function build(body) { return () => Function(body)(); }
var registry = { build: build };
registry.build('return 81;')();
"#;
    for source in [
        "with ({ registry: { build: function ignored(value) { return value; } } }) { registry.build('return 781;'); }",
        "function local() { with ({ registry: { build: function ignored(value) { return value; } } }) { return () => registry.build('return 781;'); } } local()();",
        "function local(registry) { with ({ registry: { build: function ignored(value) { return value; } } }) { registry['build']('return 781;'); } } local({ build: build });",
    ] {
        let parsed = parse(format!("{definitions}{source}"), ParseOptions::script()).unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let prepared = program.script.unwrap().prepared_dynamic_functions;
        assert!(prepared.iter().any(|entry| entry.arguments == ["return 81;"]));
        assert!(
            !prepared.iter().any(|entry| entry.arguments == ["return 781;"]),
            "a shadowed base borrowed a fallback property's call target: {source}"
        );
    }
}

#[test]
fn lexical_source_targets_precede_current_and_captured_with_objects() {
    for call in [
        "build('return 791;')();",
        "const invoke = () => build('return 791;')(); invoke();",
        "const registry = { build: build }; registry.build('return 791;')();",
    ] {
        let parsed = parse(
            format!(
                "with ({{ build: function ignored(value) {{ return value; }} }}) {{\n\
                 const build = function selected(body) {{ return Function(body); }}; {call} }}"
            ),
            ParseOptions::script(),
        )
        .unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        assert!(
            program
                .script
                .unwrap()
                .prepared_dynamic_functions
                .iter()
                .any(|entry| {
                    entry.arguments == ["return 791;"]
                        && matches!(
                            entry.outcome,
                            PreparedDynamicFunctionOutcome::Compiled { .. }
                        )
                }),
            "a nearer lexical Reference lost its source candidate: {call}"
        );
    }
}

#[test]
fn supplemental_global_call_sources_preserve_all_256_primary_bodies() {
    let bodies = (0..256)
        .map(|index| format!("return {index};"))
        .collect::<Vec<_>>();
    for (parameter, argument, supplemental) in [
        ("body", "body", "'return 997;'"),
        ("record", "record.body", "{body: 'return 997;'}"),
    ] {
        let literals = bodies
            .iter()
            .map(|body| {
                if parameter == "record" {
                    format!("{{body: {body:?}}}")
                } else {
                    format!("{body:?}")
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        let source = format!(
            r#"
function build({parameter}) {{ return () => Function({argument})(); }}
for (var value of [{literals}]) {{ build(value); }}
function indirect() {{ build({supplemental}); }}
indirect();
"#
        );
        let parsed = parse(source, ParseOptions::script()).unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let prepared = program.script.unwrap().prepared_dynamic_functions;
        for body in &bodies {
            assert!(
                prepared
                    .iter()
                    .any(|entry| entry.arguments == [body.as_str()]
                        && matches!(
                            entry.outcome,
                            PreparedDynamicFunctionOutcome::Compiled { .. }
                        )),
                "supplemental global discovery displaced primary {body:?} from {parameter}"
            );
        }
    }
}

#[test]
fn finite_for_of_identifier_hints_keep_the_original_256_candidate_budget() {
    for count in [256, 257] {
        let bodies = (0..count)
            .map(|index| format!("return {index};"))
            .collect::<Vec<_>>();
        let literals = bodies
            .iter()
            .map(|body| format!("{body:?}"))
            .collect::<Vec<_>>()
            .join(",");
        for head in ["var body", "let body", "const body"] {
            let parsed = parse(
                format!("for ({head} of [{literals}]) {{ Function(body); }}"),
                ParseOptions::script(),
            )
            .unwrap();
            let program = lower(&parsed);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.unwrap();
            assert!(
                script.body.statements.iter().any(|statement| matches!(
                    statement,
                    lila_ir::StatementIr::ForOfIterator { .. }
                )),
                "source discovery must retain the actual generic iterator"
            );
            if count == 257 {
                // The actual750c diagnostic retains the older optional
                // "undefined" var-hoist tuple only. It admits none of these
                //257 real iteration strings; table emptiness is not the limit.
                assert!(
                    script.prepared_dynamic_functions.len() <= 256,
                    "source tuple admission must retain the original256 cap: {head}"
                );
                let unmatched = bodies
                    .iter()
                    .filter(|body| {
                        !script
                            .prepared_dynamic_functions
                            .iter()
                            .any(|entry| entry.arguments == [body.as_str()])
                    })
                    .collect::<Vec<_>>();
                assert!(!unmatched.is_empty(),
                    "at least one of the exact257 tuples must retain unmatched runtime dispatch: {head}");
                fn live_constructor_call(statement: &lila_ir::StatementIr) -> bool {
                    match statement {
                        lila_ir::StatementIr::Block(block) => {
                            block.statements.iter().any(live_constructor_call)
                        }
                        lila_ir::StatementIr::LexicalBlock(statements) => {
                            statements.iter().any(live_constructor_call)
                        }
                        lila_ir::StatementIr::Expression(expression) => match &expression.expr {
                            lila_ir::ExprIr::CallNamed { args, .. }
                            | lila_ir::ExprIr::CallIndirect { args, .. } => {
                                args.len() == 1 && args[0].kind == lila_ir::ValueKind::Dynamic
                            }
                            _ => false,
                        },
                        _ => false,
                    }
                }
                let loop_body = script
                    .body
                    .statements
                    .iter()
                    .find_map(|statement| {
                        if let lila_ir::StatementIr::ForOfIterator { body, .. } = statement {
                            Some(body.as_ref())
                        } else {
                            None
                        }
                    })
                    .expect("the exact original generic iterator body");
                assert!(live_constructor_call(loop_body),
                    "the original live constructor operand must select an exact runtime tuple: {head}");
            } else {
                for body in &bodies {
                    assert!(
                        script
                            .prepared_dynamic_functions
                            .iter()
                            .any(|entry| entry.arguments == [body.as_str()]
                                && matches!(
                                    entry.outcome,
                                    PreparedDynamicFunctionOutcome::Compiled { .. }
                                )),
                        "missing {head} iteration candidate {body:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn synthetic_class_constructor_sources_preserve_each_grammar_and_original_construct() {
    use lila_ir::{ExprIr, StatementIr};

    for (intrinsic, kind) in [
        ("Function", DynamicFunctionKind::Ordinary),
        (
            "Object.getPrototypeOf(function* () {}).constructor",
            DynamicFunctionKind::Generator,
        ),
        (
            "Object.getPrototypeOf(async function () {}).constructor",
            DynamicFunctionKind::Async,
        ),
        (
            "Object.getPrototypeOf(async function* () {}).constructor",
            DynamicFunctionKind::AsyncGenerator,
        ),
    ] {
        for declaration in [
            "class Created extends Base {}".to_string(),
            "var Created = class extends Base {};".to_string(),
            "let Created = class Named extends Base {};".to_string(),
            "class First extends Base {} class Created extends First {}".to_string(),
        ] {
            for prefix in ["", "'use strict';"] {
                let parsed = parse(
                    format!("{prefix} var Base = {intrinsic}; {declaration} new Created('a', 'b', 'return a + b');"),
                    ParseOptions::script(),
                )
                .unwrap();
                let program = lower(&parsed);
                assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
                let script = program.script.unwrap();
                assert!(
                    script.prepared_dynamic_functions.iter().any(|prepared| {
                        prepared.kind == kind
                            && prepared.arguments == ["a", "b", "return a + b"]
                            && matches!(
                                prepared.outcome,
                                PreparedDynamicFunctionOutcome::Compiled { .. }
                            )
                    }),
                    "missing class source for {kind:?}: {declaration}"
                );
                let StatementIr::Expression(expression) = script.body.statements.last().unwrap()
                else {
                    panic!("the caller retains the original construction");
                };
                let ExprIr::Construct { callee, args, .. } = &expression.expr else {
                    panic!(
                        "source discovery cannot replace Construct: {:?}",
                        expression.expr
                    );
                };
                assert!(
                    matches!(
                        callee.expr,
                        ExprIr::EnvironmentIdentifier(_)
                            | ExprIr::Identifier(_)
                            | ExprIr::GlobalIdentifierRead { .. }
                            | ExprIr::GlobalPropertyRead { .. }
                    ),
                    "the actual Created Reference must remain live: {:?}",
                    callee.expr
                );
                assert_eq!(args.len(), 3);
                for (argument, expected) in args.iter().zip(["a", "b", "return a + b"]) {
                    assert!(matches!(&argument.expr, ExprIr::String(actual) if actual == expected));
                }
                assert!(script.functions.iter().any(|function| {
                    function.is_synthetic_default_derived_constructor
                        && function.is_derived_constructor
                }));
            }
        }
        let parsed = parse(
            format!("new (class extends {intrinsic} {{}})('value', 'return value;');"),
            ParseOptions::script(),
        )
        .unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        assert!(program
            .script
            .unwrap()
            .prepared_dynamic_functions
            .iter()
            .any(|prepared| {
                prepared.kind == kind
                    && prepared.arguments == ["value", "return value;"]
                    && matches!(
                        prepared.outcome,
                        PreparedDynamicFunctionOutcome::Compiled { .. }
                    )
            }));
    }
}

#[test]
fn synthetic_class_source_hints_follow_resolved_cells_and_with_reference_order() {
    for use_source in [
        "function invoke(Created) { new Created('return 997;'); } invoke(function Other(body) {});",
        "{ class Created {} new Created('return 997;'); }",
        "with ({ Created: function Other(body) {} }) { new Created('return 997;'); }",
        "with ({ Created: function Other(body) {} }) { const invoke = () => new Created('return 997;'); invoke(); }",
        "with ({ Base: function Other(body) {} }) { class Intercepted extends Base {} new Intercepted('return 997;'); }",
    ] {
        let parsed = parse(
            format!("var Base = Function; class Created extends Base {{}} new Created('return 91;'); {use_source}"),
            ParseOptions::script(),
        )
        .unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let prepared = program.script.unwrap().prepared_dynamic_functions;
        assert!(prepared.iter().any(|entry| entry.arguments == ["return 91;"]));
        assert!(!prepared.iter().any(|entry| entry.arguments == ["return 997;"]),
            "shadowed Reference borrowed the class grammar: {use_source}");
    }
    let parsed = parse(
        "with ({ Created: function Other(body) {} }) { class Created extends Function {} const invoke = () => new Created('return 92;'); invoke(); }",
        ParseOptions::script(),
    )
    .unwrap();
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    assert!(program
        .script
        .unwrap()
        .prepared_dynamic_functions
        .iter()
        .any(|entry| { entry.arguments == ["return 92;"] }));
}

#[test]
fn synthetic_class_discovery_keeps_unknown_and_explicit_constructor_sources_unprepared() {
    for source in [
        "class Created extends Function {} new Created(runtimeBody);",
        "class Created extends runtimeBase {} new Created('return 93;');",
        "class Created extends Function { constructor(body) { return {}; } } new Created('return 94;');",
    ] {
        let parsed = parse(source, ParseOptions::script()).unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        assert!(program.script.unwrap().prepared_dynamic_functions.is_empty(),
            "discovery guessed a source tuple: {source}");
    }
}
