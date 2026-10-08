use std::sync::Arc;

use lila_runtime::{
    EmbeddedModuleEntryInput, EmbeddedModuleGoal, EmbeddedModuleGraph, EmbeddedModuleInput,
    EmbeddedModuleReferrer, EmbeddedModuleResolutionInput, EmbeddedModuleSourceInput,
    HostOutputEvent, ModuleLoadingPolicy, NullHostHooks, ObservedCompletion, ObservedJsValue,
    ObservedNumber,
};
use lila_spec_exec::{observe_module, observe_script_with_module_loading_policy, ModuleHostConfig};

fn graph(
    goal: EmbeddedModuleGoal,
    source: &str,
    records: &[(&str, &str)],
    requests: &[(EmbeddedModuleReferrer, &str, &str)],
) -> Arc<EmbeddedModuleGraph> {
    EmbeddedModuleGraph::try_new_typed(
        EmbeddedModuleEntryInput {
            goal,
            identity: "entry.js".into(),
            source: source.into(),
            meta_url: "lila://entry".into(),
        },
        records
            .iter()
            .map(|(identity, source)| {
                EmbeddedModuleInput::Json(EmbeddedModuleSourceInput {
                    identity: (*identity).into(),
                    source: (*source).into(),
                    meta_url: format!("lila://{identity}"),
                })
            })
            .collect(),
        requests
            .iter()
            .map(
                |(referrer, specifier, target)| EmbeddedModuleResolutionInput {
                    referrer: referrer.clone(),
                    specifier: (*specifier).into(),
                    attributes: vec![("type".into(), "json".into())],
                    target: (*target).into(),
                },
            )
            .collect(),
    )
    .unwrap()
}

fn execute(graph: Arc<EmbeddedModuleGraph>) -> lila_spec_exec::ObservedExecutionOutcome {
    let source = graph.entry().source();
    let filename = Some(graph.entry().identity());
    let policy = ModuleLoadingPolicy::Embedded(Arc::clone(&graph));
    match graph.entry().goal() {
        EmbeddedModuleGoal::Module => observe_module(
            source,
            filename,
            ModuleHostConfig {
                module_loading_policy: policy,
                ..ModuleHostConfig::default()
            },
            &[],
            false,
            Arc::new(NullHostHooks),
        ),
        EmbeddedModuleGoal::Script => observe_script_with_module_loading_policy(
            source,
            filename,
            policy,
            &[],
            false,
            Arc::new(NullHostHooks),
        ),
    }
    .expect("the original embedded execution reaches its completion")
}

fn lines(expected: &[&str]) -> Vec<HostOutputEvent> {
    expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).into()))
        .collect()
}

#[test]
fn json_default_retains_data_descriptors_numbers_utf16_and_shared_identity() {
    let source = r#"
        import data from 'data' with { type: 'json' };
        import * as alias from 'alias' with { type: 'json' };
        const dynamic = await import('data', { with: { type: 'json' } });
        if (alias !== dynamic || data !== alias.default) throw 'duplicate JSON identity';
        if (Object.keys(alias).join(',') !== 'default') throw 'JSON export surface';
        if (Object.getPrototypeOf(data) !== Object.prototype) throw 'JSON prototype';
        const proto = Object.getOwnPropertyDescriptor(data, '__proto__');
        if (proto.value !== 2 || !proto.writable || !proto.enumerable || !proto.configurable) throw 'JSON data key';
        if (Object.keys(data).join(',') !== '__proto__,surrogate,negative,large,nested') throw 'duplicate key order';
        if (data.surrogate.length !== 1 || data.surrogate.charCodeAt(0) !== 0xd800) throw 'JSON UTF16';
        if (!Object.is(data.negative, -0) || data.large !== Infinity) throw 'JSON number semantics';
        if (Object.getPrototypeOf(data.nested) !== Array.prototype || data.nested[0].ok !== true) throw 'JSON nested data';
        data.nested.push(7);
        if ((await import('alias', { with: { type: 'json' } })).default.nested[1] !== 7) throw 'JSON mutation identity';
        print('json-data');
    "#;
    let root = EmbeddedModuleReferrer::Module("entry.js".into());
    let observed = execute(graph(
        EmbeddedModuleGoal::Module,
        source,
        &[(
            "data.json",
            r#"{"__proto__":1,"surrogate":"\ud800","negative":-0,"large":1e400,"nested":[{"ok":true}],"__proto__":2}"#,
        )],
        &[
            (root.clone(), "data", "data.json"),
            (root, "alias", "data.json"),
        ],
    ));
    assert_eq!(
        observed.completion,
        ObservedCompletion::Normal(ObservedJsValue::Undefined)
    );
    assert_eq!(observed.output_events, lines(&["json-data"]));
}

