use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use lila_runtime::{
    EmbeddedModuleEntryInput, EmbeddedModuleGoal, EmbeddedModuleGraph, EmbeddedModuleReferrer,
    EmbeddedModuleResolutionInput, EmbeddedModuleSourceInput, HostHooks, HostOutputEvent,
    ModuleLoadingPolicy, ObservedCompletion, ObservedJsValue, ObservedNumber,
};
use lila_spec_exec::{observe_module, observe_script_with_module_loading_policy, ModuleHostConfig};

#[derive(Debug, Default)]
struct RecordingHooks(Mutex<Vec<String>>);

impl HostHooks for RecordingHooks {
    fn print_line(&self, text: &str) {
        self.0.lock().unwrap().push(text.to_owned());
    }
}

fn graph(
    goal: EmbeddedModuleGoal,
    source: &str,
    modules: Vec<EmbeddedModuleSourceInput>,
    resolutions: Vec<EmbeddedModuleResolutionInput>,
) -> Arc<EmbeddedModuleGraph> {
    EmbeddedModuleGraph::try_new(
        EmbeddedModuleEntryInput {
            goal,
            identity: "case/entry.js".into(),
            source: source.into(),
            meta_url: "lila://urls/root.js".into(),
        },
        modules,
        resolutions,
    )
    .unwrap()
}

fn module(identity: &str, source: &str, meta_url: &str) -> EmbeddedModuleSourceInput {
    EmbeddedModuleSourceInput {
        identity: identity.into(),
        source: source.into(),
        meta_url: meta_url.into(),
    }
}

fn edge(
    referrer: EmbeddedModuleReferrer,
    specifier: &str,
    attributes: Vec<(String, String)>,
    target: &str,
) -> EmbeddedModuleResolutionInput {
    EmbeddedModuleResolutionInput {
        referrer,
        specifier: specifier.into(),
        attributes,
        target: target.into(),
    }
}

fn lines(expected: &[&str]) -> Vec<HostOutputEvent> {
    expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).into()))
        .collect()
}

fn script(graph: Arc<EmbeddedModuleGraph>) -> lila_spec_exec::ObservedExecutionOutcome {
    observe_script_with_module_loading_policy(
        graph.entry().source(),
        Some(graph.entry().identity()),
        ModuleLoadingPolicy::Embedded(Arc::clone(&graph)),
        &[],
        false,
        Arc::new(RecordingHooks::default()),
    )
    .expect("embedded Script must reach an ECMAScript completion")
}

#[test]
fn module_cycles_and_dynamic_aliases_retain_identity_and_declared_meta_urls() {
    let source = r#"
        import * as first from 'child';
        export function readRoot() { return 'root'; }
        print('root-url:' + import.meta.url);
        const again = await import('child-alias');
        if (first !== again) throw new Error('duplicated module namespace');
        print(again.read() + ':' + globalThis.childRuns);
    "#;
    let child = r#"
        import { readRoot } from 'root';
        globalThis.childRuns = (globalThis.childRuns || 0) + 1;
        export function read() { return readRoot(); }
        print('child-url:' + import.meta.url);
    "#;
    let graph = graph(
        EmbeddedModuleGoal::Module,
        source,
        vec![module(
            "case/child.js",
            child,
            "lila://urls/child-independent.js",
        )],
        vec![
            edge(
                EmbeddedModuleReferrer::Module("case/entry.js".into()),
                "child",
                vec![],
                "case/child.js",
            ),
            edge(
                EmbeddedModuleReferrer::Module("case/entry.js".into()),
                "child-alias",
                vec![],
                "case/child.js",
            ),
            edge(
                EmbeddedModuleReferrer::Module("case/child.js".into()),
                "root",
                vec![],
                "case/entry.js",
            ),
        ],
    );
    let outcome = observe_module(
        source,
        Some("case/entry.js"),
        ModuleHostConfig {
            // Ambient host paths are ignored by the Embedded branch.
            module_root: Some(PathBuf::from("undeclared-host-directory")),
            test_path: Some(PathBuf::from("undeclared-host-entry.js")),
            module_loading_policy: ModuleLoadingPolicy::Embedded(graph),
            ..ModuleHostConfig::default()
        },
        &[],
        false,
        Arc::new(RecordingHooks::default()),
    )
    .unwrap();
    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Undefined)
    );
    assert_eq!(
        outcome.output_events,
        lines(&[
            "child-url:lila://urls/child-independent.js",
            "root-url:lila://urls/root.js",
            "root:1",
        ])
    );
}

