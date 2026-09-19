use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower_with_host_surface_policy, GlobalDeclarationSetIr, GlobalPropertyInitializerIr,
    HostBuiltinId, HostSurfacePolicy, ScriptIr,
};

fn lower(source: &str) -> ScriptIr {
    let parsed = parse(source, ParseOptions::script()).expect("entry Script parses");
    let program = lower_with_host_surface_policy(&parsed, HostSurfacePolicy::Test262);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.expect("entry Script IR")
}

#[test]
fn prepared_scripts_and_functions_require_entry_host_globals() {
    for source in [
        "(0, eval)(\"function finish(value) { print(value); }\"); finish(7);",
        "var $262 = { evalScript: __lilaRealmEvalScript }; $262.evalScript(\"print(7);\");",
        "Function(\"print(7);\")();",
    ] {
        let script = lower(source);
        let binding = script.global_bindings.get("print").expect("host global");
        assert_eq!(
            binding.initializer,
            GlobalPropertyInitializerIr::HostFunction(HostBuiltinId::Print)
        );
        assert_eq!(binding.declarations, GlobalDeclarationSetIr::None);
        assert!(script.host_builtins.contains(&HostBuiltinId::Print));
    }
}

#[test]
fn entry_var_declarations_reuse_prepared_host_requirements() {
    let script = lower("var print; (0, eval)(\"print(7);\");");
    let binding = script.global_bindings.get("print").expect("host global");
    assert_eq!(
        binding.initializer,
        GlobalPropertyInitializerIr::HostFunction(HostBuiltinId::Print)
    );
    assert_eq!(binding.declarations, GlobalDeclarationSetIr::Var);
}

#[test]
fn entry_function_and_lexical_declarations_keep_their_binding_modes() {
    let script = lower("function print(value) { return value; } (0, eval)(\"print(7);\");");
    let binding = script
        .global_bindings
        .get("print")
        .expect("source function");
    assert!(matches!(
        binding.initializer,
        GlobalPropertyInitializerIr::SourceFunction(_)
    ));
    assert_eq!(binding.declarations, GlobalDeclarationSetIr::Function);

    let script = lower("let print = 7; (0, eval)(\"print(7);\");");
    assert!(script
        .global_bindings
        .lexical_names()
        .any(|name| name == "print"));
    assert_eq!(
        script.global_bindings.get("print").unwrap().initializer,
        GlobalPropertyInitializerIr::HostFunction(HostBuiltinId::Print)
    );
}

#[test]
fn prepared_declarations_are_not_published_with_host_requirements() {
    let script =
        lower("(0, eval)(\"var later; function finish() { print(7); } let privateName;\");");
    assert!(script.global_bindings.get("print").is_some());
    assert!(script.global_bindings.get("later").is_none());
    assert!(script.global_bindings.get("finish").is_none());
    assert!(!script
        .global_bindings
        .lexical_names()
        .any(|name| name == "privateName"));
}
