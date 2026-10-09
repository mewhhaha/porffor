use lila_engine::{
    CompileOptions, EmbeddedModuleEntryInput, EmbeddedModuleGoal, EmbeddedModuleGraph,
    EmbeddedModuleInput, EmbeddedModuleReferrer, EmbeddedModuleResolutionInput,
    EmbeddedModuleSourceInput, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ModuleLoadingPolicy, ObservedCompletion, RealmBuilder, RunOptions,
};
use std::sync::Arc;

#[test]
fn native_json_default_is_shared_data_with_original_realm_and_import_rejections() {
    const ENTRY: &str = r#"
import data from './data.json' with { type: 'json' };
import * as namespace from './data.json' with { type: 'json' };
function require(value) { if (!value) throw 'JSON module invariant'; }
const objectPrototype = Object.prototype;
const arrayPrototype = Array.prototype;
require(namespace.default === data && Object.keys(namespace).join(',') === 'default');
require(Object.getPrototypeOf(data) === objectPrototype && Object.getPrototypeOf(data.list) === arrayPrototype);
require(Object.keys(data).join(',') === 'a,__proto__,list,units,zero,large');
const descriptor = Object.getOwnPropertyDescriptor(data, '__proto__');
require(descriptor.value === 2 && descriptor.enumerable && descriptor.writable && descriptor.configurable);
require(data.list.length === 3 && data.list[0] === null && data.list[1] === true && data.list[2] === 3);
require(data.units.length === 1 && data.units.charCodeAt(0) === 0xD800);
require(1 / data.zero === -Infinity && data.large === Infinity);
data.a = 9;
globalThis.JSON = { parse() { throw 'public JSON parser observed'; } };
const results = await Promise.all([import('./data.json', { with: { type: 'json' } }), import('./alias.json', { with: { type: 'json' } })]);
require(results[0] === namespace && results[1] === namespace && results[1].default.a === 9);
const fresh = await import('./fresh.json', { with: { type: 'json' } });
require(fresh.default === 17);
try { await import('./bad.json', { with: { type: 'json' } }); throw 'bad JSON accepted'; }
catch (error) { require(error instanceof SyntaxError); }
try { await import.source('./data.json', { with: { type: 'json' } }); throw 'JSON source representation fabricated'; }
catch (error) { require(error instanceof SyntaxError); }
try { await import('./data.json'); throw 'JSON type omitted'; }
catch (error) { require(error instanceof TypeError); }
print('native-json-modules:ok');
"#;
    let edge = |specifier: &str, target: &str| EmbeddedModuleResolutionInput {
        referrer: EmbeddedModuleReferrer::Module("entry.js".into()),
        specifier: specifier.into(),
        attributes: vec![("type".into(), "json".into())],
        target: target.into(),
    };
    let json = |identity: &str, source: &str| {
        EmbeddedModuleInput::Json(EmbeddedModuleSourceInput {
            identity: identity.into(),
            source: source.into(),
            meta_url: format!("lila://json/{identity}"),
        })
    };
    let graph = EmbeddedModuleGraph::try_new_typed(EmbeddedModuleEntryInput {
        goal: EmbeddedModuleGoal::Module, identity: "entry.js".into(), source: ENTRY.into(),
        meta_url: "lila://json/entry.js".into(),
    }, vec![json("data.json", r#"{"a":1,"__proto__":1,"__proto__":2,"list":[null,true,3],"units":"\uD800","zero":-0,"large":1e400}"#),
        json("fresh.json", "17"), json("bad.json", "[1,]")],
        vec![edge("./data.json", "data.json"), edge("./alias.json", "data.json"), edge("./fresh.json", "fresh.json"), edge("./bad.json", "bad.json")]).unwrap();
    lila_engine::configure_compilation_jobs(1).unwrap();
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_module(
            ENTRY,
            CompileOptions {
                module_loading_policy: ModuleLoadingPolicy::Embedded(Arc::clone(&graph)),
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        observed.completion
    );
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine("native-json-modules:ok".into())]
    );
}