#[test]
fn computed_requests_preserve_attributes_strings_and_script_module_roles() {
    let source = r#"
        (async function () {
            const selected = 'shared';
            const value = await import(selected, { with: { mode: 'stable', type: 'javascript' } });
            print('selected:' + value.answer);
            for (const options of [undefined, { with: { mode: 'changed', type: 'javascript' } }]) {
                try { await import(selected, options); throw new Error('undeclared attributes escaped'); }
                catch (error) { if (!(error instanceof TypeError)) throw error; print('attributes-denied'); }
            }
            try { await import('lila://urls/shared.js'); throw new Error('metadata became a base'); }
            catch (error) { if (!(error instanceof TypeError)) throw error; print('url-denied'); }
            try { await import(String.fromCharCode(0xd800)); throw new Error('surrogate was escaped'); }
            catch (error) { if (!(error instanceof TypeError)) throw error; print('surrogate-denied'); }
            const literal = await import('\\ud800');
            print('literal:' + literal.answer);
            const self = await import('self');
            print('self:' + self.role);
        })();
        262;
    "#;
    let root = EmbeddedModuleReferrer::Script("case/entry.js".into());
    let outcome = script(graph(
        EmbeddedModuleGoal::Script,
        source,
        vec![
            module(
                "case/shared.js",
                "export const answer = 42;",
                "lila://urls/shared.js",
            ),
            module(
                "case/entry.js",
                "export const role = 'module';",
                "lila://urls/module-self.js",
            ),
        ],
        vec![
            edge(
                root.clone(),
                "shared",
                vec![
                    ("type".into(), "javascript".into()),
                    ("mode".into(), "stable".into()),
                ],
                "case/shared.js",
            ),
            edge(root.clone(), "\\ud800", vec![], "case/shared.js"),
            edge(root, "self", vec![], "case/entry.js"),
        ],
    ));
    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
    );
    assert_eq!(
        outcome.output_events,
        lines(&[
            "selected:42",
            "attributes-denied",
            "attributes-denied",
            "url-denied",
            "surrogate-denied",
            "literal:42",
            "self:module",
        ])
    );
}

#[test]
fn root_nested_realms_and_agents_never_restore_an_ambient_loader() {
    let ambient =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/embedded_graph_ambient.js");
    let request_units = ambient
        .to_str()
        .unwrap()
        .encode_utf16()
        .map(|unit| unit.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let source = r#"
        const request = String.fromCharCode(__REQUEST_UNITS__);
        function deniedScript(label) {
            return "import(" + JSON.stringify(request) + ").then(() => print('ambient-escape'), () => print('" + label + "-denied'));";
        }
        const outer = $262.createRealm();
        const nested = "const inner = $262.createRealm(); inner.evalScript(" + JSON.stringify(deniedScript('nested')) + "); " + deniedScript('realm');
        outer.evalScript(nested);
        $262.agent.start(deniedScript('agent') + "$262.agent.leaving();");
        import(request).then(() => print('ambient-escape'), () => print('root-denied'));
        262;
    "#.replace("__REQUEST_UNITS__", &request_units);
    let outcome = script(graph(EmbeddedModuleGoal::Script, &source, vec![], vec![]));
    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
    );
    assert_eq!(
        outcome.output_events,
        lines(&[
            "nested-denied",
            "realm-denied",
            "agent-denied",
            "root-denied"
        ])
    );
}

