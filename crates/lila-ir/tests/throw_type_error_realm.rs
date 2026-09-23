use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, lower_with_host_surface_policy, ExprIr, FunctionFlavor, FunctionProtocolIr,
    HostBuiltinId, HostSurfacePolicy, ScriptIr, StatementIr, ValueKind,
};

fn lower_script(source: &str) -> ScriptIr {
    let parsed = parse(source, ParseOptions::script()).expect("Arguments ownership source parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.expect("script IR")
}

#[test]
fn strict_and_nonsimple_factories_retain_arguments_and_parameter_protocol_facts() {
    let script = lower_script(
        "function strict(value) {'use strict'; return arguments;}\n\
         function defaults(value = 1) {return arguments;}\n\
         function rest(...values) {return arguments;}\n\
         function mapped(value) {return arguments;}",
    );
    for name in ["strict", "defaults", "rest", "mapped"] {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == name)
            .unwrap();
        assert_eq!(
            function.protocol,
            FunctionProtocolIr::OrdinaryCallAndConstruct
        );
        assert_eq!(function.return_kind, ValueKind::Arguments);
        assert_eq!(function.strict, name == "strict");
        assert_eq!(
            function.params[0].default_init.is_some(),
            name == "defaults"
        );
        assert_eq!(function.params[0].is_rest, name == "rest");
    }
}

#[test]
fn lexical_arguments_belong_to_the_ordinary_owner() {
    let script = lower_script("function strict() { 'use strict'; return () => arguments; }");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "strict")
        .unwrap();
    assert_eq!(owner.protocol.flavor(), FunctionFlavor::Ordinary);
    assert!(!owner.captures_lexical_arguments);
    let arrow = script
        .functions
        .iter()
        .find(|function| function.protocol == FunctionProtocolIr::Arrow)
        .unwrap();
    assert!(arrow.captures_lexical_arguments);
    assert_eq!(arrow.return_kind, ValueKind::Arguments);
}

#[test]
fn extracted_foreign_thrower_calls_remain_runtime_owned_and_do_not_request_gc() {
    let parsed = parse(
        "var foreign = __lilaCreateRealm().global;\n\
         var thrower = Object.getOwnPropertyDescriptor(foreign.Function.prototype, 'caller').get;\n\
         thrower();",
        ParseOptions::script(),
    )
    .expect("foreign intrinsic source parses");
    let program = lower_with_host_surface_policy(&parsed, HostSurfacePolicy::Test262);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.expect("script IR");
    assert!(script.host_builtins.contains(&HostBuiltinId::CreateRealm));
    assert!(!script.host_builtins.contains(&HostBuiltinId::Gc));
    let StatementIr::Expression(expression) = script.body.statements.last().expect("invocation")
    else {
        panic!("final invocation expression");
    };
    assert!(matches!(expression.expr, ExprIr::CallIndirect { .. }));
}
