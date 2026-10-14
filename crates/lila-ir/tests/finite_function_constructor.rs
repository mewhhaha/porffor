use lila_front::{parse, ParseOptions};
use lila_ir::{lower, DynamicFunctionKind, PreparedDynamicFunctionOutcome};

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
