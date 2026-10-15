use lila_front::{parse, ParseOptions};
use lila_ir::{lower, DynamicFunctionKind, PreparedDynamicFunctionOutcome};

#[test]
fn prepared_function_children_keep_canonical_global_constructor_grammars() {
    for prefix in ["", "'use strict';"] {
        let parsed = parse(
            format!(
                r#"{prefix}
var Ordinary = Function;
var Generator = (function* () {{}}).constructor;
var Async = (async function () {{}}).constructor;
var AsyncGenerator = (async function* () {{}}).constructor;
Function("return Ordinary('return 81;');");
Function("return Generator('return 81;');");
Function("return Async('return 81;');");
Function("return AsyncGenerator('return 81;');");
Function("return Generator('yield', 'return yield');");
"#
            ),
            ParseOptions::script(),
        )
        .unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let functions = program.script.unwrap().prepared_dynamic_functions;
        for kind in DynamicFunctionKind::ALL {
            assert!(
                functions.iter().any(|entry| entry.kind == *kind
                    && entry.arguments == ["return 81;"]
                    && matches!(
                        entry.outcome,
                        PreparedDynamicFunctionOutcome::Compiled { .. }
                    )),
                "missing canonical {kind:?} global grammar"
            );
        }
        assert!(functions
            .iter()
            .any(|entry| entry.kind == DynamicFunctionKind::Generator
                && entry.arguments == ["yield", "return yield"]
                && matches!(
                    entry.outcome,
                    PreparedDynamicFunctionOutcome::SyntaxError { .. }
                )));
    }
}

#[test]
fn prepared_global_constructor_hints_cover_unmodified_generator_runtime_source() {
    const SOURCE: &str =
        include_str!("../../../test262/vendor/test262/test/staging/sm/generators/runtime.js");
    for prefix in ["", "'use strict';"] {
        let parsed = parse(format!("{prefix}{SOURCE}"), ParseOptions::script()).unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let functions = program.script.unwrap().prepared_dynamic_functions;
        for arguments in [
            vec!["yield", "return yield"],
            vec!["\"use strict\"; with (x) return foo;"],
        ] {
            assert!(
                functions
                    .iter()
                    .any(|entry| entry.kind == DynamicFunctionKind::Generator
                        && entry.arguments == arguments
                        && matches!(
                            entry.outcome,
                            PreparedDynamicFunctionOutcome::SyntaxError { .. }
                        )),
                "missing generator syntax rejection for {arguments:?}"
            );
        }
        assert!(functions
            .iter()
            .any(|entry| entry.kind == DynamicFunctionKind::Generator
                && entry.arguments.is_empty()
                && matches!(
                    entry.outcome,
                    PreparedDynamicFunctionOutcome::Compiled { .. }
                )));
    }
}

#[test]
fn prepared_global_constructor_hints_stop_at_shadow_with_and_local_cells() {
    let parsed = parse(
        r#"
var Factory = (function* () {}).constructor;
Function('Factory', "return Factory('return 997;');");
Function("var Factory; return Factory('return 998;');");
Function("with (globalThis.unknown) { Factory('return 999;'); }");
function owner() {
    var LocalOnly = (function* () {}).constructor;
    Function("return LocalOnly('return 996;');");
}
owner();
"#,
        ParseOptions::script(),
    )
    .unwrap();
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    for entry in program.script.unwrap().prepared_dynamic_functions {
        assert!(
            !["return 996;", "return 997;", "return 998;", "return 999;"]
                .iter()
                .any(|body| entry.kind == DynamicFunctionKind::Generator
                    && entry.arguments == [*body]),
            "an unrelated local or Object Environment borrowed a global grammar hint"
        );
    }
}

#[test]
fn supplemental_global_constructor_sources_never_authorize_host_requests() {
    use lila_ir::{HostSurfacePolicy, LoweringSession};
    let parsed = parse(
        r#"
var Factory = (function* () {}).constructor;
Function("return Factory(\"yield realm.importValue('./hint-only.js', 'value');\");");
Function("return Factory(\"yield import('./dynamic-only.js');\");");
Function("return Factory(\"yield eval('privateName');\");");
Function("return Factory(\"yield ({}).#privateName;\");");
"#,
        ParseOptions::script(),
    )
    .unwrap();
    let session = LoweringSession::default();
    let program = session.lower_source(&parsed, HostSurfacePolicy::default());
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    assert!(session.realm_module_requests().is_empty());
    assert!(session.prepared_script_import_sites().is_empty());
    let functions = program.script.unwrap().prepared_dynamic_functions;
    assert!(!functions
        .iter()
        .any(|entry| entry.kind == DynamicFunctionKind::Generator
            && entry.arguments.iter().any(|body| body.contains("hint-only")
                || body.contains("dynamic-only")
                || body.contains("eval("))));
    // Independent Function grammar rejects unknown private names; it never
    // borrows the caller's private environment to compile the expression.
    assert!(functions
        .iter()
        .any(|entry| entry.kind == DynamicFunctionKind::Generator
            && entry.arguments == ["yield ({}).#privateName;"]
            && matches!(
                entry.outcome,
                PreparedDynamicFunctionOutcome::SyntaxError { .. }
            )));
}