#[test]
fn unlocated_contexts_share_source_policy_but_have_fresh_module_instances() {
    let source = r#"
        globalThis.contextName = 'root';
        const realm = $262.createRealm();
        realm.evalScript("globalThis.contextName = 'realm'; Promise.all([import('unlocated-shared'), import('unlocated-shared')]).then(([a,b]) => { if (a !== b) throw new Error('duplicate realm module'); print(a.describe()); });");
        $262.agent.start("globalThis.contextName = 'agent'; import('unlocated-shared').then(value => print(value.describe())); $262.agent.leaving();");
        import('root-shared').then(value => print(value.describe()));
        262;
    "#;
    let shared = r#"
        globalThis.moduleRuns = (globalThis.moduleRuns || 0) + 1;
        export function describe() { return globalThis.contextName + ':' + globalThis.moduleRuns + ':' + import.meta.url; }
    "#;
    let graph = graph(
        EmbeddedModuleGoal::Script,
        source,
        vec![module(
            "case/shared.js",
            shared,
            "lila://urls/context-shared.js",
        )],
        vec![
            edge(
                EmbeddedModuleReferrer::Script("case/entry.js".into()),
                "root-shared",
                vec![],
                "case/shared.js",
            ),
            edge(
                EmbeddedModuleReferrer::Unlocated,
                "unlocated-shared",
                vec![],
                "case/shared.js",
            ),
        ],
    );
    let outcome = script(Arc::clone(&graph));
    assert_eq!(
        outcome.output_events,
        lines(&[
            "realm:1:lila://urls/context-shared.js",
            "agent:1:lila://urls/context-shared.js",
            "root:1:lila://urls/context-shared.js",
        ])
    );
    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
    );
    let replayed = script(graph);
    assert_eq!(replayed.completion, outcome.completion);
    assert_eq!(replayed.output_events, outcome.output_events);
}

#[test]
fn source_then_defer_then_evaluation_reuses_parsed_source_without_early_effects() {
    let source = r#"
        (async function () {
            try { await import.source('phaseful'); throw new Error('source text object fabricated'); }
            catch (error) { if (!(error instanceof SyntaxError)) throw error; print('source-denied'); }
            if (globalThis.targetRuns !== undefined) throw new Error('source phase evaluated target');
            print('not-evaluated');
            const lazy = await import.defer('phaseful');
            if (globalThis.targetRuns !== undefined) throw new Error('defer eagerly evaluated target');
            print('deferred');
            print('value:' + lazy.answer);
            const eager = await import('phaseful');
            if (eager.answer !== 9) throw new Error('wrong eager export');
            print('runs:' + globalThis.targetRuns);
        })();
        262;
    "#;
    let target = "globalThis.targetRuns = (globalThis.targetRuns || 0) + 1; print('evaluated'); export const answer = 9;";
    let outcome = script(graph(
        EmbeddedModuleGoal::Script,
        source,
        vec![module(
            "case/phaseful.js",
            target,
            "lila://urls/phaseful.js",
        )],
        vec![edge(
            EmbeddedModuleReferrer::Script("case/entry.js".into()),
            "phaseful",
            vec![],
            "case/phaseful.js",
        )],
    ));
    assert_eq!(
        outcome.output_events,
        lines(&[
            "source-denied",
            "not-evaluated",
            "deferred",
            "evaluated",
            "value:9",
            "runs:1",
        ])
    );
    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
    );
}

