use std::sync::Arc;

use lila_engine::{
    CompileOptions, EmbeddedModuleEntryInput, EmbeddedModuleGoal, EmbeddedModuleGraph,
    EmbeddedModuleReferrer, EmbeddedModuleResolutionInput, EmbeddedModuleSourceInput, Engine,
    ExecutionBackend, HostOutputEvent, HostSurfacePolicy, ModuleLoadingPolicy, ObservedCompletion,
    ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn graph(
    goal: EmbeddedModuleGoal,
    source: &str,
    modules: &[(&str, &str, &str)],
    resolutions: Vec<EmbeddedModuleResolutionInput>,
) -> Arc<EmbeddedModuleGraph> {
    EmbeddedModuleGraph::try_new(
        EmbeddedModuleEntryInput {
            goal,
            identity: "entry.js".into(),
            source: source.into(),
            meta_url: "lila://observed/entry.js".into(),
        },
        modules
            .iter()
            .map(|(identity, source, url)| EmbeddedModuleSourceInput {
                identity: (*identity).into(),
                source: (*source).into(),
                meta_url: (*url).into(),
            })
            .collect(),
        resolutions,
    )
    .expect("finite graph is dependency sealed")
}

fn edge(
    specifier: &str,
    attributes: &[(&str, &str)],
    target: &str,
) -> EmbeddedModuleResolutionInput {
    EmbeddedModuleResolutionInput {
        referrer: EmbeddedModuleReferrer::Script("entry.js".into()),
        specifier: specifier.into(),
        attributes: attributes
            .iter()
            .map(|(key, value)| ((*key).into(), (*value).into()))
            .collect(),
        target: target.into(),
    }
}

fn assert_script(graph: Arc<EmbeddedModuleGraph>, output: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    let result = Engine::new(RealmBuilder::new().build())
        .observe_script(
            graph.entry().source(),
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
        .expect("embedded source executes through the actual Wasm AOT path");
    assert_eq!(result.backend_used, ExecutionBackend::WasmAot);
    assert_eq!(
        result.completion,
        ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
    );
    assert_eq!(
        result.output_events,
        vec![HostOutputEvent::PrintLine(output.into())]
    );
}

#[test]
fn computed_exact_attributes_keep_canonical_identity_and_lazy_catalog_bodies() {
    for directive in ["", "'use strict';\n"] {
        let source = format!(
            r#"{directive}
globalThis.loads = 0;
function require(value) {{ if (!value) throw 'embedded value invariant'; }}
var name = 'chosen';
var options = {{ with: {{ '\uE000': 'bmp', '\u{{10000}}': 'astral', flavour: 'blue' }} }};
Promise.all([import(name, options), import(name, options), import('self')]).then(function (values) {{
  require(values[0] === values[1] && values[0].value === 42 && loads === 1);
  require(values[0].url === 'lila://independent/dep.js');
  require(values[2].value === 7 && values[2].moduleThis === 'undefined');
  require(this === undefined || this === globalThis);
  return import(name, {{ with: {{ flavour: 'blue' }} }}).then(function () {{ throw 'dropped attrs matched'; }}, function (error) {{
    require(error instanceof TypeError && loads === 1); print('embedded-values:ok');
  }});
}});
262;
"#
        );
        assert_script(graph(EmbeddedModuleGoal::Script, &source, &[
            ("dep.js", "globalThis.loads += 1; export const value = 42; export const url = import.meta.url;", "lila://independent/dep.js"),
            ("red.js", "throw 'unselected attribute body'; export const value = 13;", "lila://red.js"),
            ("entry.js", "export const value = 7; export const moduleThis = typeof this;", "lila://module/entry.js"),
            ("unused.js", "throw 'unreferenced body'; export {};", "lila://unused.js"),
            ("unused-invalid.js", "export const =", "lila://unused-invalid.js"),
        ], vec![
            edge("chosen", &[("flavour", "blue"), ("\u{10000}", "astral"), ("\u{e000}", "bmp")], "dep.js"),
            edge("chosen", &[("flavour", "red")], "red.js"),
            edge("self", &[], "entry.js"),
        ]), "embedded-values:ok");
    }
}

#[test]
fn computed_rejections_and_source_defer_phases_keep_original_failure_cells() {
    for directive in ["", "'use strict';\n"] {
        let source = format!(
            r#"{directive}
globalThis.phaseLoads = 0;
function require(value) {{ if (!value) throw 'embedded phase invariant'; }}
var bad = 'bad'; var cold = 'cold'; var missing = 'absent';
var failures = 0;
Promise.all([
  import(bad).then(function () {{ throw 'malformed fulfilled'; }}, function (error) {{ require(error instanceof SyntaxError); failures++; }}),
  import.source(bad).then(function () {{ throw 'malformed source fulfilled'; }}, function (error) {{ require(error instanceof SyntaxError); failures++; }}),
  import.source(cold).then(function () {{ throw 'source fulfilled'; }}, function (error) {{ require(error instanceof SyntaxError); failures++; }}),
  import(missing).then(function () {{ throw 'missing fulfilled'; }}, function (error) {{ require(error instanceof TypeError); failures++; }})
]).then(function () {{
  require(failures === 4 && phaseLoads === 0);
  return import.defer(cold).then(function (namespace) {{
    require(phaseLoads === 0 && Object.keys(namespace).join() === 'value');
    require(namespace.value === 55 && phaseLoads === 1);
    return import(cold).then(function (eager) {{ require(eager.value === 55 && phaseLoads === 1); print('embedded-phases:ok'); }});
  }});
}});
262;
"#
        );
        assert_script(
            graph(
                EmbeddedModuleGoal::Script,
                &source,
                &[
                    ("bad.js", "export const =", "lila://bad.js"),
                    (
                        "cold.js",
                        "globalThis.phaseLoads += 1; export const value = 55;",
                        "lila://cold.js",
                    ),
                ],
                vec![edge("bad", &[], "bad.js"), edge("cold", &[], "cold.js")],
            ),
            "embedded-phases:ok",
        );
    }
}

#[test]
fn non_scalar_dynamic_strings_never_alias_declared_backslash_requests_or_attributes() {
    for directive in ["", "'use strict';\n"] {
        let source = format!(
            r#"{directive}
function require(value) {{ if (!value) throw 'embedded exact string invariant'; }}
var rejected = 0;
Promise.all([
  import('\uD800').then(function () {{ throw 'lone specifier matched'; }}, function (error) {{ require(error instanceof TypeError); rejected++; }}),
  import('attrs', {{ with: {{ '\uD800': 'yes' }} }}).then(function () {{ throw 'lone key matched'; }}, function (error) {{ require(error instanceof TypeError); rejected++; }}),
  import('value', {{ with: {{ key: '\uD800' }} }}).then(function () {{ throw 'lone value matched'; }}, function (error) {{ require(error instanceof TypeError); rejected++; }}),
  import('\\uD800'), import('attrs', {{ with: {{ '\\uD800': 'yes' }} }}),
  import('value', {{ with: {{ key: '\\uD800' }} }}),
  import('overwritten', {{ with: {{ key: '\uD800', key: 'yes' }} }})
]).then(function (values) {{
  require(rejected === 3 && values[3] === values[4] && values[4] === values[5] && values[6] === values[3]);
  require(values[3].value === 88); print('embedded-strings:ok');
}});
262;
"#
        );
        assert_script(
            graph(
                EmbeddedModuleGoal::Script,
                &source,
                &[("exact.js", "export const value = 88;", "lila://exact.js")],
                vec![
                    edge("\\uD800", &[], "exact.js"),
                    edge("attrs", &[("\\uD800", "yes")], "exact.js"),
                    edge("value", &[("key", "\\uD800")], "exact.js"),
                    edge("overwritten", &[("key", "yes")], "exact.js"),
                ],
            ),
            "embedded-strings:ok",
        );
    }
}

#[test]
fn module_entry_cycle_reuses_the_declared_entry_and_observes_independent_urls() {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    let source = "import { read } from 'dep'; export const value = 31; if (read() !== 31 || import.meta.url !== 'lila://observed/entry.js') throw 'entry cycle'; print('embedded-cycle:ok');";
    let graph = graph(
        EmbeddedModuleGoal::Module,
        source,
        &[(
            "dep.js",
            "import { value } from 'back'; export function read() { return value; }",
            "lila://different/dep.js",
        )],
        vec![
            EmbeddedModuleResolutionInput {
                referrer: EmbeddedModuleReferrer::Module("entry.js".into()),
                specifier: "dep".into(),
                attributes: vec![],
                target: "dep.js".into(),
            },
            EmbeddedModuleResolutionInput {
                referrer: EmbeddedModuleReferrer::Module("dep.js".into()),
                specifier: "back".into(),
                attributes: vec![],
                target: "entry.js".into(),
            },
        ],
    );
    let result = Engine::new(RealmBuilder::new().build())
        .observe_module(
            graph.entry().source(),
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
        .expect("canonical Module entry cycle executes");
    assert_eq!(result.backend_used, ExecutionBackend::WasmAot);
    assert_eq!(
        result.completion,
        ObservedCompletion::Normal(ObservedJsValue::Undefined)
    );
    assert_eq!(
        result.output_events,
        vec![HostOutputEvent::PrintLine("embedded-cycle:ok".into())]
    );
}
