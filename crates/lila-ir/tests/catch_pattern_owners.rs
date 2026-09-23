use lila_front::{parse, ParseOptions};
use lila_ir::{lower, FunctionExecutionKind, FunctionProtocolIr, ScriptIr, StatementIr};
use std::collections::BTreeSet;

fn lower_catch(source: &str) -> ScriptIr {
    let parsed = parse(source, ParseOptions::script()).expect("catch source parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{:?}\n{source}",
        program.diagnostics
    );
    program.script.expect("script IR")
}

#[test]
fn catch_default_classes_have_constructor_and_member_owners() {
    for source in [
        "try { throw {}; } catch ({ cls = class {}, named = class Inner { static self() { return Inner; } } }) { cls; named.self(); }",
        "try { throw []; } catch ([cls = class {}, named = class Inner { static self() { return Inner; } }]) { cls; named.self(); }",
    ] {
        let script = lower_catch(source);
        let method = script
            .functions
            .iter()
            .find(|function| function.name == "Inner.self")
            .expect("qualified class member owner");
        assert_eq!(
            method.protocol,
            FunctionProtocolIr::ClassMethod(FunctionExecutionKind::Ordinary)
        );
        assert!(method
            .captured_bindings
            .iter()
            .any(|binding| binding.source_name == "Inner"));
        let constructors = script
            .functions
            .iter()
            .filter(|function| function.protocol == FunctionProtocolIr::ClassConstructor)
            .map(|function| function.to_string_representation.materialize())
            .collect::<BTreeSet<_>>();
        assert_eq!(constructors.len(), 2);
    }
}

#[test]
fn catch_patterns_collect_all_callable_default_owners() {
    for (expression, protocol) in [
        (
            "function () { return 1; }",
            FunctionProtocolIr::OrdinaryCallAndConstruct,
        ),
        ("() => 1", FunctionProtocolIr::Arrow),
        ("function* () { yield 1; }", FunctionProtocolIr::Generator),
        ("async function () { return 1; }", FunctionProtocolIr::Async),
        ("async () => 1", FunctionProtocolIr::AsyncArrow),
        (
            "async function* () { yield 1; }",
            FunctionProtocolIr::AsyncGenerator,
        ),
    ] {
        for pattern in [
            format!("{{ read = {expression} }}"),
            format!("[read = {expression}]"),
        ] {
            let source = format!("try {{ throw []; }} catch ({pattern}) {{ read; }}");
            let script = lower_catch(&source);
            assert_eq!(script.functions.len(), 1);
            assert_eq!(script.functions[0].protocol, protocol);
        }
    }
}

#[test]
fn catch_parameter_closures_and_body_closures_capture_distinct_lexical_owners() {
    let script = lower_catch(
        "function owner() { let value = 'outside'; try { throw []; } catch ([parameter = 1, read = function parameterReader() { return [value, parameter]; }]) { let value = 'inside'; function bodyReader() { return value; } return [read, bodyReader]; } }",
    );
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .unwrap();
    let parameter_reader = script
        .functions
        .iter()
        .find(|function| function.name == "parameterReader")
        .unwrap();
    let body_reader = script
        .functions
        .iter()
        .find(|function| function.name == "bodyReader")
        .unwrap();
    let parameter_capture = parameter_reader
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "value")
        .expect("outer value capture");
    let body_capture = body_reader
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "value")
        .expect("body value capture");
    assert_ne!(parameter_capture.name, body_capture.name);
    assert!(owner
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == parameter_capture.name));
    let StatementIr::TryCatch {
        catch_parameter_environment,
        catch_block,
        ..
    } = owner
        .body
        .statements
        .iter()
        .find(|statement| matches!(statement, StatementIr::TryCatch { .. }))
        .unwrap()
    else {
        unreachable!()
    };
    let catch_parameter_capture = parameter_reader
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "parameter")
        .expect("catch parameter capture");
    assert!(catch_parameter_environment
        .as_ref()
        .expect("captured catch parameter needs an environment")
        .bindings
        .iter()
        .any(|binding| binding.name == catch_parameter_capture.name));
    assert_ne!(catch_parameter_capture.name, parameter_capture.name);
    assert_ne!(catch_parameter_capture.name, body_capture.name);
    assert!(catch_block.lexical_environment.is_none());
    let StatementIr::Block(body) = catch_block
        .statements
        .last()
        .expect("catch body follows binding initialization")
    else {
        panic!("body environment must start after parameter initialization")
    };
    assert!(body
        .lexical_environment
        .as_ref()
        .expect("body lexical environment")
        .bindings
        .iter()
        .any(|binding| binding.name == body_capture.name));
}

#[test]
fn computed_keys_and_nested_pattern_defaults_keep_their_execution_owners() {
    let script = lower_catch(
        "try { throw {value: {nested: []}}; } catch ({[(() => 'value')()]: {nested: [read = function* () { yield 7; }]}}) { read; }",
    );
    let sources = script
        .functions
        .iter()
        .map(|function| function.to_string_representation.materialize())
        .collect::<BTreeSet<_>>();
    assert_eq!(sources.len(), 2);
    assert!(script
        .functions
        .iter()
        .any(|function| function.protocol == FunctionProtocolIr::Generator));
    assert!(script
        .functions
        .iter()
        .any(|function| function.protocol == FunctionProtocolIr::Arrow));
}