#[test]
fn foreign_module_callback_retains_targets_and_first_metadata_in_its_source_realm() {
    let source = r#"
        const realm = $262.createRealm();
        realm.evalScript("import('parent').then(async value => { globalThis.callback = value.load; globalThis.firstCallback = value.first; globalThis.original = await value.load(); });");
        realm.global.callback().then(async value => {
            if (value !== realm.global.original) throw new Error('foreign Module target was duplicated');
            print('foreign-stable:' + value.answer);
            const fresh = await realm.global.firstCallback();
            if (fresh.rootThis !== realm.global || Object.getPrototypeOf(fresh.marker) !== realm.global.Array.prototype) throw new Error('first Module callback target has caller Realm');
            print('foreign-first:origin');
        });
        262;
    "#;
    let parent = r#"
        export function load() {
            print('foreign-url:' + import.meta.url);
            return import('child');
        }
        export function first() { return import('fresh'); }
    "#;
    let outcome = script(graph(
        EmbeddedModuleGoal::Script,
        source,
        vec![
            module(
                "case/foreign-parent.js",
                parent,
                "lila://urls/foreign-parent.js",
            ),
            module(
                "case/foreign-child.js",
                "export const answer = 73;",
                "lila://urls/foreign-child.js",
            ),
            module(
                "case/foreign-fresh.js",
                "export const rootThis = globalThis; export const marker = [];",
                "lila://urls/foreign-fresh.js",
            ),
        ],
        vec![
            edge(
                EmbeddedModuleReferrer::Unlocated,
                "parent",
                vec![],
                "case/foreign-parent.js",
            ),
            edge(
                EmbeddedModuleReferrer::Module("case/foreign-parent.js".into()),
                "child",
                vec![],
                "case/foreign-child.js",
            ),
            edge(
                EmbeddedModuleReferrer::Module("case/foreign-parent.js".into()),
                "fresh",
                vec![],
                "case/foreign-fresh.js",
            ),
        ],
    ));
    assert_eq!(
        outcome.output_events,
        lines(&[
            "foreign-url:lila://urls/foreign-parent.js",
            "foreign-url:lila://urls/foreign-parent.js",
            "foreign-stable:73",
            "foreign-first:origin",
        ])
    );
    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
    );
}

#[test]
fn foreign_callbacks_link_cached_source_and_new_static_imports_in_one_interner_domain() {
    let source = r#"
        const realm = $262.createRealm();
        realm.evalScript("import('parent').then(async value => { globalThis.evaluateCached = value.evaluateCached; globalThis.loadFresh = value.loadFresh; try { await value.preloadSource(); throw new Error('source unexpectedly succeeded'); } catch (error) { if (!(error instanceof SyntaxError)) throw error; print('foreign-source:denied'); } });");
        realm.global.evaluateCached().then(async cached => {
            if (cached.originOnlyExport !== 91 || realm.global.cachedRuns !== 1 || Object.getPrototypeOf(cached.originArray) !== realm.global.Array.prototype) throw new Error('cached source linked with the wrong symbols or Realm');
            print('foreign-cached:' + cached.originOnlyExport + ':' + realm.global.cachedRuns);
            const fresh = await realm.global.loadFresh();
            if (fresh.computed !== 92 || fresh.originOnlyExport !== 91 || fresh.rootThis !== realm.global || realm.global.cachedRuns !== 1) throw new Error('new foreign static import resolved through the wrong interner');
            print('foreign-static:' + fresh.computed);
        });
        262;
    "#;
    let parent = r#"
        export function preloadSource() { return import.source('cached'); }
        export function evaluateCached() { return import('cached'); }
        export function loadFresh() { return import('fresh'); }
    "#;
    let cached = r#"
        globalThis.cachedRuns = (globalThis.cachedRuns || 0) + 1;
        export const originOnlyExport = 91;
        export const originArray = [];
    "#;
    let fresh = r#"
        import { originOnlyExport as importedOrigin } from 'cached-static';
        export { originOnlyExport } from 'cached-static';
        export const computed = importedOrigin + 1;
        export const rootThis = globalThis;
    "#;
    let outcome = script(graph(
        EmbeddedModuleGoal::Script,
        source,
        vec![
            module(
                "case/interner-parent.js",
                parent,
                "lila://urls/interner-parent.js",
            ),
            module(
                "case/interner-cached.js",
                cached,
                "lila://urls/interner-cached.js",
            ),
            module(
                "case/interner-fresh.js",
                fresh,
                "lila://urls/interner-fresh.js",
            ),
        ],
        vec![
            edge(
                EmbeddedModuleReferrer::Unlocated,
                "parent",
                vec![],
                "case/interner-parent.js",
            ),
            edge(
                EmbeddedModuleReferrer::Module("case/interner-parent.js".into()),
                "cached",
                vec![],
                "case/interner-cached.js",
            ),
            edge(
                EmbeddedModuleReferrer::Module("case/interner-parent.js".into()),
                "fresh",
                vec![],
                "case/interner-fresh.js",
            ),
            edge(
                EmbeddedModuleReferrer::Module("case/interner-fresh.js".into()),
                "cached-static",
                vec![],
                "case/interner-cached.js",
            ),
        ],
    ));
    assert_eq!(
        outcome.output_events,
        lines(&[
            "foreign-source:denied",
            "foreign-cached:91:1",
            "foreign-static:92",
        ])
    );
    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
    );
}

