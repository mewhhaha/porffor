use lila_front::{parse, ParseOptions};
use lila_ir::{lower, DynamicSourceGap, DynamicSourceKind, UnsupportedFeature};

#[test]
fn empty_function_construction_is_admitted_without_admitting_source_arguments() {
    for constructor in [
        "Function",
        "(function* () {}).constructor",
        "(async function () {}).constructor",
        "(async function* () {}).constructor",
    ] {
        for invocation in ["C();", "new C();", "C.call(null);"] {
            let source = format!("var C = {constructor}; {invocation}");
            let parsed = parse(&source, ParseOptions::script()).expect("source parses");
            let program = lower(&parsed);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
        }
        for invocation in ["C('');", "C(undefined);", "new C(...[]);"] {
            let source = format!("var C = {constructor}; {invocation}");
            let parsed = parse(&source, ParseOptions::script()).expect("source parses");
            let program = lower(&parsed);
            assert!(
                program.diagnostics.iter().any(|diagnostic| matches!(
                    diagnostic.unsupported_feature(),
                    Some(UnsupportedFeature::DynamicSource(_))
                )),
                "{source}: {:?}",
                program.diagnostics
            );
        }
    }
}

#[test]
fn possibly_deleted_global_eval_retains_indirect_source_capability() {
    for source in [
        "globalThis.unknownHook(); (0, eval)('source');",
        "globalThis.unknownHook(); var retained = eval; retained('source');",
        "delete eval; globalThis.unknownHook(); (0, eval)('source');",
        "delete eval; globalThis.unknownHook(); var retained = eval; retained('source');",
        "var retained = eval; delete eval; retained('source');",
        "globalThis.unknownHook(); delete globalThis.eval; (0, eval)('source');",
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("source parses");
        let program = lower(&parsed);
        assert!(
            program.diagnostics.iter().any(|diagnostic| {
                diagnostic.unsupported_feature()
                    == Some(UnsupportedFeature::DynamicSource(
                        DynamicSourceGap::aot_known_source(DynamicSourceKind::IndirectEval),
                    ))
            }),
            "{source}: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn definitely_deleted_eval_does_not_retain_the_removed_callable() {
    for deletion in ["delete eval;", "delete globalThis.eval;"] {
        for invocation in [
            "eval('source');",
            "(0, eval)('source');",
            "var retained = eval; retained('source');",
        ] {
            let source = format!("{deletion} {invocation}");
            let parsed = parse(&source, ParseOptions::script()).expect("source parses");
            let program = lower(&parsed);
            assert!(
                program.diagnostics.iter().all(|diagnostic| {
                    !matches!(
                        diagnostic.unsupported_feature(),
                        Some(UnsupportedFeature::DynamicSource(_))
                    )
                }),
                "{source}: {:?}",
                program.diagnostics
            );
        }
    }
}

#[test]
fn erased_eval_source_boundaries_preserve_no_source_and_replacement_calls() {
    for source in [
        "globalThis.unknownHook(); (0, eval)(42);",
        "globalThis.unknownHook(); (0, eval)();",
        "globalThis.unknownHook(); eval = function (value) { return value; }; (0, eval)('source');",
        "delete eval; (0, eval)('source');",
        "function example(eval) { return eval('source'); } example(function (value) { return value; });",
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("source parses");
        let program = lower(&parsed);
        assert!(
            program.diagnostics.iter().all(|diagnostic| {
                !matches!(
                    diagnostic.unsupported_feature(),
                    Some(UnsupportedFeature::DynamicSource(_))
                )
            }),
            "{source}: {:?}",
            program.diagnostics
        );
    }
}