#[test]
fn json_parse_and_request_rejections_keep_dynamic_import_timing() {
    let source = r#"
        print('before');
        const pending = import(
            { toString() { print('specifier'); return 'bad'; } },
            { get with() { print('attributes'); return { type: 'json' }; } }
        );
        print('after-request');
        try { await pending; throw 'invalid JSON accepted'; }
        catch (error) { if (!(error instanceof SyntaxError)) throw error; print('syntax'); }
        for (const options of [undefined, { with: { type: 'text' } }]) {
            try { await import('data', options); throw 'wrong request accepted'; }
            catch (error) { if (!(error instanceof TypeError)) throw error; print('request'); }
        }
        const value = await import('data', { with: { type: 'json' } });
        if (value.default !== 42) throw 'JSON primitive default';
        try {
            await import.source(
                { toString() { print('source-specifier'); return 'data'; } },
                { get with() { print('source-attributes'); return { type: 'json' }; } }
            );
            throw 'JSON source fabricated';
        }
        catch (error) { if (!(error instanceof SyntaxError)) throw error; print('source-unavailable'); }
    "#;
    let root = EmbeddedModuleReferrer::Module("entry.js".into());
    let observed = execute(graph(
        EmbeddedModuleGoal::Module,
        source,
        &[("bad.json", "{"), ("data.json", "42")],
        &[
            (root.clone(), "bad", "bad.json"),
            (root, "data", "data.json"),
        ],
    ));
    assert_eq!(
        observed.completion,
        ObservedCompletion::Normal(ObservedJsValue::Undefined),
        "actual rejection stage is retained by the ordered transcript: {:?}; {}",
        observed.output_events,
        observed.note,
    );
    assert_eq!(
        observed.output_events,
        lines(&[
            "before",
            "specifier",
            "attributes",
            "after-request",
            "syntax",
            "request",
            "request",
            "source-specifier",
            "source-attributes",
            "source-unavailable"
        ])
    );
}

#[test]
fn borrowed_foreign_json_imports_allocate_data_and_errors_in_the_referrer_realm() {
    let source = r#"
        const realm = $262.createRealm();
        realm.evalScript("globalThis.load = function () { return import('data', {with:{type:'json'}}); }; globalThis.bad = function () { return import('bad', {with:{type:'json'}}); }; globalThis.source = function () { return import.source('data', {with:{type:'json'}}); };");
        Promise.all([import('data', {with:{type:'json'}}), realm.global.load()]).then(async values => {
            const root = values[0].default, foreign = values[1].default;
            if (root === foreign) throw 'JSON records escaped their Realm cache';
            if (Object.getPrototypeOf(root) !== Object.prototype) throw 'root JSON Realm';
            if (Object.getPrototypeOf(foreign) !== realm.global.Object.prototype || Object.getPrototypeOf(foreign.list) !== realm.global.Array.prototype) throw 'foreign JSON Realm';
            if ((await realm.global.load()) !== values[1]) throw 'foreign JSON cache identity';
            try { await realm.global.bad(); throw 'bad foreign JSON accepted'; }
            catch (error) { if (Object.getPrototypeOf(error) !== realm.global.SyntaxError.prototype) throw 'foreign JSON parse error Realm'; }
            try { await realm.global.source(); throw 'foreign JSON source fabricated'; }
            catch (error) { if (Object.getPrototypeOf(error) !== realm.global.SyntaxError.prototype) throw 'foreign JSON source error Realm'; }
            print('json-realms');
        });
        262;
    "#;
    let observed = execute(graph(
        EmbeddedModuleGoal::Script,
        source,
        &[("data.json", r#"{"list":[]}"#), ("bad.json", "{")],
        &[
            (
                EmbeddedModuleReferrer::Script("entry.js".into()),
                "data",
                "data.json",
            ),
            (EmbeddedModuleReferrer::Unlocated, "data", "data.json"),
            (EmbeddedModuleReferrer::Unlocated, "bad", "bad.json"),
        ],
    ));
    assert_eq!(
        observed.completion,
        ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
    );
    assert_eq!(observed.output_events, lines(&["json-realms"]));
}