#[test]
fn mixed_prototype_readers_keep_existing_primary_boxed_source_candidates() {
    let parsed = parse(
        r#"
var builtinReader = Object.getPrototypeOf;
function userReader(value) { return { constructor: Function }; }
function source() {}
var choice = globalThis.unknownChoice;
var String = choice ? builtinReader : userReader;
var value = choice ? source : 'return 73;';
var body = String(value);
Function(body);
"#,
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
            .any(|entry| entry.kind == DynamicFunctionKind::Ordinary
                && entry.arguments == ["return 73;"]
                && matches!(
                    entry.outcome,
                    PreparedDynamicFunctionOutcome::Compiled { .. }
                )),
        "an additive prototype hint replaced an existing primary conversion view"
    );
}

#[test]
fn resolved_constructor_values_flow_through_literal_arrays_and_finite_iterators() {
    let definitions = r#"
var Ordinary = Function;
var Generator = (function* () {}).constructor;
var Async = (async function () {}).constructor;
var AsyncGenerator = (async function* () {}).constructor;
var constructors = [Ordinary, Generator, Async, AsyncGenerator];
var iterable = { [Symbol.iterator]() {
    var index = 0;
    return { next() { return { done: index >= 4, value: constructors[index++] }; } };
} };
"#;
    for prefix in ["", "'use strict';"] {
        for iterable in ["constructors", "iterable"] {
            let source = parse(
                format!("{prefix}{definitions} for (const constructor of {iterable}) {{ constructor('value', 'return value;'); constructor('#!\\n_'); }}"),
                ParseOptions::script(),
            ).unwrap();
            let program = lower(&source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.unwrap();
            assert!(script
                .body
                .statements
                .iter()
                .any(|statement| matches!(statement, lila_ir::StatementIr::ForOfIterator { .. })));
            for kind in DynamicFunctionKind::ALL {
                assert!(
                    script
                        .prepared_dynamic_functions
                        .iter()
                        .any(|entry| entry.kind == *kind
                            && entry.arguments == ["value", "return value;"]
                            && matches!(
                                entry.outcome,
                                PreparedDynamicFunctionOutcome::Compiled { .. }
                            )),
                    "missing {kind:?} through {iterable}"
                );
                assert!(
                    script
                        .prepared_dynamic_functions
                        .iter()
                        .any(|entry| entry.kind == *kind
                            && entry.arguments == ["#!\n_"]
                            && matches!(
                                entry.outcome,
                                PreparedDynamicFunctionOutcome::SyntaxError { .. }
                            )),
                    "missing deferred {kind:?} hashbang error through {iterable}"
                );
            }
        }
    }
}

#[test]
fn resolved_named_function_prototypes_retain_each_constructor_grammar() {
    for (declaration, kind) in [
        ("function source() {}", DynamicFunctionKind::Ordinary),
        ("function* source() {}", DynamicFunctionKind::Generator),
        ("async function source() {}", DynamicFunctionKind::Async),
        (
            "async function* source() {}",
            DynamicFunctionKind::AsyncGenerator,
        ),
        (
            "var source = function* () {};",
            DynamicFunctionKind::Generator,
        ),
    ] {
        for reader in ["Object.getPrototypeOf", "Reflect.getPrototypeOf"] {
            for read in [format!("{reader}(source)"), "read(source)".to_string()] {
                let parsed = parse(
                    format!("{declaration} var read = {reader}; var prototype = {read}; var constructor = prototype.constructor; constructor('value', 'return value;');"),
                    ParseOptions::script(),
                ).unwrap();
                let program = lower(&parsed);
                assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
                assert!(
                    program
                        .script
                        .unwrap()
                        .prepared_dynamic_functions
                        .iter()
                        .any(|entry| entry.kind == kind
                            && entry.arguments == ["value", "return value;"]
                            && matches!(
                                entry.outcome,
                                PreparedDynamicFunctionOutcome::Compiled { .. }
                            )),
                    "missing {kind:?} through {read}"
                );
            }
        }
    }
}

#[test]
fn shadowed_constructor_and_prototype_reader_bindings_do_not_seed_unrelated_sources() {
    for source in [
        "function outer(Function) { var constructors = [Function]; for (const constructor of constructors) constructor('return 997;'); } outer(function ordinary(body) { return body; });",
        "function outer(Object) { function* source() {} var prototype = Object.getPrototypeOf(source); var constructor = prototype.constructor; constructor('return 998;'); } outer({ getPrototypeOf(value) { return { constructor(body) { return body; } }; } });",
        "function* source() {} var read = function getPrototypeOf(value) { return { constructor(body) { return body; } }; }; var constructor = read(source).constructor; constructor('return 999;');",
    ] {
        let parsed = parse(source, ParseOptions::script()).unwrap();
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        for entry in program.script.unwrap().prepared_dynamic_functions {
            assert!(!["return 997;", "return 998;", "return 999;"].iter()
                .any(|body| entry.arguments == [*body]),
                "a spelling or unrelated callable seeded {source}");
        }
    }
}
