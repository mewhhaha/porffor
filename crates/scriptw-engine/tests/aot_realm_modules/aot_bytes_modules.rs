use scriptw_engine::{
    CompileOptions, EmbeddedModuleBytesInput, EmbeddedModuleEntryInput, EmbeddedModuleGoal,
    EmbeddedModuleGraph, EmbeddedModuleInput, EmbeddedModuleReferrer,
    EmbeddedModuleResolutionInput, EmbeddedModuleSourceInput, Engine, ExecutionBackend,
    HostOutputEvent, HostSurfacePolicy, ModuleLoadingPolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};

#[test]
fn raw_bytes_defaults_share_immutable_data_and_keep_mixed_url_records_distinct() {
    const SOURCE: &str = "export default 42; export const url = import.meta.url;";
    const ENTRY: &str = r#"
import source, {url} from './shared.js';
import raw from './shared.js' with {type:'bytes'};
import * as namespace from './shared.js' with {type:'bytes'};
import binary from './binary.bin' with {type:'bytes'};
import json from './data.json' with {type:'json'};
import jsonBytes from './data.json' with {type:'bytes'};
function require(value, label) { if (!value) throw 'Bytes module invariant: ' + label; }
const viewPrototype = Uint8Array.prototype;
const bufferPrototype = ArrayBuffer.prototype;
require(source === 42 && url === 'scriptw://data/shared.js', 'source text record and URL');
require(raw === namespace.default && Object.keys(namespace).join(',') === 'default', 'default export and namespace');
require(Object.getPrototypeOf(raw) === viewPrototype && Object.getPrototypeOf(raw.buffer) === bufferPrototype, 'view and buffer prototypes');
require(raw.length === 54 && raw[0] === 101 && raw[53] === 59, 'source text raw bytes');
require(binary.length === 4 && binary[0] === 0 && binary[1] === 255 && binary[2] === 128 && binary[3] === 65, 'arbitrary binary payload');
require(json.a === 1 && jsonBytes.length === 8 && jsonBytes[7] === 10, 'JSON and Bytes records');
require(raw.buffer.immutable === true && raw.buffer.resizable === false, 'immutable fixed buffer flags');
for (const mutate of [() => { raw[0] = 7; }, () => raw.set([7]),
    () => new DataView(raw.buffer).setUint8(0, 7),
    () => raw.buffer.resize(0), () => raw.buffer.transfer(), () => raw.buffer.transferToFixedLength()]) {
    try { mutate(); throw 'immutable Bytes buffer changed'; }
    catch (error) { require(error instanceof TypeError, 'immutable mutation rejects with TypeError'); }
}
require(raw[0] === 101, 'original byte remains');
const repeated = await Promise.all([import('./shared.js', {with:{type:'bytes'}}),
    import('./alias.js', {with:{type:'bytes'}})]);
require(repeated[0] === namespace && repeated[1] === namespace, 'dynamic and aliased namespace identity');
globalThis.Uint8Array = function () { throw 'public view constructor observed'; };
globalThis.ArrayBuffer = function () { throw 'public buffer constructor observed'; };
const empty = await import('./empty.dat', {with:{type:'bytes'}});
require(empty.default.length === 0 && empty.default.buffer.immutable === true, 'empty immutable buffer');
require(Object.getPrototypeOf(empty.default) === viewPrototype, 'intrinsic allocation after public constructor replacement');
print('native-bytes-modules:ok');
"#;
    let bytes = |identity: &str, payload: &[u8]| {
        EmbeddedModuleInput::Bytes(EmbeddedModuleBytesInput {
            identity: identity.into(),
            bytes: payload.to_vec(),
            meta_url: format!("scriptw://data/{identity}"),
        })
    };
    let edge = |specifier: &str, target: &str, kind: Option<&str>| EmbeddedModuleResolutionInput {
        referrer: EmbeddedModuleReferrer::Module("entry.js".into()),
        specifier: specifier.into(),
        attributes: kind
            .map(|kind| vec![("type".into(), kind.into())])
            .unwrap_or_default(),
        target: target.into(),
    };
    let graph = EmbeddedModuleGraph::try_new_typed(
        EmbeddedModuleEntryInput {
            goal: EmbeddedModuleGoal::Module,
            identity: "entry.js".into(),
            source: ENTRY.into(),
            meta_url: "scriptw://entry.js".into(),
        },
        vec![
            EmbeddedModuleInput::SourceText(EmbeddedModuleSourceInput {
                identity: "shared.js".into(),
                source: SOURCE.into(),
                meta_url: "scriptw://data/shared.js".into(),
            }),
            bytes("shared.js", SOURCE.as_bytes()),
            bytes("binary.bin", &[0, 255, 128, 65]),
            bytes("empty.dat", &[]),
            EmbeddedModuleInput::Json(EmbeddedModuleSourceInput {
                identity: "data.json".into(),
                source: "{\"a\":1}\n".into(),
                meta_url: "scriptw://data/data.json".into(),
            }),
            bytes("data.json", b"{\"a\":1}\n"),
        ],
        vec![
            edge("./shared.js", "shared.js", None),
            edge("./shared.js", "shared.js", Some("bytes")),
            edge("./alias.js", "shared.js", Some("bytes")),
            edge("./binary.bin", "binary.bin", Some("bytes")),
            edge("./empty.dat", "empty.dat", Some("bytes")),
            edge("./data.json", "data.json", Some("json")),
            edge("./data.json", "data.json", Some("bytes")),
        ],
    )
    .unwrap();
    scriptw_engine::configure_compilation_jobs(1).unwrap();
    #[cfg(feature = "spec-exec-oracle")]
    let backends = [ExecutionBackend::WasmAot, ExecutionBackend::SpecExec];
    #[cfg(not(feature = "spec-exec-oracle"))]
    let backends = [ExecutionBackend::WasmAot];
    for backend in backends {
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_module(
                ENTRY,
                CompileOptions {
                    module_loading_policy: ModuleLoadingPolicy::Embedded(graph.clone()),
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .unwrap();
        assert_eq!(observed.backend_used, backend);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{backend:?}: {:?}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine("native-bytes-modules:ok".into())]
        );
    }
}
