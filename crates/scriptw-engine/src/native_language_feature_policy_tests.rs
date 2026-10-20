//! Real native callbacks must retain the caller's selected language semantics.
use super::*;

fn converted_text(text: &str) -> String {
    format!(
        "String.fromCharCode({})",
        text.encode_utf16()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",")
    )
}

#[test]
fn actual_native_function_global_script_and_direct_eval_inherit_language_policy() {
    configure_compilation_jobs(1).unwrap();
    let eval_text = "(() => { class Base { constructor(value) { return value; } } class Stamp extends Base { #field; static has(value) { return #field in value; } } const receiver = Object.preventExtensions({}); let caught; try { new Stamp(receiver); } catch (error) { caught = error; } return globalThis.proposalEnabled ? caught instanceof TypeError && !Stamp.has(receiver) : caught === undefined && Stamp.has(receiver); })()";
    let function_text = format!("return {eval_text};");
    let eval_units = eval_text.encode_utf16().collect::<Vec<_>>();
    for enabled in [false, true] {
        let function_observer =
            Arc::new(wasm_native_function_compiler::NativeFunctionCompilationObserver::default());
        let global_observer = Arc::new(
            wasm_native_global_script_compiler::NativeGlobalScriptCompilationObserver::default(),
        );
        let direct_observer = Arc::new(
            wasm_native_direct_eval_compiler::NativeDirectEvalCompilationObserver::default(),
        );
        let mut engine = Engine::new(RealmBuilder::new().build());
        engine.native_function_compilation_observer = Some(Arc::clone(&function_observer));
        engine.native_global_script_compilation_observer = Some(Arc::clone(&global_observer));
        engine.native_direct_eval_compilation_observer = Some(Arc::clone(&direct_observer));
        let source = format!("globalThis.proposalEnabled = {enabled}; const text = {}; const direct = eval(text); const indirect = (0, eval)(text); const created = Function({})(); direct && indirect && created;", converted_text(eval_text), converted_text(&function_text));
        let options = CompileOptions {
            language_features: LanguageFeaturePolicy::default().with_private_element_extensibility(
                if enabled {
                    PrivateElementExtensibility::NonExtensibleAppliesToPrivate
                } else {
                    PrivateElementExtensibility::Ecma262
                },
            ),
            ..CompileOptions::default()
        };
        let unit = engine.compile_script(&source, options).unwrap();
        let script = unit.ir.script.as_ref().unwrap();
        assert!(!script.prepared_dynamic_functions.iter().any(|row| row
            .arguments
            .iter()
            .map(String::as_str)
            .eq([function_text.as_str()])));
        assert!(!script
            .prepared_scripts
            .iter()
            .any(|row| scriptw_front::decode_js_string_utf16(&row.source) == eval_units));
        let outcome = engine
            .run_compiled_unit(
                &unit,
                &source,
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .unwrap();
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(
            outcome.note.contains("boolean(true)"),
            "{enabled}: {}",
            outcome.note
        );
        let function = function_observer.snapshot();
        let global = global_observer.snapshot();
        let direct = direct_observer.snapshot();
        assert_eq!(
            (
                function.requests,
                function.completed_factories,
                function.syntax_rejections
            ),
            (1, 1, 0)
        );
        assert_eq!(
            (
                global.requests,
                global.completed_factories,
                global.syntax_rejections
            ),
            (1, 1, 0)
        );
        assert_eq!(
            (
                direct.requests,
                direct.completed_factories,
                direct.syntax_rejections
            ),
            (1, 1, 0)
        );
    }
}
