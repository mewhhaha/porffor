use scriptw_engine::{
    CompileOptions, EmbeddedModuleEntryInput, EmbeddedModuleGoal, EmbeddedModuleGraph,
    EmbeddedModuleInput, EmbeddedModuleReferrer, EmbeddedModuleResolutionInput,
    EmbeddedModuleSourceInput, EmbeddedModuleSourceValueInput, Engine, ExecutionBackend,
    HostOutputEvent, HostSurfacePolicy, ModuleLoadingPolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};
use scriptw_ir::{ModuleKey, ModuleSourceIr};

fn observe(
    entry: &str,
    files: &[(&str, &str)],
    sources: &[(&[&str], &str)],
) -> Result<scriptw_engine::ObservedRunOutcome, scriptw_engine::EngineError> {
    scriptw_engine::configure_compilation_jobs(1).unwrap();
    let mut modules = Vec::new();
    for (i, (_, source)) in files.iter().enumerate() {
        modules.push(EmbeddedModuleInput::SourceText(EmbeddedModuleSourceInput {
            identity: format!("module{i}.js"),
            source: (*source).into(),
            meta_url: format!("scriptw://module{i}.js"),
        }));
    }
    for (i, (_, tag)) in sources.iter().enumerate() {
        modules.push(EmbeddedModuleInput::ModuleSource(
            EmbeddedModuleSourceValueInput {
                identity: format!("native/source{i}"),
                source_tag: (*tag).into(),
                meta_url: format!("scriptw://native/source{i}"),
            },
        ));
    }
    let target = |specifier: &str| {
        files
            .iter()
            .position(|(name, _)| *name == specifier)
            .map(|i| format!("module{i}.js"))
            .or_else(|| {
                sources
                    .iter()
                    .position(|(names, _)| names.contains(&specifier))
                    .map(|i| format!("native/source{i}"))
            })
    };
    let mut resolutions = Vec::new();
    for (identity, body) in std::iter::once(("entry.js".to_owned(), entry)).chain(
        files
            .iter()
            .enumerate()
            .map(|(i, (_, body))| (format!("module{i}.js"), *body)),
    ) {
        let parsed = ModuleSourceIr::new(
            ModuleKey::from_host(&identity),
            body.into(),
            format!("scriptw://{identity}"),
        );
        for request in parsed.module_requests().unwrap() {
            if let Some(target) = target(request.specifier()) {
                resolutions.push(EmbeddedModuleResolutionInput {
                    referrer: EmbeddedModuleReferrer::Module(identity.clone()),
                    specifier: request.specifier().into(),
                    attributes: request
                        .attributes()
                        .iter()
                        .map(|a| (a.key.clone(), a.value.clone()))
                        .collect(),
                    target,
                });
            }
        }
    }
    // Realm imports have their own explicit domain, independent of a caller.
    for (i, (specifier, _)) in files.iter().enumerate() {
        resolutions.push(EmbeddedModuleResolutionInput {
            referrer: EmbeddedModuleReferrer::Realm,
            specifier: (*specifier).into(),
            attributes: Vec::new(),
            target: format!("module{i}.js"),
        });
    }
    let graph = EmbeddedModuleGraph::try_new_typed(
        EmbeddedModuleEntryInput {
            goal: EmbeddedModuleGoal::Module,
            identity: "entry.js".into(),
            source: entry.into(),
            meta_url: "scriptw://entry.js".into(),
        },
        modules,
        resolutions,
    )
    .unwrap();
    Engine::new(RealmBuilder::new().build()).observe_module(
        entry,
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
}

fn run(entry: &str, files: &[(&str, &str)], sources: &[(&[&str], &str)]) {
    let observed = observe(entry, files, sources).unwrap();
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        observed.completion
    );
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine("native-module-source:ok".into())]
    );
}

#[test]
fn original_three_pinned_source_reexport_bodies_run_with_a_real_host_source() {
    const STAR_ENTRY: &str = include_str!("../../../test262/vendor/test262/test/language/module-code/ambiguous-export-bindings/namespace-unambiguous-if-import-source-and-export.js");
    const ONE: &str = include_str!("../../../test262/vendor/test262/test/language/module-code/ambiguous-export-bindings/namespace-import-source-and-export-1_FIXTURE.js");
    const TWO: &str = include_str!("../../../test262/vendor/test262/test/language/module-code/ambiguous-export-bindings/namespace-import-source-and-export-2_FIXTURE.js");
    const STAR: &str = include_str!("../../../test262/vendor/test262/test/language/module-code/ambiguous-export-bindings/namespace-import-source-and-export-reexport_FIXTURE.js");
    const NAMED: &str = include_str!("../../../test262/vendor/test262/test/language/module-code/source-phase-import/reexport-source-binding-named-import.js");
    const NAMESPACE: &str = include_str!("../../../test262/vendor/test262/test/language/module-code/source-phase-import/reexport-source-binding-namespace-get.js");
    const REEXPORT: &str = include_str!("../../../test262/vendor/test262/test/language/module-code/source-phase-import/reexport-source-binding_FIXTURE.js");
    const STA: &str = include_str!("../../../test262/vendor/test262/harness/sta.js");
    const ASSERT: &str = include_str!("../../../test262/vendor/test262/harness/assert.js");
    for (body, files) in [
        (
            STAR_ENTRY,
            vec![
                ("./namespace-import-source-and-export-1_FIXTURE.js", ONE),
                ("./namespace-import-source-and-export-2_FIXTURE.js", TWO),
                (
                    "./namespace-import-source-and-export-reexport_FIXTURE.js",
                    STAR,
                ),
            ],
        ),
        (
            NAMED,
            vec![("./reexport-source-binding_FIXTURE.js", REEXPORT)],
        ),
        (
            NAMESPACE,
            vec![("./reexport-source-binding_FIXTURE.js", REEXPORT)],
        ),
    ] {
        let entry = format!("{STA}\n{ASSERT}\nconst $262 = {{ AbstractModuleSource: __scriptwGetAbstractModuleSource() }};\n{body}\nprint('native-module-source:ok');");
        run(&entry, &files, &[(&["<module source>"], "ModuleSource")]);
    }
}

