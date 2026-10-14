use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "lila-bytes-catalog-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn catalog_reads_binary_fixtures_and_retains_mixed_records_at_one_url() {
    let directory = Directory::new();
    let source = "import value from './value_FIXTURE.js'; import raw from './value_FIXTURE.js' with {type:'bytes'}; const path = './raw_FIXTURE.png'; import(path, {with:{type:'bytes'}});";
    let entry_path = directory.0.join("entry.js");
    fs::write(&entry_path, source).unwrap();
    fs::write(directory.0.join("value_FIXTURE.js"), b"export default 42;").unwrap();
    fs::write(directory.0.join("raw_FIXTURE.png"), [0, 255, 128, 10]).unwrap();
    fs::write(directory.0.join("empty_FIXTURE.txt"), []).unwrap();
    let graph = declare(&entry_path, EmbeddedModuleGoal::Module, source, source)
        .unwrap()
        .unwrap()
        .graph;
    let referrer = EmbeddedModuleReferrer::Module(graph.entry().identity().into());
    let request = |specifier: &str, bytes: bool| {
        lila_engine::EmbeddedModuleRequest::try_new(
            specifier,
            if bytes {
                vec![("type".into(), "bytes".into())]
            } else {
                Vec::new()
            },
        )
        .unwrap()
    };
    let text = graph
        .resolve(&referrer, &request("./value_FIXTURE.js", false))
        .unwrap();
    let raw = graph
        .resolve(&referrer, &request("./value_FIXTURE.js", true))
        .unwrap();
    assert_eq!(text.kind(), EmbeddedModuleKind::SourceText);
    assert_eq!(raw.kind(), EmbeddedModuleKind::Bytes);
    assert_eq!(text.identity(), raw.identity());
    assert_eq!(text.meta_url(), raw.meta_url());
    assert_eq!(raw.payload_bytes(), b"export default 42;");
    let image = graph
        .resolve(&referrer, &request("./raw_FIXTURE.png", true))
        .unwrap();
    assert_eq!(image.payload_bytes(), &[0, 255, 128, 10]);
    assert!(graph
        .resolve(&referrer, &request("./raw_FIXTURE.png", false))
        .is_none());
    assert_eq!(
        graph
            .resolve(&referrer, &request("./empty_FIXTURE.txt", true))
            .unwrap()
            .payload_bytes(),
        b"" as &[u8]
    );
}

#[test]
fn text_catalog_keeps_self_import_raw_and_admits_computed_json_text() {
    let directory = Directory::new();
    let source = "import self from './entry.js' with {type:'text'}; const path = './raw_FIXTURE.json'; import(path, {with:{type:'text'}});";
    let entry = directory.0.join("entry.js");
    fs::write(&entry, source).unwrap();
    let raw = "\u{feff}{not JSON}\0𝄞";
    fs::write(directory.0.join("raw_FIXTURE.json"), raw).unwrap();
    fs::write(directory.0.join("invalid_FIXTURE.txt"), [255]).unwrap();
    let graph = declare(&entry, EmbeddedModuleGoal::Module, source, source)
        .unwrap()
        .unwrap()
        .graph;
    let referrer = EmbeddedModuleReferrer::Module(graph.entry().identity().into());
    let text_request = |specifier: &str| {
        lila_engine::EmbeddedModuleRequest::try_new(specifier, vec![("type".into(), "text".into())])
            .unwrap()
    };
    let own = graph
        .resolve(&referrer, &text_request("./entry.js"))
        .unwrap();
    assert_eq!(own.kind(), EmbeddedModuleKind::Text);
    assert_eq!(own.source_text(), Some(source));
    assert!(graph
        .module(graph.entry().identity(), EmbeddedModuleKind::SourceText)
        .is_some());
    let data = graph
        .resolve(&referrer, &text_request("./raw_FIXTURE.json"))
        .unwrap();
    assert_eq!(data.kind(), EmbeddedModuleKind::Text);
    assert_eq!(data.source_text(), Some(raw));
    assert!(graph
        .resolve(&referrer, &text_request("./invalid_FIXTURE.txt"))
        .is_none());
}

