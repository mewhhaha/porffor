use super::*;

#[test]
fn native_nested_eval_and_old_program_reentry_keep_separate_defining_owners() {
    let first = "(function(callback){gc();if(callback)callback();return ++value;});";
    let second = "(function(){return value+6;});";
    let nested = "40+2;";
    let source = format!(
        r#"
function create(text){{let value=17;return eval(text);}}
var first=create({first}),second=create({second}),nested={nested};
gc();print(first(function(){{print(second());print(eval(nested));}}));
gc();print(first());
"#,
        first = converted_text(first),
        second = converted_text(second),
        nested = converted_text(nested)
    );
    assert_trace(
        &source,
        &["23", "42", "18", "19"],
        &[units(first), units(second), units(nested)],
        (3, 3, 0),
    );
}

#[test]
fn direct_eval_native_compilation_and_execution_keep_the_original_parent_timeout() {
    configure_compilation_jobs(1).unwrap();
    let observer = Arc::new(NativeDirectEvalCompilationObserver::default());
    let mut engine = Engine::new(RealmBuilder::new().build());
    engine.native_direct_eval_compilation_observer = Some(Arc::clone(&observer));
    let source = format!("eval({});", converted_text("while(true){}"));
    let unit = engine
        .compile_script(&source, CompileOptions::default())
        .unwrap();
    let artifact = engine.emit_wasm(&unit).unwrap();
    let error = run_on_sized_stack(|| {
        engine.execute_with_wasm_bytes_inner(
            artifact.program(),
            Some(500),
            false,
            WasmModuleMemoryCachePolicy::Retain,
            &WasmExecutionMode::Structured,
        )
    })
    .err()
    .unwrap();
    assert_eq!(
        error.wasm_execution_failure_kind(),
        Some(WasmExecutionFailureKind::Timeout)
    );
    assert_eq!(observer.snapshot().requests, 1);
    assert_eq!(observer.snapshot().syntax_rejections, 0);
}
