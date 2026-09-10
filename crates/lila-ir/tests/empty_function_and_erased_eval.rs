use lila_front::{parse, ParseOptions};
use lila_ir::{lower, PreparedScriptKind, PreparedScriptOutcome, UnsupportedFeature};

#[test]
fn function_construction_admits_empty_static_and_runtime_arguments() {
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
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
        }
    }
}

#[test]
fn possibly_deleted_global_eval_registers_its_indirect_source() {
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
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        assert!(
            program
                .script
                .expect("Script IR")
                .prepared_scripts
                .iter()
                .any(|prepared| prepared.kind == PreparedScriptKind::IndirectEval
                    && prepared.source == "source"
                    && matches!(prepared.outcome, PreparedScriptOutcome::Executable(_))),
            "{source}"
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