#[test]
fn source_cells_preserve_alias_identity_brand_unicode_and_gc_without_namespace_aliasing() {
    run(
        r#"
import source first from 'host:first';
import source alias from 'host:alias';
import source other from 'host:other';
import { value } from './wrapper.js';
import * as wrapper from './wrapper.js';
import * as empty from './empty.js';
const C = __scriptwGetAbstractModuleSource();
function check(x) { if (!x) throw 'source identity or brand'; }
const tag = Object.getOwnPropertyDescriptor(C.prototype, Symbol.toStringTag).get;
check(first === alias && first === value && first === wrapper.value && first !== other);
const sourcePrototype = Object.getPrototypeOf(first);
check(sourcePrototype !== C.prototype && Object.getPrototypeOf(sourcePrototype) === C.prototype);
check(Object.getPrototypeOf(other) === sourcePrototype && first instanceof C);
check(tag.call(sourcePrototype) === undefined);
check(tag.call(first) === '源🌐' && tag.call(other) === '源🌐');
check(tag.call(C.prototype) === undefined && tag.call(Object.create(C.prototype)) === undefined);
check(tag.call(new Proxy(first, {})) === undefined);
check(Object.prototype.toString.call(first) === '[object 源🌐]');
check(Object.keys(empty).length === 0 && empty !== first);
first.user = 17; gc(); check(first.user === 17 && tag.call(first) === '源🌐');
Object.setPrototypeOf(first, null); check(tag.call(first) === '源🌐');
print('native-module-source:ok');
"#,
        &[
            (
                "./wrapper.js",
                "import source value from 'host:first'; export { value };",
            ),
            ("./empty.js", ""),
        ],
        &[
            (&["host:first", "host:alias"], "源🌐"),
            (&["host:other"], "源🌐"),
        ],
    );
}

#[test]
fn dynamic_source_imports_share_static_cells_and_keep_other_failure_owners() {
    run(
        r#"
import source first from 'host:first';
const result = await import.source('host:first');
if (result !== first) throw 'dynamic source identity';
const again = await import.source('host:alias');
if (again !== first) throw 'dynamic alias identity';
try { await import.source('./ordinary.js'); throw 'ordinary source fulfilled'; }
catch (e) { if (!(e instanceof SyntaxError) || !e.message.includes('no source representation')) throw e; }
try { await import.source('missing'); throw 'missing source fulfilled'; }
catch (e) { if (!(e instanceof TypeError)) throw e; }
const marker = {};
try { await import.source({toString(){throw marker}}); throw 'coercion fulfilled'; }
catch (e) { if (e !== marker) throw 'coercion error replaced'; }
gc(); if ((await import.source('host:first')) !== first) throw 'source GC identity';
print('native-module-source:ok');
"#,
        &[("./ordinary.js", "throw 'source target evaluated';")],
        &[(&["host:first", "host:alias"], "ModuleSource")],
    );
}

#[test]
fn source_objects_keep_the_actual_defining_realm_across_shadow_realm_module_graphs() {
    run(
        r#"
import source first from 'host:first';
const C = __scriptwGetAbstractModuleSource();
const sourcePrototype = Object.getPrototypeOf(first);
sourcePrototype.realmMarker = 17;
const remote = new ShadowRealm();
const check = await remote.importValue('./realm.js', 'check');
if (check() !== 'remote-source:ok') throw 'foreign source owner';
if (Object.getPrototypeOf(Object.getPrototypeOf(first)) !== C.prototype) throw 'root source owner changed';
gc(); if (check() !== 'remote-source:ok') throw 'foreign source lost across GC';
print('native-module-source:ok');
"#,
        &[(
            "./realm.js",
            r#"
import source first from 'host:first';
import source alias from 'host:alias';
const C = Object.getPrototypeOf(Object.getPrototypeOf(first)).constructor;
export function check() {
  const getter = Object.getOwnPropertyDescriptor(C.prototype, Symbol.toStringTag).get;
  if (Object.getPrototypeOf(first).realmMarker !== undefined) throw 'host prototype crossed Realms';
  if (first !== alias || Object.getPrototypeOf(Object.getPrototypeOf(first)) !== C.prototype || !(first instanceof C) || getter.call(first) !== 'ModuleSource') throw 'remote source brand';
  return 'remote-source:ok';
}
"#,
        )],
        &[(&["host:first", "host:alias"], "ModuleSource")],
    );
}

#[path = "aot_module_sources/evaluation_capability.rs"]
mod evaluation_capability;

#[path = "aot_module_sources/prototype_chain.rs"]
mod prototype_chain;
