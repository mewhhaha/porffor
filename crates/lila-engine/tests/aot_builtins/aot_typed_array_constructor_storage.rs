use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn concrete_constructors_keep_storage_kind_separate_from_new_target_prototype() {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    let source = r#"
        const other = __lilaCreateRealm().global;
        const cases = [
            ['Int8Array', 1, -1, -1], ['Uint8Array', 1, -1, 255],
            ['Uint8ClampedArray', 1, -1, 0], ['Int16Array', 2, -1, -1],
            ['Uint16Array', 2, -1, 65535], ['Int32Array', 4, -1, -1],
            ['Uint32Array', 4, -1, 4294967295],
            ['Float16Array', 2, 1.5, 1.5], ['Float32Array', 4, 1.5, 1.5],
            ['Float64Array', 8, 1.5, 1.5], ['BigInt64Array', 8, -1n, -1n],
            ['BigUint64Array', 8, -1n, 18446744073709551615n]
        ];
        function check(value, stage, name) { if (!value) throw stage + ':' + name; }
        for (const realm of [globalThis, other]) {
            for (const row of cases) {
                const name = row[0], Constructor = realm[name];
                const buffer = new realm.SharedArrayBuffer(row[1] * 2);
                const views = [new Constructor(buffer), new Constructor(2),
                    Reflect.construct(Constructor, [buffer], other.Float64Array)];
                for (let index = 0; index < views.length; index++) {
                    const view = views[index];
                    check(view.length === 2, 'length', name);
                    check(view.byteLength === row[1] * 2, 'width', name);
                    check(Object.prototype.toString.call(view) === '[object ' + name + ']', 'brand', name);
                    check(Object.getPrototypeOf(view) === (index === 2
                        ? other.Float64Array.prototype : Constructor.prototype), 'prototype', name);
                    view[0] = row[2];
                    check(view[0] === row[3], 'conversion', name);
                    if (index !== 1) check(view.buffer === buffer, 'shared backing', name);
                }
            }
        }
        print('typed-array-storage:ok');
    "#;
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("constructor storage control executes through Wasm AOT");
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{observed:?}"
    );
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine("typed-array-storage:ok".into())]
    );
}
