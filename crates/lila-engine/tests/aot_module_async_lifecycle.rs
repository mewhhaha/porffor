use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedRunOutcome, PromiseRejectionPolicy, RealmBuilder,
    RunOptions,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

struct Modules(PathBuf);
impl Drop for Modules {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn observe(
    files: &[(&str, &str)],
    prelude: Option<&str>,
    policy: PromiseRejectionPolicy,
) -> ObservedRunOutcome {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let fixture = Modules(std::env::temp_dir().join(format!(
        "lila-async-module-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir_all(&fixture.0).unwrap();
    for (name, source) in files {
        std::fs::write(fixture.0.join(name), source).unwrap();
    }
    lila_engine::configure_compilation_jobs(1).unwrap();
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_module(
            files[0].1,
            CompileOptions {
                filename: Some(fixture.0.join(files[0].0).to_str().unwrap().into()),
                module_root: Some(fixture.0.to_str().unwrap().into()),
                module_prelude: prelude.map(str::to_owned),
                host_surface_policy: HostSurfacePolicy::Test262,
                promise_rejection_policy: policy,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("compiled Module graph executes");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    observed
}

fn success(files: &[(&str, &str)], expected: &[&str], prelude: Option<&str>) {
    let observed = observe(files, prelude, PromiseRejectionPolicy::FailRun);
    assert_eq!(
        observed.completion,
        ObservedCompletion::Normal(ObservedJsValue::Undefined)
    );
    assert_eq!(
        observed.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn async_owners_keep_live_import_cells_separate_from_suspended_lexical_chains() {
    success(&[
        ("entry.js", r#"import { value, replace, read } from './dependency.js';
const shared = 'entry';
if (typeof arguments !== 'undefined' || this !== undefined) throw 'module lexical owner';
{ let value = 'shadow'; await 0; if (value !== 'shadow') throw 'resumed lexical scope'; }
if (value !== 3 || read() !== 'dependency') throw 'canonical import environment';
replace(8); await 0;
if (value !== 8 || helperRead() !== 'global' || shared !== 'entry') throw 'live or global capture';
print('canonical async environments');"#),
        ("dependency.js", "const shared = 'dependency'; export let value = 3; export function replace(next) { value = next; } export function read() { return shared; } await 0;"),
    ], &["canonical async environments"], Some("const shared = 'global'; function helperRead() { return shared; }"));
}

#[test]
fn deferred_requests_promote_async_leaves_without_evaluating_synchronous_wrappers() {
    success(
        &[
            (
                "entry.js",
                r#"import './setup.js'; import defer * as left from './left.js'; import defer * as right from './right.js';
if (events.join(',') !== 'A start,B start,A end,B end') throw events.join(',');
events.push('entry');
if (left.value !== 1 || right.value !== 2) throw 'exports';
if (events.join(',') !== 'A start,B start,A end,B end,entry,left,right') throw events.join(',');
print('flattened deferred dependencies');"#,
            ),
            ("setup.js", "globalThis.events = [];"),
            (
                "left.js",
                "import { value } from './a.js'; events.push('left'); export { value };",
            ),
            (
                "right.js",
                "import { value } from './b.js'; events.push('right'); export { value };",
            ),
            (
                "a.js",
                "events.push('A start'); await 0; events.push('A end'); export const value = 1;",
            ),
            (
                "b.js",
                "events.push('B start'); await 0; events.push('B end'); export const value = 2;",
            ),
        ],
        &["flattened deferred dependencies"],
        None,
    );
}

#[test]
fn executing_async_self_and_transitive_deferred_reads_do_not_poison_evaluation() {
    success(&[
        ("entry.js", r#"import defer * as self from './entry.js'; import defer * as dependent from './dependent.js';
globalThis.dependentCalls = 0;
function check() { for (const read of [() => self.missing, () => dependent.value]) { let caught = false; try { read(); } catch (error) { caught = error instanceof TypeError; } if (!caught) throw 'active namespace accepted'; } }
check(); await 0; check();
if (dependentCalls !== 0) throw 'dependent body entered';
export const value = 9;
import('./entry.js').then(ns => { if (ns.value !== 9 || dependent.value !== 9 || dependentCalls !== 1) throw 'post-completion access'; print('readiness is retryable'); });"#),
        ("dependent.js", "import { value } from './entry.js'; globalThis.dependentCalls++; export { value };"),
    ], &["readiness is retryable"], None);
}

#[test]
fn duplicate_dependencies_on_one_runtime_cycle_root_release_the_parent_once() {
    success(&[
        ("entry.js", "import './setup.js'; import { a } from './a.js'; import { b } from './b.js'; events.push('entry'); if (a + b !== 3 || events.join(',') !== 'B start,B end,A start,A end,entry') throw events.join(','); print('cycle occurrence counts');"),
        ("setup.js", "globalThis.events = [];"),
        ("a.js", "import './b.js'; events.push('A start'); await 0; events.push('A end'); export const a = 1;"),
        ("b.js", "import './a.js'; events.push('B start'); await 0; events.push('B end'); export const b = 2;"),
    ], &["cycle occurrence counts"], None);
}

#[test]
fn internal_evaluation_waits_do_not_observe_mutable_promise_hooks() {
    success(&[
        ("entry.js", "import { value } from './dependency.js'; if (value !== 7) throw 'export'; print('intrinsic evaluation waits');"),
        ("dependency.js", r#"Object.defineProperty(Promise.prototype, 'constructor', { get() { throw 'constructor hook'; }, configurable: true });
Object.defineProperty(Promise, Symbol.species, { get() { throw 'species hook'; }, configurable: true });
Promise.prototype.then = function() { throw 'then hook'; };
await 0; export const value = 7;"#),
    ], &["intrinsic evaluation waits"], None);
}

#[test]
fn ordinary_source_await_still_performs_promise_resolve() {
    success(
        &[(
            "entry.js",
            r#"const sentinel = {}; const promise = Promise.resolve(0);
Object.defineProperty(promise, 'constructor', { get() { throw sentinel; } });
let caught = false; try { await promise; } catch (error) { caught = error === sentinel; }
if (!caught) throw 'source Await bypassed constructor'; print('ordinary await protocol');"#,
        )],
        &["ordinary await protocol"],
        None,
    );
}

#[test]
fn dynamic_deferred_join_starts_all_leaves_and_preserves_namespace_identity() {
    success(&[
        ("entry.js", r#"globalThis.events = []; globalThis.wrapperCalls = 0;
Promise.all = function() { throw 'public Promise.all'; };
const first = await import.defer('./wrapper.js');
if (wrapperCalls !== 0 || events.join(',') !== 'A start,B start,A end,B end') throw events.join(',');
const second = await import.defer('./wrapper.js');
if (first !== second || wrapperCalls !== 0) throw 'namespace identity';
if (first.value !== 3 || wrapperCalls !== 1 || second.value !== 3) throw 'deferred body';
print('parallel deferred join');"#),
        ("wrapper.js", "import { a } from './a.js'; import { b } from './b.js'; globalThis.wrapperCalls++; export const value = a + b;"),
        ("a.js", "events.push('A start'); await 0; events.push('A end'); export const a = 1;"),
        ("b.js", "events.push('B start'); await 0; events.push('B end'); export const b = 2;"),
    ], &["parallel deferred join"], None);
}

#[test]
fn concurrent_imports_keep_fresh_promises_and_the_exact_undefined_rejection() {
    success(
        &[
            (
                "entry.js",
                r#"const first = import('./bad.js'); const second = import('./bad.js'); if (first === second) throw 'reused import promise';
let count = 0;
try { await first; } catch (error) { if (error !== undefined) throw 'replaced rejection'; count++; }
try { await second; } catch (error) { if (error !== undefined) throw 'replaced rejection'; count++; }
if (count !== 2) throw 'missing rejection'; print('undefined import rejection');"#,
            ),
            ("bad.js", "await 0; throw undefined;"),
        ],
        &["undefined import rejection"],
        None,
    );
}

#[test]
fn a_late_async_sibling_cannot_replace_or_reenter_an_aborted_entry() {
    for policy in [
        PromiseRejectionPolicy::Ignore,
        PromiseRejectionPolicy::FailRun,
    ] {
        let observed = observe(&[
            ("entry.js", "import './async.js'; import './failure.js'; throw 'entry body must not execute';"),
            ("async.js", "await 0; print('late sibling completed');"),
            ("failure.js", "throw undefined;"),
        ], None, policy);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Throw(ObservedJsValue::Undefined)
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine("late sibling completed".into())]
        );
    }
}

#[test]
fn malformed_dynamic_only_targets_reject_their_import_after_a_tla_entry_resumes() {
    success(&[
        ("entry.js", "await 0; let caught = false; try { await import('./invalid.js'); } catch (error) { caught = error instanceof SyntaxError; } if (!caught) throw 'missing syntax rejection'; print('isolated dynamic syntax rejection');"),
        ("invalid.js", "invalid syntax!"),
    ], &["isolated dynamic syntax rejection"], None);
}

#[test]
fn async_module_resource_lifetimes_begin_after_instantiation_and_dispose_after_await() {
    success(
        &[(
            "entry.js",
            r#"const events = [];
{ using resource = { [Symbol.dispose]() { events.push('dispose'); } }; events.push('before'); await 0; events.push('after'); }
if (events.join(',') !== 'before,after,dispose') throw events.join(',');
print('async module resource lifetime');"#,
        )],
        &["async module resource lifetime"],
        None,
    );
}
