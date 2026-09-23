use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

struct Modules(PathBuf);

impl Drop for Modules {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn assert_modules(files: &[(&str, &str)], expected: &[&str], prelude: Option<&str>) {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let fixture = Modules(std::env::temp_dir().join(format!(
        "lila-module-instantiation-{}-{}",
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
                module_prelude: prelude.map(str::to_owned),
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("module graph compiles and executes through Wasm");
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
fn script_prelude_hosts_are_available_to_module_completion_jobs() {
    assert_modules(
        &[(
            "entry.js",
            r#"
const original = finish;
globalThis.finish = function(value) { original(value); };
Promise.resolve(7).then(finish);
"#,
        )],
        &["finished:7"],
        Some("function finish(value) { print('finished:' + value); }"),
    );
}

#[test]
fn nested_import_readers_keep_live_values_after_reassignment() {
    assert_modules(
        &[
            ("entry.js", r#"
import defer * as namespace from './value.js';
import { value, replace } from './value.js';
const readers = [() => namespace.value.property, () => value.property];
for (const read of readers) if (read() !== 3) throw 'initial imported object';
replace({ property: 7 });
for (const read of readers) if (read() !== 7) throw 'stale imported object';
print('live captured imports');
"#),
            ("value.js", "export let value = { property: 3 }; export function replace(next) { value = next; }"),
        ],
        &["live captured imports"],
        None,
    );
}

#[test]
fn deferred_dependencies_wait_for_first_get_and_module_names_stay_separate() {
    assert_modules(
        &[
            ("entry.js", r#"
import './setup.js';
import defer * as parent from './parent.js';
if (globalThis.evaluations.length !== 0) throw 'evaluated while linking';
const nested = parent.nested;
if (globalThis.evaluations.join(',') !== 'dependency,parent') throw 'dependency order';
if (parent.value !== 4 || parent.nested !== nested) throw 'parent was evaluated again';
if (nested.value !== 9 || globalThis.evaluations.join(',') !== 'dependency,parent,nested') throw 'deferred child';
if (globalThis.localName !== undefined) throw 'module var escaped';
print('deferred dependency order');
"#),
            ("setup.js", "globalThis.evaluations = [];"),
            ("parent.js", r#"
import { value as dependency } from './dependency.js';
import defer * as nested from './nested.js';
export { nested };
var localName = 'parent';
globalThis.evaluations.push(localName);
export const value = dependency + 1;
"#),
            ("dependency.js", "var localName = 'dependency'; globalThis.evaluations.push(localName); export let value = 3;"),
            ("nested.js", "var localName = 'nested'; globalThis.evaluations.push(localName); export let value = 9;"),
        ],
        &["deferred dependency order"],
        None,
    );
}

#[test]
fn dynamic_deferred_import_preserves_promise_order_namespace_identity_and_rejections() {
    assert_modules(
        &[
            ("entry.js", r#"
import './setup.js';
import defer * as first from './value.js';
const sentinel = {};
const bad = { toString() { throw sentinel; } };
const rejected = import.defer(bad);
const imported = import.defer('./value.js');
if (!(imported instanceof Promise) || !(rejected instanceof Promise)) throw 'import promise';
print('before jobs');
imported.then(ns => {
  if (ns !== first || globalThis.evaluations.length !== 0) throw 'import evaluated a synchronous target';
  if (ns.value !== 6 || globalThis.evaluations.join(',') !== 'dependency,value') throw 'first get';
  return import.defer('./value.js');
}).then(ns => {
  if (ns !== first || globalThis.evaluations.length !== 2) throw 'repeated import';
  print('same deferred namespace');
});
rejected.then(() => { throw 'coercion fulfilled'; }, error => {
  if (error !== sentinel) throw 'coercion identity';
  print('same coercion error');
});
"#),
            ("setup.js", "globalThis.evaluations = [];"),
            ("value.js", "import { value as dependency } from './dependency.js'; globalThis.evaluations.push('value'); export const value = dependency + 1;"),
            ("dependency.js", "globalThis.evaluations.push('dependency'); export const value = 5;"),
        ],
        &["before jobs", "same coercion error", "same deferred namespace"],
        None,
    );
}

#[test]
fn current_and_transitive_evaluation_reject_reentry_before_running_a_body() {
    assert_modules(
        &[
            ("entry.js", r#"
import defer * as self from './entry.js';
import defer * as dependent from './dependent.js';
globalThis.dependentCalls = 0;
for (const read of [() => self.absent, () => dependent.value]) {
  let received;
  try { read(); } catch (error) { received = error; }
  if (!(received instanceof TypeError)) throw 'evaluating graph accepted';
}
if (globalThis.dependentCalls !== 0) throw 'dependent body ran';
export const value = 1;
print('evaluating readiness');
"#),
            ("dependent.js", "import { value as parent } from './entry.js'; globalThis.dependentCalls++; export const value = parent + 1;"),
        ],
        &["evaluating readiness"],
        None,
    );
}

#[test]
fn readiness_stops_at_an_evaluated_dependency_even_when_its_importer_is_evaluating() {
    assert_modules(
        &[
            ("entry.js", r#"
import { value } from './evaluated.js';
import defer * as waiting from './waiting.js';
if (value !== 3 || waiting.value !== 4) throw 'evaluated dependency was traversed';
print('evaluated readiness');
"#),
            ("evaluated.js", "import defer * as entry from './entry.js'; export const value = 3;"),
            ("waiting.js", "import { value as previous } from './evaluated.js'; export const value = previous + 1;"),
        ],
        &["evaluated readiness"],
        None,
    );
}

#[test]
fn deferred_readiness_cycles_use_a_seen_set_and_evaluate_each_body_once() {
    assert_modules(
        &[
            ("entry.js", r#"
import defer * as left from './left.js';
import defer * as right from './right.js';
globalThis.calls = [];
if (left.value !== 2 || right.value !== 3 || left.value !== 2) throw 'deferred cycle values';
if (globalThis.calls.join(',') !== 'left,right') throw 'deferred cycle order';
print('deferred readiness cycle');
"#),
            ("left.js", "import defer * as right from './right.js'; globalThis.calls.push('left'); export const value = 2;"),
            ("right.js", "import defer * as left from './left.js'; globalThis.calls.push('right'); export const value = left.value + 1;"),
        ],
        &["deferred readiness cycle"],
        None,
    );
}

#[test]
fn dependency_errors_are_cached_without_losing_the_original_thrown_value() {
    assert_modules(
        &[
            ("entry.js", r#"
import defer * as parent from './parent.js';
import defer * as dependency from './dependency.js';
globalThis.calls = [];
globalThis.moduleError = {};
for (const read of [() => parent.value, () => parent.value, () => dependency.value]) {
  let received;
  try { read(); } catch (error) { received = error; }
  if (received !== globalThis.moduleError) throw 'module error identity';
}
if (globalThis.calls.join(',') !== 'dependency') throw 'errored graph re-executed';
print('cached dependency error');
"#),
            ("parent.js", "import { value as original } from './dependency.js'; globalThis.calls.push('parent'); export const value = original;"),
            ("dependency.js", "globalThis.calls.push('dependency'); throw globalThis.moduleError; export const value = 1;"),
        ],
        &["cached dependency error"],
        None,
    );
}

#[test]
fn indirect_import_cells_are_live_readonly_and_shared_with_direct_eval() {
    assert_modules(
        &[
            (
                "entry.js",
                r#"
import defer * as reader from './reader.js';
if (reader.read() !== 1) throw 'initial imported value';
reader.replace(7);
if (reader.read() !== 7 || reader.reexport !== 7) throw 'copied import';
if (!reader.rejectWrite() || reader.read() !== 7) throw 'writable import';
print('live import and eval');
"#,
            ),
            (
                "reader.js",
                r#"
import { value, replace } from './value.js';
export { value as reexport, replace };
export function read() { return eval('value'); }
export function rejectWrite() {
  let received;
  try { eval('value = 19'); } catch (error) { received = error; }
  return received instanceof TypeError;
}
"#,
            ),
            (
                "value.js",
                "export let value = 1; export function replace(next) { value = next; }",
            ),
        ],
        &["live import and eval"],
        None,
    );
}

#[test]
fn private_activations_preserve_module_lexical_rules_and_all_line_terminators() {
    assert_modules(
        &[
            ("entry.js", r#"
import defer * as value from './value.js';
globalThis.arguments = 'global argument binding';
if (value.rootThis !== undefined || value.arrowThis !== undefined ||
    value.args !== 'global argument binding') throw 'module lexical binding';
if (value.default.name !== 'default' || value.default() !== 8) throw 'default name';
if (value.readBeforeInit !== true || value.destructured !== 4 || value.varValue !== 6) throw 'module initialization';
if (globalThis.moduleVar !== undefined) throw 'module variable leaked';
print('private module lexical rules');
"#),
            ("value.js", "// 🦀\r\nexport const rootThis = this;\rexport const arrowThis = (() => this)();\u{2028}export const args = arguments;\u{2029}export default function () { return 8; }\nexport const readBeforeInit = (() => { try { return later; } catch (error) { return error instanceof ReferenceError; } })();\nlet later = 2;\nexport const { value: destructured } = { value: 4 };\nvar moduleVar = 6; export { moduleVar as varValue };"),
        ],
        &["private module lexical rules"],
        None,
    );
}

#[test]
fn global_script_prelude_keeps_its_scope_strictness_and_live_globals() {
    assert_modules(
        &[
            (
                "entry.js",
                r#"
import defer * as dependency from './dependency.js';
const sameName = 'entry';
if (this !== undefined || (() => this)() !== undefined) throw 'module this';
if (helperThis() !== globalThis || helperRead() !== 'global') throw 'Script scope';
if (globalThis.helperValue !== 7 || globalThis.sameName !== undefined) throw 'global bindings';
if (dependency.value !== 11 || sameName !== 'entry') throw 'dependency scope';
if (parseInt('1') !== 42) throw 'stale intrinsic';
print('independent global Script');
"#,
            ),
            (
                "dependency.js",
                r#"
const sameName = 'dependency';
if (helperThis() !== globalThis || helperRead() !== 'global') throw 'dependency helper scope';
if (this !== undefined || globalThis.sameName !== undefined) throw 'module isolation';
export const value = helperValue + lexicalValue;
"#,
            ),
        ],
        &["prelude", "independent global Script"],
        Some(
            r#"
var helperValue = 7;
let lexicalValue = 4;
const sameName = 'global';
function helperThis() { return this; }
function helperRead() { return sameName; }
parseInt = function() { return 42; };
print('prelude');
"#,
        ),
    );
}

#[test]
fn throwing_global_script_prevents_module_evaluation() {
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_module(
            "print('module body must not run');",
            CompileOptions {
                module_prelude: Some("print('prelude throw'); throw 7;".into()),
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("prelude abrupt completion remains a language throw");
    assert!(matches!(observed.completion, ObservedCompletion::Throw(_)));
    assert_eq!(
        observed.output_events,
        [HostOutputEvent::PrintLine("prelude throw".into())]
    );
}

#[test]
fn ordinary_module_shadows_global_script_lexicals_without_redeclaring_them() {
    assert_modules(
        &[
            (
                "entry.js",
                r#"
import { dependencyValue } from './dependency.js';
const shared = 'entry';
var moduleVar = 3;
function moduleFunction() { return shared; }
if (helperRead() !== 'global' || dependencyValue !== 'global') throw 'Script closure scope';
if (shared !== 'entry' || moduleFunction() !== 'entry') throw 'Module lexical scope';
if (globalThis.moduleVar !== undefined || globalThis.moduleFunction !== undefined) throw 'Module declaration leaked';
if (arguments !== 17 || (() => arguments)() !== 17) throw 'implicit arguments';
if (this !== undefined || (() => this)() !== undefined) throw 'Module this';
print('ordinary Module isolation');
"#,
            ),
            (
                "dependency.js",
                "export const dependencyValue = helperRead();",
            ),
        ],
        &["ordinary Module isolation"],
        Some(
            "const shared = 'global'; var arguments = 17; function helperRead() { return shared; }",
        ),
    );
}

#[test]
fn retained_evaluation_cycle_keeps_module_declarations_out_of_global_script() {
    assert_modules(
        &[
            (
                "entry.js",
                r#"
import { cycleValue } from './cycle.js';
const shared = 'entry';
var moduleVar = 3;
if (cycleValue !== 'global' || helperRead() !== 'global' || shared !== 'entry') throw 'cycle closure scope';
if (globalThis.moduleVar !== undefined || globalThis.cycleVar !== undefined) throw 'cycle declaration leaked';
if (arguments !== 17 || (() => arguments)() !== 17) throw 'cycle arguments';
if (this !== undefined || (() => this)() !== undefined) throw 'cycle this';
print('cycle Module isolation');
"#,
            ),
            (
                "cycle.js",
                r#"
import './entry.js';
var cycleVar = 5;
if (arguments !== 17 || this !== undefined) throw 'dependency lexical context';
export const cycleValue = helperRead();
"#,
            ),
        ],
        &["cycle Module isolation"],
        Some(
            "const shared = 'global'; var arguments = 17; function helperRead() { return shared; }",
        ),
    );
}

#[test]
fn retained_top_level_await_preserves_global_arguments_and_script_closures() {
    assert_modules(
        &[(
            "entry.js",
            r#"
if (arguments !== 17 || this !== undefined) throw 'before await lexical context';
const readArguments = () => arguments;
await 0;
const shared = 'entry';
var moduleVar = 3;
if (helperRead() !== 'global' || shared !== 'entry') throw 'async Script closure scope';
if (globalThis.moduleVar !== undefined) throw 'async Module declaration leaked';
if (arguments !== 17 || readArguments() !== 17) throw 'after await arguments';
if (this !== undefined || (() => this)() !== undefined) throw 'after await this';
print('async Module isolation');
"#,
        )],
        &["async Module isolation"],
        Some(
            "const shared = 'global'; var arguments = 17; function helperRead() { return shared; }",
        ),
    );
}

#[test]
fn retained_source_phase_graph_does_not_expose_its_bindings_to_global_script() {
    assert_modules(
        &[
            (
                "entry.js",
                r#"
import source source from './source.js';
const shared = 'entry';
var moduleVar = 3;
if (helperRead() !== 'global' || shared !== 'entry') throw 'source-phase Script closure scope';
if (globalThis.moduleVar !== undefined || arguments !== 17) throw 'source-phase lexical context';
if (this !== undefined) throw 'source-phase this';
print('source-phase Module isolation');
"#,
            ),
            (
                "source.js",
                "throw 'source-only module must not evaluate'; export const unused = 1;",
            ),
        ],
        &["source-phase Module isolation"],
        Some(
            "const shared = 'global'; var arguments = 17; function helperRead() { return shared; }",
        ),
    );
}

#[test]
fn repeated_synchronous_module_catches_preserve_iterations_and_following_statements() {
    assert_modules(
        &[(
            "entry.js",
            r#"
const events = [];
for (let iteration = 0; iteration < 3; iteration++) {
  try { throw iteration; } catch (caught) { events.push(caught); }
  events.push('iteration');
}
print(events.join(','));
print('after catch loop');
"#,
        )],
        &["0,iteration,1,iteration,2,iteration", "after catch loop"],
        None,
    );
}

#[test]
fn module_finally_scopes_preserve_continue_break_and_normal_completion() {
    assert_modules(
        &[
            (
                "entry.js",
                r#"
import defer * as dependency from './dependency.js';
const events = [];
for (let iteration = 0; iteration < 3; iteration++) {
  try {
    try {
      events.push('try' + iteration);
      if (iteration === 0) continue;
      if (iteration === 2) break;
      throw iteration;
    } catch (error) {
      events.push('catch' + error);
    } finally {
      events.push('finally' + iteration);
    }
  } finally {
    events.push('outer' + iteration);
  }
  events.push('tail' + iteration);
}
print(events.join(','));
print('after finally loop');
"#,
            ),
            ("dependency.js", "export const value = 1;"),
        ],
        &[
            "try0,finally0,outer0,try1,catch1,finally1,outer1,tail1,try2,finally2,outer2",
            "after finally loop",
        ],
        None,
    );
}

#[test]
fn module_catch_parameter_and_body_closures_keep_each_iteration_environment() {
    assert_modules(
        &[(
            "entry.js",
            r#"
const reads = [];
for (const marker of [3, 7]) {
  try { throw { value: marker }; }
  catch ({ value, read = () => value }) {
    const fromBody = () => value;
    reads.push(read, fromBody);
    value++;
  }
}
print(reads[0](), reads[1](), reads[2](), reads[3]());
print('after captured catch loop');
"#,
        )],
        &["4 4 8 8", "after captured catch loop"],
        None,
    );
}

#[test]
fn synchronous_module_resources_dispose_each_iteration_after_its_catch() {
    assert_modules(
        &[
            (
                "entry.js",
                r#"
import defer * as dependency from './dependency.js';
const events = [];
for (const value of [1, 2]) {
  using resource = { [Symbol.dispose]() { events.push('dispose' + value); } };
  try { throw value; } catch (error) { events.push('catch' + error); }
}
print(events.join(','));
print('after resource loop');
"#,
            ),
            ("dependency.js", "export const value = 1;"),
        ],
        &["catch1,dispose1,catch2,dispose2", "after resource loop"],
        None,
    );
}

#[test]
fn deferred_module_resources_are_acquired_only_during_evaluation() {
    assert_modules(
        &[
            (
                "entry.js",
                r#"
import defer * as resource from './resource.js';
globalThis.events = [];
globalThis.events.push('before');
const value = resource.value;
globalThis.events.push('after' + value);
print(globalThis.events.join(','));
"#,
            ),
            (
                "resource.js",
                r#"
using resource = (globalThis.events.push('acquire'), {
  [Symbol.dispose]() { globalThis.events.push('dispose'); }
});
globalThis.events.push('body');
export const value = 3;
"#,
            ),
        ],
        &["before,acquire,body,dispose,after3"],
        None,
    );
}

#[test]
fn module_classic_for_resources_keep_continue_break_and_disposal_order() {
    assert_modules(
        &[(
            "entry.js",
            r#"
const events = [];
function acquire(name) {
  events.push('acquire' + name);
  return { [Symbol.dispose]() { events.push('dispose' + name); } };
}
let index = 0;
for (using resource = acquire('loop'); index < 3; index++) {
  events.push('body' + index);
  if (index === 0) continue;
  break;
}
for (using resource = acquire('empty'); false;) { throw 'unreachable body'; }
events.push('after');
print(events.join(','));
"#,
        )],
        &["acquireloop,body0,body1,disposeloop,acquireempty,disposeempty,after"],
        None,
    );
}

#[test]
fn module_for_of_resources_dispose_before_iterator_close_and_propagate_errors() {
    assert_modules(
        &[(
            "entry.js",
            r#"
const events = [];
const marker = {};
function resource(name, fail) {
  return { [Symbol.dispose]() { events.push('dispose' + name); if (fail) throw marker; } };
}
for (using value of [resource('first', false), resource('second', false)]) {
  events.push('body');
}
const iterable = {
  [Symbol.iterator]() {
    return {
      next() { return {value:resource('throwing', true), done:false}; },
      return() { events.push('close'); return {done:true}; }
    };
  }
};
let caught = false;
try {
  for (using value of iterable) { events.push('last body'); break; }
} catch (error) { caught = error === marker; }
if (!caught) throw 'disposal error identity';
events.push('after');
print(events.join(','));
"#,
        )],
        &["body,disposefirst,body,disposesecond,last body,disposethrowing,close,after"],
        None,
    );
}

#[test]
fn nested_module_resources_preserve_reverse_disposal_and_suppressed_errors() {
    assert_modules(
        &[(
            "entry.js",
            r#"
const events = [];
const bodyError = {};
const disposalError = {};
function nested() {
  using first = { [Symbol.dispose]() { events.push('first'); } };
  using second = { [Symbol.dispose]() { events.push('second'); throw disposalError; } };
  events.push('body');
  throw bodyError;
}
let caught = false;
try { nested(); } catch (error) {
  caught = error instanceof SuppressedError && error.error === disposalError && error.suppressed === bodyError;
}
if (!caught) throw 'suppressed disposal error';
{ using resource = { [Symbol.dispose]() { events.push('block'); } }; }
events.push('after');
print(events.join(','));
"#,
        )],
        &["body,second,first,block,after"],
        None,
    );
}
