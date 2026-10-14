use lila_engine::{
    CompileOptions, EmbeddedModuleEntryInput, EmbeddedModuleGoal, EmbeddedModuleGraph,
    EmbeddedModuleInput, EmbeddedModuleReferrer, EmbeddedModuleResolutionInput,
    EmbeddedModuleSourceInput, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ModuleLoadingPolicy, ObservedCompletion, RealmBuilder, RunOptions,
};

fn observe(
    goal: EmbeddedModuleGoal,
    entry: &str,
    rows: &[(&str, &str, &str)],
    edges: &[(&str, &str, Option<&str>)],
    marker: &str,
) {
    let graph = EmbeddedModuleGraph::try_new_typed(
        EmbeddedModuleEntryInput {
            goal,
            identity: "entry.js".into(),
            source: entry.into(),
            meta_url: "lila://entry.js".into(),
        },
        rows.iter()
            .map(|(identity, kind, source)| {
                let row = EmbeddedModuleSourceInput {
                    identity: (*identity).into(),
                    source: (*source).into(),
                    meta_url: format!("lila://{identity}"),
                };
                match *kind {
                    "text" => EmbeddedModuleInput::Text(row),
                    "json" => EmbeddedModuleInput::Json(row),
                    "source" => EmbeddedModuleInput::SourceText(row),
                    _ => panic!("fixture kind"),
                }
            })
            .collect(),
        edges
            .iter()
            .map(|(specifier, target, kind)| EmbeddedModuleResolutionInput {
                referrer: match goal {
                    EmbeddedModuleGoal::Module => EmbeddedModuleReferrer::Module("entry.js".into()),
                    EmbeddedModuleGoal::Script => EmbeddedModuleReferrer::Script("entry.js".into()),
                },
                specifier: (*specifier).into(),
                target: (*target).into(),
                attributes: kind
                    .map(|kind| vec![("type".into(), kind.into())])
                    .unwrap_or_default(),
            })
            .collect(),
    )
    .unwrap();
    lila_engine::configure_compilation_jobs(1).unwrap();
    #[cfg(feature = "spec-exec-oracle")]
    let backends = [ExecutionBackend::WasmAot, ExecutionBackend::SpecExec];
    #[cfg(not(feature = "spec-exec-oracle"))]
    let backends = [ExecutionBackend::WasmAot];
    for backend in backends {
        let engine = Engine::new(RealmBuilder::new().build());
        let options = CompileOptions {
            module_loading_policy: ModuleLoadingPolicy::Embedded(graph.clone()),
            host_surface_policy: HostSurfacePolicy::Test262,
            ..CompileOptions::default()
        };
        let run = RunOptions {
            backend,
            timeout_ms: Some(30_000),
            ..RunOptions::default()
        };
        let observed = match goal {
            EmbeddedModuleGoal::Module => engine.observe_module(entry, options, run),
            EmbeddedModuleGoal::Script => engine.observe_script(entry, options, run),
        }
        .unwrap();
        assert_eq!(observed.backend_used, backend);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{backend:?}: {:?}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(marker.into())]
        );
    }
}

#[test]
fn static_text_defaults_preserve_source_strings_and_distinguish_self_records() {
    const ENTRY: &str = r#"
import own from './entry.js' with {type:'text'};
import value from './shared.js';
import raw from './shared.js' with {type:'text'};
import * as namespace from './shared.js' with {type:'text'};
import invalid from './invalid.txt' with {type:'text'};
import empty from './empty.txt' with {type:'text'};
import unicode from './unicode.txt' with {type:'text'};
function require(value) { if (!value) throw 'text module invariant'; }
require(typeof own === 'string' && own.includes("import own from"));
require(value === 42 && globalThis.sourceCount === 1);
require(raw === "globalThis.sourceCount = 1; export default 42;");
require(namespace.default === raw && Object.keys(namespace).join(',') === 'default');
require(invalid === "invalid syntax! import './absent.js';" && empty === '');
require(unicode === '\uFEFFĀ\0𝄞\n');
print('static-text:ok');
"#;
    observe(
        EmbeddedModuleGoal::Module,
        ENTRY,
        &[
            ("entry.js", "text", ENTRY),
            (
                "shared.js",
                "source",
                "globalThis.sourceCount = 1; export default 42;",
            ),
            (
                "shared.js",
                "text",
                "globalThis.sourceCount = 1; export default 42;",
            ),
            (
                "invalid.txt",
                "text",
                "invalid syntax! import './absent.js';",
            ),
            ("empty.txt", "text", ""),
            ("unicode.txt", "text", "\u{feff}Ā\0𝄞\n"),
        ],
        &[
            ("./entry.js", "entry.js", Some("text")),
            ("./shared.js", "shared.js", None),
            ("./shared.js", "shared.js", Some("text")),
            ("./invalid.txt", "invalid.txt", Some("text")),
            ("./empty.txt", "empty.txt", Some("text")),
            ("./unicode.txt", "unicode.txt", Some("text")),
        ],
        "static-text:ok",
    );
}

#[test]
fn computed_dynamic_text_imports_keep_json_and_alias_namespaces_distinct() {
    const ENTRY: &str = r#"
const request = './data.json';
Promise.all([import(request, {with:{type:'text'}}), import('./alias.json', {with:{type:'text'}}), import(request, {with:{type:'json'}})]).then(values => {
    if (values[0] !== values[1] || values[0] === values[2]) throw 'record identity';
    if (values[0].default !== '{"a":1}\n' || values[2].default.a !== 1) throw 'grammar selected by kind';
    if (Object.keys(values[0]).join(',') !== 'default') throw 'text namespace';
    print('dynamic-text:ok');
});
"#;
    observe(
        EmbeddedModuleGoal::Script,
        ENTRY,
        &[
            ("data.json", "text", "{\"a\":1}\n"),
            ("data.json", "json", "{\"a\":1}\n"),
        ],
        &[
            ("./data.json", "data.json", Some("text")),
            ("./alias.json", "data.json", Some("text")),
            ("./data.json", "data.json", Some("json")),
        ],
        "dynamic-text:ok",
    );
}
