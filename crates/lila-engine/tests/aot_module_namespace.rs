use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

struct NamespaceModules(PathBuf);

impl Drop for NamespaceModules {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn assert_namespace_modules(files: &[(&str, &str)], expected: &[&str]) {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let fixture = NamespaceModules(std::env::temp_dir().join(format!(
        "lila-module-namespace-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir_all(&fixture.0).expect("create module fixture");
    for (name, source) in files {
        std::fs::write(fixture.0.join(name), source).expect("write module fixture");
    }
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_module(
            files[0].1,
            CompileOptions {
                filename: Some(fixture.0.join(files[0].0).to_str().unwrap().into()),
                module_root: Some(fixture.0.to_str().unwrap().into()),
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("module namespace fixture compiles and executes through Wasm");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        observed.completion
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
fn namespace_descriptor_queries_read_tdz_bindings_while_keys_and_has_do_not() {
    assert_namespace_modules(
        &[(
            "entry.js",
            r#"
import * as ns from './entry.js';
function expectReferenceError(action) {
  let received;
  try { action(); } catch (error) { received = error; }
  if (!(received instanceof ReferenceError)) throw 'namespace TDZ';
}
if (Object.getOwnPropertyNames(ns).join('|') !== 'later' ||
    Reflect.ownKeys(ns)[0] !== 'later' || Reflect.ownKeys(ns)[1] !== Symbol.toStringTag ||
    Object.getOwnPropertySymbols(ns)[0] !== Symbol.toStringTag ||
    !('later' in ns) || !Reflect.has(ns, 'later') || !('later' in Object.create(ns))) throw 'keys or Has';
expectReferenceError(() => ns.later);
expectReferenceError(() => Object.getOwnPropertyDescriptor(ns, 'later'));
expectReferenceError(() => Reflect.getOwnPropertyDescriptor(ns, 'later'));
expectReferenceError(() => Object.keys(ns));
expectReferenceError(() => Object.hasOwn(ns, 'later'));
expectReferenceError(() => Object.prototype.hasOwnProperty.call(ns, 'later'));
expectReferenceError(() => Object.prototype.propertyIsEnumerable.call(ns, 'later'));
if (Object.getOwnPropertyDescriptor(ns, 'absent') !== undefined ||
    Object.prototype.hasOwnProperty.call(ns, 'absent') || Reflect.has(ns, 'absent')) throw 'absent export';
export let later = 42;
const descriptor = Object.getOwnPropertyDescriptor(ns, 'later');
if (descriptor.value !== 42 || descriptor.writable !== true || descriptor.enumerable !== true ||
    descriptor.configurable !== false || 'get' in descriptor || 'set' in descriptor) throw 'data descriptor';
print('namespace TDZ and keys');
"#,
        )],
        &["namespace TDZ and keys"],
    );
}

#[test]
fn namespace_receivers_read_tdz_for_super_data_writes_but_setters_run_directly() {
    assert_namespace_modules(
        &[(
            "entry.js",
            r#"
import * as ns from './entry.js';
class Base { constructor() { return ns; } }
class DataWrite extends Base {
  constructor() { super(); super.later = 14; }
}
let received;
try { new DataWrite(); } catch (error) { received = error; }
if (!(received instanceof ReferenceError)) throw 'super receiver descriptor did not read TDZ';
let setterCalls = 0;
class SetterBase {
  constructor() { return ns; }
  set later(value) {
    if (this !== ns || value !== 14) throw 'setter receiver';
    setterCalls++;
  }
}
class SetterWrite extends SetterBase {
  constructor() { super(); super.later = 14; }
}
new SetterWrite();
if (setterCalls !== 1) throw 'super setter did not run';
if (Reflect.set(ns, 'later', 1) !== false || Reflect.set(ns, 'absent', 1) !== false ||
    Reflect.set(ns, Symbol.toStringTag, 'Other') !== false ||
    Reflect.set(Object.create(ns), 'later', 1) !== false) throw 'namespace Set';
let assignmentError;
try { ns.later = 1; } catch (error) { assignmentError = error; }
if (!(assignmentError instanceof TypeError)) throw 'strict namespace assignment';
export let later = 42;
print('namespace super receivers');
"#,
        )],
        &["namespace super receivers"],
    );
}

#[test]
fn namespace_descriptors_are_live_data_and_define_property_uses_same_value() {
    assert_namespace_modules(
        &[
            (
                "entry.js",
                r#"
import * as ns from './value.js';
function checkDescriptor(value) {
  const descriptor = Object.getOwnPropertyDescriptor(ns, 'value');
  if (!Object.is(descriptor.value, value) || !descriptor.writable || !descriptor.enumerable ||
      descriptor.configurable || 'get' in descriptor || 'set' in descriptor) throw 'namespace descriptor';
  return descriptor;
}
const original = checkDescriptor(-0);
const inherited = Object.create(ns);
if (!Object.is(inherited.value, -0) || Object.getPrototypeOf(ns) !== null || Object.isExtensible(ns)) throw 'namespace shape';
if (!Reflect.defineProperty(ns, 'value', {}) ||
    !Reflect.defineProperty(ns, 'value', {value: -0, writable: true, enumerable: true, configurable: false}) ||
    Reflect.defineProperty(ns, 'value', {value: 0}) ||
    Reflect.defineProperty(ns, 'value', {writable: false}) ||
    Reflect.defineProperty(ns, 'value', {enumerable: false}) ||
    Reflect.defineProperty(ns, 'value', {configurable: true}) ||
    Reflect.defineProperty(ns, 'value', {get() { return 1; }}) ||
    Reflect.defineProperty(ns, 'missing', {})) throw 'namespace DefineOwnProperty';
ns.replace(NaN);
checkDescriptor(NaN);
if (!Object.is(original.value, -0) || !Number.isNaN(inherited.value) ||
    !Reflect.defineProperty(ns, 'value', {value: NaN})) throw 'live value or SameValue';
ns.replace(17);
if (inherited.value !== 17 || checkDescriptor(17).value !== 17) throw 'inherited live read';
if (Object.seal(ns) !== ns || !Object.isSealed(ns) || Object.isFrozen(ns)) throw 'seal namespace';
let freezeError;
try { Object.freeze(ns); } catch (error) { freezeError = error; }
if (!(freezeError instanceof TypeError)) throw 'freeze writable export';
const tag = Object.getOwnPropertyDescriptor(ns, Symbol.toStringTag);
if (tag.value !== 'Module' || tag.writable || tag.enumerable || tag.configurable) throw 'tag descriptor';
print('namespace live descriptors');
"#,
            ),
            (
                "value.js",
                "export let value = -0; export function replace(next) { value = next; }",
            ),
        ],
        &["namespace live descriptors"],
    );
}

#[test]
fn namespace_keys_use_utf16_lexicographic_order_and_proxy_invariants_use_data_descriptors() {
    assert_namespace_modules(
        &[
            (
                "entry.js",
                r#"
import * as ns from './exports.js';
const names = Object.getOwnPropertyNames(ns);
if (names.join('|') !== '10|2|a|\uD83D\uDE00|\uE000' ||
    Object.keys(ns).join('|') !== names.join('|')) throw 'namespace key ordering';
const keys = Reflect.ownKeys(ns);
if (keys.length !== 6 || keys[5] !== Symbol.toStringTag) throw 'namespace symbol order';
const transparent = new Proxy(ns, {});
const descriptor = Object.getOwnPropertyDescriptor(transparent, '2');
if (descriptor.value !== 4 || !descriptor.writable || descriptor.configurable ||
    'get' in descriptor || Reflect.ownKeys(transparent)[0] !== '10') throw 'transparent proxy';
let trapCalls = 0;
const forwarding = new Proxy(ns, {
  getOwnPropertyDescriptor(target, key) { trapCalls++; return Reflect.getOwnPropertyDescriptor(target, key); },
  ownKeys(target) { return Reflect.ownKeys(target); }
});
if (Object.keys(forwarding).join('|') !== names.join('|') || trapCalls !== 5) throw 'trapped proxy forwarding';
function expectTypeError(action) {
  let received;
  try { action(); } catch (error) { received = error; }
  if (!(received instanceof TypeError)) throw 'proxy invariant';
}
expectTypeError(() => Object.getOwnPropertyDescriptor(new Proxy(ns, {
  getOwnPropertyDescriptor() { return {value: 4, writable: false, enumerable: true, configurable: false}; }
}), '2'));
expectTypeError(() => Reflect.ownKeys(new Proxy(ns, {ownKeys() { return ['2']; }})));
print('namespace key order and proxies');
"#,
            ),
            (
                "exports.js",
                r"const value = 4; export {value as '2', value as '10', value as 'a', value as '\uD83D\uDE00', value as '\uE000'};",
            ),
        ],
        &["namespace key order and proxies"],
    );
}

#[test]
fn deferred_namespace_symbols_and_then_do_not_evaluate_but_an_absent_string_get_does() {
    assert_namespace_modules(
        &[
            (
                "entry.js",
                r#"
import defer * as ns from './deferred.js';
print('before deferred');
if (ns[Symbol.toStringTag] !== 'Deferred Module' || ns[Symbol('missing')] !== undefined || ns.then !== undefined ||
    Reflect.has(ns, 'then') || Reflect.getOwnPropertyDescriptor(ns, 'then') !== undefined ||
    !Reflect.has(ns, Symbol.toStringTag) || !Reflect.deleteProperty(ns, 'then') ||
    Reflect.defineProperty(ns, 'then', {value: 1}) || Reflect.set(ns, 'value', 2)) throw 'symbol-like key';
if (Object.getPrototypeOf(ns) !== null || Object.isExtensible(ns)) throw 'namespace shape';
print('after symbol-like keys');
if (ns.missing !== undefined || ns.value !== 7 || ns.value !== 7) throw 'deferred read';
print('after deferred');
"#,
            ),
            (
                "deferred.js",
                "print('deferred evaluated'); export const value = 7; export const then = 3;",
            ),
        ],
        &[
            "before deferred",
            "after symbol-like keys",
            "deferred evaluated",
            "after deferred",
        ],
    );
}

#[test]
fn each_deferred_namespace_internal_operation_evaluates_its_fresh_module() {
    let operations = [
        ("get", "void ns.missing;"),
        (
            "descriptor",
            "Reflect.getOwnPropertyDescriptor(ns, 'missing');",
        ),
        ("has", "Reflect.has(ns, 'missing');"),
        ("delete", "Reflect.deleteProperty(ns, 'missing');"),
        ("define", "Reflect.defineProperty(ns, 'missing', {});"),
        ("ownKeys", "Reflect.ownKeys(ns);"),
        ("ownSymbols", "Object.getOwnPropertySymbols(ns);"),
        ("inheritedGet", "void Object.create(ns).missing;"),
        ("inheritedHas", "void ('missing' in Object.create(ns));"),
        ("receiverSet", "Reflect.set({}, 'value', 1, ns);"),
    ];
    let mut entry = String::new();
    let mut sources = Vec::new();
    let mut expected = Vec::new();
    for (index, (label, _)) in operations.iter().enumerate() {
        entry.push_str(&format!(
            "import defer * as ns{index} from './dep{index}.js';\n"
        ));
        sources.push((
            format!("dep{index}.js"),
            format!("print('evaluated {label}'); export const value = 1;"),
        ));
    }
    for (index, (label, operation)) in operations.iter().enumerate() {
        entry.push_str(&format!("print('before {label}');\n"));
        entry.push_str(&operation.replace("ns", &format!("ns{index}")));
        entry.push_str(&format!("\nprint('after {label}');\n"));
        expected.extend([
            format!("before {label}"),
            format!("evaluated {label}"),
            format!("after {label}"),
        ]);
    }
    let mut files = vec![("entry.js", entry.as_str())];
    files.extend(
        sources
            .iter()
            .map(|(name, source)| (name.as_str(), source.as_str())),
    );
    assert_namespace_modules(
        &files,
        &expected.iter().map(String::as_str).collect::<Vec<_>>(),
    );
}

#[test]
fn deferred_namespace_evaluation_throws_keep_the_original_completion() {
    assert_namespace_modules(
        &[
            (
                "entry.js",
                r#"
import defer * as missing from './missing.js';
import defer * as existing from './existing.js';
import defer * as descriptor from './descriptor.js';
import defer * as presence from './presence.js';
import defer * as keys from './keys.js';
globalThis.namespaceFailure = {message: 'namespace evaluation failure'};
function expectFailure(action) {
  let received;
  try { action(); } catch (error) { received = error; }
  if (received !== globalThis.namespaceFailure) throw 'deferred completion identity';
}
expectFailure(() => missing.absent);
expectFailure(() => existing.value);
expectFailure(() => Reflect.getOwnPropertyDescriptor(descriptor, 'value'));
expectFailure(() => Reflect.has(presence, 'absent'));
expectFailure(() => Reflect.ownKeys(keys));
print('deferred completion identity');
"#,
            ),
            (
                "missing.js",
                "throw globalThis.namespaceFailure; export const value = 1;",
            ),
            (
                "existing.js",
                "throw globalThis.namespaceFailure; export const value = 1;",
            ),
            (
                "descriptor.js",
                "throw globalThis.namespaceFailure; export const value = 1;",
            ),
            (
                "presence.js",
                "throw globalThis.namespaceFailure; export const value = 1;",
            ),
            (
                "keys.js",
                "throw globalThis.namespaceFailure; export const value = 1;",
            ),
        ],
        &["deferred completion identity"],
    );
}

#[test]
fn transparent_namespace_definitions_preserve_compatibility_and_proxy_identity() {
    assert_namespace_modules(
        &[(
            "main.mjs",
            r###"import * as ns from './main.mjs';
export let value = 1;
const proxy = new Proxy(ns, {});
const nested = new Proxy(proxy, {});
let conversions = 0;
const descriptor = {get value() { conversions++; return 1; }};
const compatible = Reflect.defineProperty(proxy, 'value', descriptor);
const returned = Object.defineProperty(nested, 'value', {value: 1}) === nested;
const incompatible = Reflect.defineProperty(nested, 'value', {value: 2});
let rejected = false;
try { Object.defineProperty(proxy, 'value', {value: 2}); } catch (error) { rejected = error instanceof TypeError; }
print([compatible, returned, incompatible, rejected, conversions, ns.value].join('|'));
"###,
        )],
        &["true|true|false|true|1|1"],
    );
}

#[test]
fn transparent_namespace_definitions_read_tdz_bindings() {
    assert_namespace_modules(
        &[(
            "main.mjs",
            r###"import * as ns from './main.mjs';
const proxy = new Proxy(new Proxy(ns, {}), {});
let reflectTDZ = false, objectTDZ = false;
try { Reflect.defineProperty(proxy, 'value', {}); } catch (error) { reflectTDZ = error instanceof ReferenceError; }
try { Object.defineProperty(proxy, 'value', {}); } catch (error) { objectTDZ = error instanceof ReferenceError; }
export let value = 1;
print(reflectTDZ + '|' + objectTDZ);
"###,
        )],
        &["true|true"],
    );
}

#[test]
fn transparent_deferred_namespace_definitions_preserve_evaluation_errors() {
    assert_namespace_modules(
        &[
            (
                "main.mjs",
                r###"import defer * as ns from './dep.mjs';
const sentinel = {};
globalThis.namespaceSentinel = sentinel;
const proxy = new Proxy(ns, {});
let same = false;
try { Reflect.defineProperty(proxy, 'absent', {}); } catch (error) { same = error === sentinel; }
print(same);
"###,
            ),
            (
                "dep.mjs",
                r###"throw globalThis.namespaceSentinel;
export const value = 1;
"###,
            ),
        ],
        &["true"],
    );
}

#[test]
fn namespace_key_arrays_use_the_invoked_builtin_realm() {
    assert_namespace_modules(
        &[(
            "runtime-intrinsic.mjs",
            r###"import * as ns from './runtime-intrinsic.mjs';
export const value = 1;
const foreign = __lilaCreateRealm().global;
const names = foreign.Object.getOwnPropertyNames(ns);
const symbols = foreign.Object.getOwnPropertySymbols(ns);
const all = foreign.Reflect.ownKeys(ns);
print([Object.getPrototypeOf(names) === foreign.Array.prototype,
 Object.getPrototypeOf(symbols) === foreign.Array.prototype,
 Object.getPrototypeOf(all) === foreign.Array.prototype].join('|'));
"###,
        )],
        &["true|true|true"],
    );
}

#[test]
fn namespace_identities_follow_import_phase_through_reexports_and_dynamic_import() {
    assert_namespace_modules(
        &[
            (
                "main.mjs",
                r###"import * as eager from './dep.mjs';
import defer * as deferred from './dep.mjs';
import defer * as repeated from './dep.mjs';
import {forwarded} from './forward.mjs';
const dynamic = await import.defer('./dep.mjs');
print([eager !== deferred, deferred === repeated, deferred === forwarded,
 deferred === dynamic, eager[Symbol.toStringTag], deferred[Symbol.toStringTag],
 eager.value, deferred.value].join('|'));
"###,
            ),
            (
                "dep.mjs",
                r###"export let value = 1;
"###,
            ),
            (
                "forward.mjs",
                r###"import defer * as forwarded from './dep.mjs';
export {forwarded};
"###,
            ),
        ],
        &["true|true|true|true|Module|Deferred Module|1|1"],
    );
}

#[test]
fn namespace_import_reexports_resolve_unambiguously() {
    assert_namespace_modules(
        &[
            (
                "main.mjs",
                r###"export * from './left.mjs';
export * from './right.mjs';
import {forwarded} from './main.mjs';
print(forwarded.value);
"###,
            ),
            (
                "dep.mjs",
                r###"export const value = 7;
"###,
            ),
            (
                "left.mjs",
                r###"import * as forwarded from './dep.mjs';
export {forwarded};
"###,
            ),
            (
                "right.mjs",
                r###"import * as forwarded from './dep.mjs';
export {forwarded};
"###,
            ),
        ],
        &["7"],
    );
}

#[test]
fn deferred_evaluation_rethrows_the_original_completion_for_every_later_access() {
    for thrown in ["undefined", "null", "false", "0", "Symbol('failure')", "{}"] {
        let entry = format!(
            r###"import defer * as ns from './dep.mjs';
globalThis.deferredFailure = {thrown};
globalThis.deferredRuns = 0;
const operations = [
  () => ns.value,
  () => Object.getOwnPropertyDescriptor(ns, 'value'),
  () => Object.getOwnPropertyNames(ns),
  () => Reflect.has(ns, 'value'),
  () => Reflect.ownKeys(ns),
  () => ns.absent
];
for (const operation of operations) {{
  let caught = false;
  try {{ operation(); }} catch (error) {{
    caught = true;
    if (error !== globalThis.deferredFailure) throw 'lost module failure';
  }}
  if (!caught) throw 'lost abrupt completion';
}}
if (globalThis.deferredRuns !== 1) throw 'repeated failed evaluation';
if (ns[Symbol.toStringTag] !== 'Deferred Module' || ns.then !== undefined)
  throw 'non-evaluating namespace reads';
print('original failure retained');
"###,
        );
        assert_namespace_modules(
            &[
                ("entry.mjs", &entry),
                (
                    "dep.mjs",
                    "globalThis.deferredRuns++; export const value = 42; throw globalThis.deferredFailure;",
                ),
            ],
            &["original failure retained"],
        );
    }
}

#[test]
fn reentrant_deferred_evaluation_reuses_readers_without_caching_a_caught_tdz() {
    assert_namespace_modules(
        &[
            (
                "entry.mjs",
                r###"import defer * as ns from './dep.mjs';
globalThis.deferredRuns = 0;
globalThis.inspectDeferred = function () {
  if (Object.getOwnPropertyNames(ns).join('|') !== 'before|later') throw 'reentrant keys';
  if (ns.before() !== 'hoisted') throw 'reentrant function declaration';
  let caught;
  try { ns.later; } catch (error) { caught = error; }
  if (!(caught instanceof ReferenceError)) throw 'reentrant lexical TDZ';
};
if (ns.later !== 42 || ns.later !== 42 || globalThis.deferredRuns !== 1)
  throw 'successful evaluation was not cached';
print('reentrant readers and declaration scope');
"###,
            ),
            (
                "dep.mjs",
                "globalThis.deferredRuns++; globalThis.inspectDeferred(); export function before() { return 'hoisted'; } export const later = 42;",
            ),
        ],
        &["reentrant readers and declaration scope"],
    );
}

#[test]
fn deferred_default_definitions_retain_their_names_sources_and_module_bindings() {
    assert_namespace_modules(
        &[
            (
                "entry.mjs",
                r###"import defer * as callable from './callable.mjs';
import defer * as constructible from './constructible.mjs';
if (callable.default.name !== 'default' || callable.default() !== 42 ||
    callable.default.toString() !== 'function () { return captured; }')
  throw 'deferred anonymous function definition';
if (constructible.default.name !== 'default' || new constructible.default().value !== 43 ||
    constructible.default.toString() !== 'class { constructor() { this.value = captured; } }')
  throw 'deferred anonymous class definition';
print('deferred default definitions');
"###,
            ),
            (
                "callable.mjs",
                "export default function () { return captured; } const captured = 42;",
            ),
            (
                "constructible.mjs",
                "export default class { constructor() { this.value = captured; } } const captured = 43;",
            ),
        ],
        &["deferred default definitions"],
    );
}

#[test]
fn successful_deferred_evaluation_does_not_cache_later_user_getter_errors() {
    assert_namespace_modules(
        &[
            (
                "entry.mjs",
                r###"import defer * as ns from './dep.mjs';
globalThis.deferredRuns = 0;
globalThis.deferredGetterRuns = 0;
const first = {};
const second = {};
for (const marker of [first, second]) {
  globalThis.deferredGetterFailure = marker;
  let caught;
  try { ns.value.failure; } catch (error) { caught = error; }
  if (caught !== marker) throw 'getter completion identity';
}
if (globalThis.deferredRuns !== 1 || globalThis.deferredGetterRuns !== 2)
  throw 'evaluation and getter lifecycles';
if (Object.getOwnPropertyDescriptor(ns, 'value').value !== ns.value)
  throw 'successful module poisoned by getter error';
print('getter failures do not change module evaluation');
"###,
            ),
            (
                "dep.mjs",
                "globalThis.deferredRuns++; export const value = { get failure() { globalThis.deferredGetterRuns++; throw globalThis.deferredGetterFailure; } };",
            ),
        ],
        &["getter failures do not change module evaluation"],
    );
}
