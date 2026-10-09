use lila_aot_wasm::{
    emit_with_promise_rejection_policy, PromiseRejectionPolicy, MODULE_EVALUATION_STATUS_EXPORT,
};
use lila_front::{parse, ParseOptions};
use lila_ir::lower;
use wasmparser::{ExternalKind, Parser, Payload, Validator, WasmFeatures};

fn status_exports(source: &'static str, module: bool, policy: PromiseRejectionPolicy) -> usize {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let parsed = parse(
                source,
                if module {
                    ParseOptions::module()
                } else {
                    ParseOptions::script()
                },
            )
            .unwrap();
            let program = lower(&parsed);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let artifact = emit_with_promise_rejection_policy(&program, policy).unwrap();
            Validator::new_with_features(
                WasmFeatures::default()
                    | WasmFeatures::THREADS
                    | WasmFeatures::FUNCTION_REFERENCES
                    | WasmFeatures::GC
                    | WasmFeatures::EXCEPTIONS,
            )
            .validate_all(&artifact.bytes)
            .unwrap();
            let mut statuses = 0;
            for payload in Parser::new(0).parse_all(&artifact.bytes) {
                if let Payload::ExportSection(exports) = payload.unwrap() {
                    for export in exports {
                        let export = export.unwrap();
                        if export.name == MODULE_EVALUATION_STATUS_EXPORT {
                            assert_eq!(export.kind, ExternalKind::Global);
                            statuses += 1;
                        }
                    }
                }
            }
            statuses
        })
        .unwrap()
        .join()
        .unwrap()
}

#[test]
fn every_module_entry_exports_completion_status_under_both_policies() {
    for policy in [
        PromiseRejectionPolicy::Ignore,
        PromiseRejectionPolicy::FailRun,
    ] {
        for source in ["42;", "await 0;", "await new Promise(() => {});"] {
            assert_eq!(status_exports(source, true, policy), 1);
        }
    }
}

#[test]
fn ordinary_async_script_has_no_module_completion_export() {
    assert_eq!(
        status_exports(
            "void (async () => { await 0; })();",
            false,
            PromiseRejectionPolicy::Ignore
        ),
        0
    );
}