#[test]
fn foreign_script_callback_retains_namespace_and_first_target_realm() {
    let source = r#"
        const realm = $262.createRealm();
        realm.evalScript("globalThis.callback = function () { return import('child'); }; globalThis.firstCallback = function () { return import('fresh'); }; globalThis.callback().then(value => { globalThis.original = value; });");
        realm.global.callback().then(async value => {
            if (value !== realm.global.original) throw new Error('foreign Script target was duplicated');
            print('script-stable:' + value.answer);
            const fresh = await realm.global.firstCallback();
            if (fresh.rootThis !== realm.global || Object.getPrototypeOf(fresh.marker) !== realm.global.Array.prototype) throw new Error('first Script callback target has caller Realm');
            print('script-first:' + fresh.url);
        });
        262;
    "#;
    let outcome = script(graph(
        EmbeddedModuleGoal::Script,
        source,
        vec![
            module("case/script-child.js", "export const answer = 29;", "lila://urls/script-child.js"),
            module("case/script-fresh.js", "export const rootThis = globalThis; export const marker = []; export const url = import.meta.url;", "lila://urls/script-fresh.js"),
        ],
        vec![
            edge(EmbeddedModuleReferrer::Unlocated, "child", vec![], "case/script-child.js"),
            edge(EmbeddedModuleReferrer::Unlocated, "fresh", vec![], "case/script-fresh.js"),
        ],
    ));
    assert_eq!(
        outcome.output_events,
        lines(&[
            "script-stable:29",
            "script-first:lila://urls/script-fresh.js"
        ])
    );
    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
    );
}

#[test]
fn invalid_loaded_source_and_entry_mismatch_do_not_evaluate_programs() {
    let source = "import 'bad'; print('must-not-evaluate');";
    let hooks = Arc::new(RecordingHooks::default());
    let graph = graph(
        EmbeddedModuleGoal::Module,
        source,
        vec![module(
            "case/bad.js",
            "export const = ;",
            "lila://urls/bad.js",
        )],
        vec![edge(
            EmbeddedModuleReferrer::Module("case/entry.js".into()),
            "bad",
            vec![],
            "case/bad.js",
        )],
    );
    let error = observe_module(
        source,
        Some("case/entry.js"),
        ModuleHostConfig {
            module_loading_policy: ModuleLoadingPolicy::Embedded(Arc::clone(&graph)),
            ..ModuleHostConfig::default()
        },
        &[],
        false,
        hooks.clone(),
    )
    .expect_err("a declared target's parser error precedes graph evaluation");
    assert!(!error.message().is_empty());
    assert!(hooks.0.lock().unwrap().is_empty());
    let wrong_source = observe_module(
        "print('must-not-evaluate');",
        Some("case/entry.js"),
        ModuleHostConfig {
            module_loading_policy: ModuleLoadingPolicy::Embedded(Arc::clone(&graph)),
            ..ModuleHostConfig::default()
        },
        &[],
        false,
        hooks.clone(),
    )
    .expect_err("caller source cannot replace the immutable entry");
    assert!(wrong_source
        .message()
        .contains("must match the graph entry"));
    let wrong_goal = observe_script_with_module_loading_policy(
        source,
        Some("case/entry.js"),
        ModuleLoadingPolicy::Embedded(graph),
        &[],
        false,
        hooks.clone(),
    )
    .expect_err("a Module entry cannot be reparsed as a Script");
    assert!(wrong_goal.message().contains("must match the graph entry"));
    assert!(hooks.0.lock().unwrap().is_empty());
}
