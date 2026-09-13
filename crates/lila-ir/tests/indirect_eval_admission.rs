use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower_with_host_surface_policy, DynamicSourceGap, DynamicSourceKind, HostSurfacePolicy,
    ProgramIr, UnsupportedFeature, ValueKind,
};

fn lower(source: &str) -> ProgramIr {
    let parsed = parse(source, ParseOptions::script()).expect("outer source parses");
    lower_with_host_surface_policy(&parsed, HostSurfacePolicy::Test262)
}

#[test]
fn runtime_indirect_invocations_need_no_fictitious_prepared_source() {
    for call in [
        "(0, eval)(globalThis.value)",
        "eval.call(undefined, globalThis.value)",
        "eval.apply(undefined, [globalThis.value])",
        "Reflect.apply(eval, undefined, [globalThis.value])",
        "(0, eval)(...globalThis.arguments)",
    ] {
        let program = lower(call);
        assert!(
            program.is_wasm_supported(),
            "{call}: {:?}",
            program.diagnostics
        );
        let script = program
            .script
            .expect("runtime invocation remains in Script IR");
        assert!(script.prepared_scripts.is_empty(), "{call}");
        assert_eq!(script.result_kind(), ValueKind::Dynamic, "{call}");
    }
}

#[test]
fn non_string_proof_retains_its_precise_result() {
    for (call, expected) in [
        ("(0, eval)()", ValueKind::Undefined),
        ("(0, eval)(17)", ValueKind::Number),
        ("(0, eval)(true)", ValueKind::Boolean),
        ("(0, eval)({})", ValueKind::Object),
    ] {
        let program = lower(call);
        assert!(
            program.is_wasm_supported(),
            "{call}: {:?}",
            program.diagnostics
        );
        let script = program.script.expect("statically proven non-string call");
        assert_eq!(script.result_kind(), expected, "{call}");
    }
}

#[test]
fn boxed_values_in_mutable_globals_remain_executable() {
    let program = lower(
        "var x = {}; function check(actual, expected) { return actual === expected; } \
         check((0, eval)(x), x); x = new Number(1); check((0, eval)(x), x); \
         x = new Boolean(true); check((0, eval)(x), x); \
         x = new String('1+1'); check((0, eval)(x), x);",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    assert!(program.script.is_some());
}

#[test]
fn realm_script_keeps_its_distinct_source_conversion_requirement() {
    let program = lower("__lilaRealmEvalScript(globalThis.unknownSource);");
    let gaps = program
        .diagnostics
        .iter()
        .filter_map(|diagnostic| match diagnostic.unsupported_feature() {
            Some(UnsupportedFeature::DynamicSource(gap)) => Some(gap),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        gaps,
        vec![DynamicSourceGap::runtime_source(
            DynamicSourceKind::RealmEvalScript
        )]
    );
}
