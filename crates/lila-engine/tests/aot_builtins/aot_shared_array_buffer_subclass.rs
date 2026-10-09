use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

#[test]
fn shared_array_buffer_subclasses_preserve_prototypes_and_growable_storage() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            r#"
const ExpressionSubclass = class extends SharedArrayBuffer {};
class DeclarationSubclass extends SharedArrayBuffer {}
for (const Subclass of [ExpressionSubclass, DeclarationSubclass]) {
    const empty = new Subclass();
    print(empty instanceof Subclass && empty instanceof SharedArrayBuffer);
    print(empty.byteLength === 0);
    const growable = new Subclass(2, {maxByteLength: 8});
    const view = new Uint8Array(growable);
    view[0] = 73;
    growable.grow(6);
    print(growable instanceof Subclass && growable instanceof SharedArrayBuffer);
    print(growable.byteLength === 6 && growable.maxByteLength === 8);
    print(view.length === 6 && view[0] === 73 && view[5] === 0);
}
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("shared buffer subclasses compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string()); 10]
    );
}