#[test]
fn literal_runtime_attributes_declare_only_that_locator_supported_record_variants() {
    let directory = Directory::new();
    let source =
        "var options = {with:{type:'json'}}; import('./plain.json', options); // import(ignored)";
    let entry = directory.0.join("entry.js");
    fs::write(&entry, source).unwrap();
    fs::write(directory.0.join("plain.json"), "262").unwrap();
    fs::write(directory.0.join("unrelated_FIXTURE.json"), "123").unwrap();
    for goal in [EmbeddedModuleGoal::Script, EmbeddedModuleGoal::Module] {
        let graph = declare(&entry, goal, source, source)
            .unwrap()
            .unwrap()
            .graph;
        let referrer = match goal {
            EmbeddedModuleGoal::Script => {
                EmbeddedModuleReferrer::Script(graph.entry().identity().into())
            }
            EmbeddedModuleGoal::Module => {
                EmbeddedModuleReferrer::Module(graph.entry().identity().into())
            }
        };
        for (name, kind) in [
            ("json", EmbeddedModuleKind::Json),
            ("bytes", EmbeddedModuleKind::Bytes),
            ("text", EmbeddedModuleKind::Text),
        ] {
            let request = lila_engine::EmbeddedModuleRequest::try_new(
                "./plain.json",
                vec![("type".into(), name.into())],
            )
            .unwrap();
            let target = graph.resolve(&referrer, &request).unwrap();
            assert_eq!(target.kind(), kind);
            assert_eq!(target.payload_bytes(), b"262");
        }
        assert_eq!(
            graph.dependency_modules().count(),
            3,
            "literal requests do not expand unrelated fixtures"
        );
        for attributes in [
            Vec::new(),
            vec![("type".into(), "css".into())],
            vec![
                ("type".into(), "json".into()),
                ("extra".into(), "no".into()),
            ],
        ] {
            let request =
                lila_engine::EmbeddedModuleRequest::try_new("./plain.json", attributes).unwrap();
            assert!(graph.resolve(&referrer, &request).is_none());
        }
    }
    let unsupported = "import('./plain.json', {with:{type:'css'}}); // import(ignored)";
    fs::write(&entry, unsupported).unwrap();
    let graph = declare(&entry, EmbeddedModuleGoal::Script, unsupported, unsupported)
        .unwrap()
        .unwrap()
        .graph;
    assert_eq!(
        graph.dependency_modules().count(),
        0,
        "known unsupported attributes do not become runtime or computed requests"
    );
    let computed = "const path = './plain.json'; import(path, {with:{type:'css'}});";
    fs::write(&entry, computed).unwrap();
    let graph = declare(&entry, EmbeddedModuleGoal::Script, computed, computed)
        .unwrap()
        .unwrap()
        .graph;
    assert_eq!(
        graph.dependency_modules().count(),
        0,
        "known computed attributes cannot select supported-kind fixture rows"
    );
}

#[test]
fn literal_runtime_attribute_catalog_executes_live_options_and_proxy_enumeration() {
    use lila_engine::{
        CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
        ModuleLoadingPolicy, ObservedCompletion, RealmBuilder, RunOptions,
    };
    const BODY: &str = r#"
function require(value, label) { if (!value) throw 'catalog attribute invariant: ' + label; }
var trace = [], symbol = Symbol('ignored');
var proxy = new Proxy({}, {
  ownKeys() { trace.push('keys'); return ['type', symbol, 'hidden']; },
  getOwnPropertyDescriptor(target, key) { trace.push('desc:' + key); return {configurable:true, enumerable:key === 'type'}; },
  get(target, key) { trace.push('get:' + key); if (key !== 'type') throw 'non-enumerable or Symbol Get'; return 'json'; }
});
var options = {get with() { trace.push('with'); return proxy; }};
var checks = [import('./2nd-param_FIXTURE.json', options).then(ns => require(ns.default === 262, 'Proxy JSON'))];
require(trace.join(',') === 'with,keys,desc:type,get:type,desc:hidden', 'live enumeration order');
var plain = {with:{type:'json'}};
checks.push(import('./2nd-param_FIXTURE.json', plain).then(ns => require(ns.default === 262, 'options identifier')));
var text = {with:{type:'text'}}, bytes = {with:{type:'bytes'}};
checks.push(import('./2nd-param_FIXTURE.json', text).then(ns => require(ns.default === '262', 'text kind')));
checks.push(import('./2nd-param_FIXTURE.json', bytes).then(ns => require(ns.default.length === 3 && ns.default[0] === 50 && ns.default[2] === 50, 'bytes kind')));
function rejectsType(promise) { return promise.then(() => { throw 'bad attributes fulfilled'; }, error => require(error instanceof TypeError, 'attribute rejection type')); }
var hidden = {with:new Proxy({}, {
  ownKeys() { return ['type']; },
  getOwnPropertyDescriptor() { return {configurable:true, enumerable:false}; },
  get() { throw 'non-enumerable value observed'; }
})};
checks.push(rejectsType(import('./2nd-param_FIXTURE.json', hidden)));
var extra = {with:{type:'json',extra:'no'}}, wrong = {with:{type:1}};
checks.push(rejectsType(import('./2nd-param_FIXTURE.json', extra)));
checks.push(rejectsType(import('./2nd-param_FIXTURE.json', wrong)));
var marker = {}, abrupt = {get with() { throw marker; }};
checks.push(import('./2nd-param_FIXTURE.json', abrupt).then(() => { throw 'abrupt options fulfilled'; }, error => require(error === marker, 'abrupt identity')));
Promise.all(checks).then(() => print('literal-runtime-attributes:ok'));
"#;
    let directory = Directory::new();
    let entry = directory.0.join("entry.js");
    fs::write(directory.0.join("2nd-param_FIXTURE.json"), "262").unwrap();
    lila_engine::configure_compilation_jobs(1).unwrap();
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{BODY}");
        fs::write(&entry, &source).unwrap();
        let graph = declare(&entry, EmbeddedModuleGoal::Script, &source, &source)
            .unwrap()
            .unwrap()
            .graph;
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    module_loading_policy: ModuleLoadingPolicy::Embedded(graph),
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
            vec![HostOutputEvent::PrintLine(
                "literal-runtime-attributes:ok".into()
            )]
        );
    }
}
