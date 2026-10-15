use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

// These are single-program entry-ABI regressions. They do not replace the
// required retained two-program conflict and same-Store host compiler controls.
#[test]
fn program_entry_operands_preserve_native_proxy_bound_construct_and_tail_forwarding() {
    assert_modes(
        r#"
let trace = '';
const captured = 'source-owner';
function read(value) { trace += 'R'; return captured + ':' + value; }
if (read.call(null, 17) !== 'source-owner:17') throw 'native call';
if (Reflect.apply(read, null, [19]) !== 'source-owner:19') throw 'native apply';
const bound = read.bind(null, 23);
const proxy = new Proxy(bound, {
  apply(target, receiver, args) { trace += 'P'; return Reflect.apply(target, receiver, args); }
});
function tail(target) { 'use strict'; return target(); }
if (tail(proxy) !== 'source-owner:23') throw 'bound proxy tail';
function Constructor(value) { this.text = captured + ':' + value; this.target = new.target; }
function NewTarget() {}
const constructed = Reflect.construct(Constructor.bind(null, 29), [], NewTarget);
if (constructed.text !== 'source-owner:29' || constructed.target !== NewTarget ||
    Object.getPrototypeOf(constructed) !== NewTarget.prototype) throw 'construct operands';
const proxiedConstructor = new Proxy(Constructor, {
  construct(target, args, newTarget) { trace += 'C'; return Reflect.construct(target, args, newTarget); }
});
if (new proxiedConstructor(31).text !== 'source-owner:31') throw 'proxy construct';
const empty = Function();
if (empty() !== undefined || typeof new empty() !== 'object') throw 'R-generated empty source';
const mapped = [37].map(value => read(value));
if (mapped[0] !== 'source-owner:37' || trace !== 'RRPRCR') throw 'native callback';
$262.gc();
if (proxy() !== 'source-owner:23') throw 'retained source after GC';
print('program-entry-forwarding:ok');
262;
"#,
        "program-entry-forwarding:ok",
    );
}

#[test]
fn program_entry_operands_preserve_retained_generator_and_async_activations() {
    assert_modes(
        r#"
const captured = 'resumed-source-owner';
function* generator(value) {
  yield captured + ':' + value;
  return () => captured + ':returned';
}
const iterator = generator(41);
$262.gc();
if (iterator.next().value !== 'resumed-source-owner:41') throw 'generator entry';
$262.gc();
const returned = iterator.next();
if (!returned.done || returned.value() !== 'resumed-source-owner:returned') throw 'generator owner';
async function asynchronous(value) {
  await Promise.resolve(0);
  $262.gc();
  return captured + ':' + value;
}
async function* asyncGenerator(value) {
  await Promise.resolve(0);
  yield captured + ':' + value;
  return captured + ':done';
}
const asynchronousIterator = asyncGenerator(47);
Promise.all([asynchronous(43), asynchronousIterator.next()]).then(values => {
  if (values[0] !== 'resumed-source-owner:43' || values[1].value !== 'resumed-source-owner:47' ||
      values[1].done) throw 'async entry operands';
  return asynchronousIterator.next();
}).then(value => {
  if (!value.done || value.value !== 'resumed-source-owner:done') throw 'async retained owner';
  print('program-entry-resumption:ok');
});
262;
"#,
        "program-entry-resumption:ok",
    );
}

fn assert_modes(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("callable entry roles execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(expected.into())],
            "{source}"
        );
    }
}

#[test]
fn new_target_is_separate_from_resumable_activations_and_lexical_captures() {
    assert_modes(
        include_str!("../fixtures/callable_entry_roles/new_target_and_resumption.js"),
        "callable-entry-roles:ok",
    );
}

#[test]
fn prepared_realm_scripts_publish_the_right_callable_family_and_context() {
    assert_modes(
        include_str!("../fixtures/callable_entry_roles/prepared_realm_entries.js"),
        "prepared-callable-entry-roles:ok",
    );
}

fn assert_sloppy_arguments(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("Annex B arguments binding executes through real Wasm");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn annex_b_arguments_copy_updates_the_existing_binding_only_when_selected() {
    assert_sloppy_arguments(
        r#"
function selected(value) {
  var before = arguments;
  var read = () => arguments;
  if (typeof before !== 'object' || before[0] !== 17) throw 'arguments missing at entry';
  if (false) { function arguments() {} }
  if (read() !== before) throw 'unselected declaration replaced arguments';
  { function arguments() { return 19; } }
  return typeof arguments === 'function' && read() === arguments && arguments() === 19;
}
function parameter(arguments) {
  { function arguments() {} }
  return arguments === 23;
}
function lexical() {
  let arguments = 29;
  { function arguments() {} }
  return arguments === 29;
}
selected(17) && parameter(23) && lexical();
"#,
    );
}

#[test]
fn annex_b_arguments_late_body_binding_preserves_parameter_and_arrow_environments() {
    assert_sloppy_arguments(
        r#"
function defaults(read = () => arguments, alter = () => arguments = 31) {
  let before = read();
  let later = () => arguments;
  if (later() !== before || typeof before !== 'object') throw 'body binding created at entry';
  { function arguments() { return 37; } }
  let body = arguments;
  return later() === body && read() === before && body() === 37
    && alter() === 31 && read() === 31 && later() === body;
}
function enclosing(value) {
  let before = arguments;
  let arrow = () => {
    let read = () => arguments;
    if (read() !== before) throw 'arrow created own arguments at entry';
    { function arguments() { return 41; } }
    return read() === arguments && arguments() === 41;
  };
  return arrow() && arguments === before && before[0] === value;
}
defaults() && enclosing(43);
"#,
    );
}

fn assert_labelled_lexical_depth(source: &str) {
    const STA: &str = include_str!("../../../../test262/vendor/test262/harness/sta.js");
    const ASSERT: &str = include_str!("../../../../test262/vendor/test262/harness/assert.js");
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    // Host surface selection must not select a labelled-declaration dialect.
    for host_surface_policy in [HostSurfacePolicy::Product, HostSurfacePolicy::Test262] {
        let source = format!("{STA}\n{ASSERT}\n{source}\ntrue;");
        let outcome = Engine::new(RealmBuilder::new().build())
            .run_script(
                &source,
                CompileOptions {
                    host_surface_policy,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(60_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("labelled lexical depth failed: {error}\n{source}"));
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }
}

#[test]
fn labelled_single_lexical_items_and_top_level_chains_keep_hoisted_functions() {
    assert_labelled_lexical_depth(
        r#"
assert.sameValue(root(), 17);
top: next: function root() { return 17; }
function owner() {
  assert.sameValue(inner(), 19);
  first: second: function inner() { return 19; }
  {
    assert.sameValue(single(), 23);
    label: function single() { return 23; }
    assert.sameValue((() => single)(), single);
  }
  switch (0) {
    case 0:
      assert.sameValue(caseSingle(), 29);
      break;
    default:
      label: function caseSingle() { return 29; }
  }
}
owner();
"#,
    );
}

#[test]
fn labelled_chains_do_not_create_block_or_case_bindings() {
    assert_labelled_lexical_depth(
        r#"
{
  assert.sameValue(typeof missingBlock, 'undefined');
  assert.throws(ReferenceError, () => missingBlock);
  outer: inner: function missingBlock() { return 31; }
  assert.throws(ReferenceError, () => missingBlock);
}
switch (0) {
  case 0:
    assert.sameValue(typeof missingCase, 'undefined');
    assert.throws(ReferenceError, () => missingCase);
    outer: inner: function missingCase() { return 37; }
    assert.throws(ReferenceError, () => missingCase);
    break;
}
function outerBindings() {
  let existing = 41;
  {
    assert.sameValue(existing, 41);
    a: b: c: function existing() { return 43; }
    assert.sameValue(existing, 41);
  }
  switch (0) {
    case 0:
      assert.sameValue(existing, 41);
      a: b: function existing() { return 47; }
      assert.sameValue(existing, 41);
  }
  return existing;
}
assert.sameValue(outerBindings(), 41);
{
  assert.sameValue(direct(), 53);
  function direct() { return 53; }
  a: b: function direct() { return 59; }
  assert.sameValue(direct(), 53);
}
with ({ visible: 61 }) {
  a: b: function visible() { return 67; }
  assert.sameValue(visible, 61);
}
try { throw 71; } catch (caught) {
  a: b: function caught() { return 73; }
  assert.sameValue(caught, 71);
}
"#,
    );
}

#[test]
fn labelled_chains_do_not_invent_closure_aliases_or_finite_eval_bindings() {
    assert_labelled_lexical_depth(
        r#"
var readMissing;
{
  readMissing = () => absent;
  a: b: function absent() { return 79; }
}
assert.throws(ReferenceError, readMissing);
eval("{ assert.throws(ReferenceError, () => evalMissing); a: b: function evalMissing() {} assert.throws(ReferenceError, () => evalMissing); }");
function capturedOuter() {
  let visible = 83;
  let read;
  {
    read = () => visible;
    assert.sameValue(eval('visible'), 83);
    a: b: function visible() { return 89; }
    assert.sameValue(eval('visible'), 83);
  }
  assert.sameValue(read(), 83);
  switch (0) {
    case 0:
      assert.sameValue(eval('visible'), 83);
      a: b: function visible() { return 97; }
      assert.sameValue(eval('visible'), 83);
  }
  return read();
}
assert.sameValue(capturedOuter(), 83);
"#,
    );
}

#[test]
fn unchanged_pinned_labelled_function_syntax_keeps_both_top_level_names() {
    const PINNED: &str = include_str!(
        "../../../../test262/vendor/test262/test/annexB/language/statements/labeled/function-declaration.js"
    );
    let source = format!(
        "{PINNED}\nassert.sameValue(typeof g, 'function');\nassert.sameValue(typeof f, 'function');"
    );
    assert_labelled_lexical_depth(&source);
}

#[test]
fn labelled_nonlexical_chains_do_not_shadow_initialized_bindings_in_direct_eval() {
    assert_labelled_lexical_depth(
        r#"
function owner() {
  let visible = 83;
  {
    assert.sameValue(eval('visible'), 83);
    a: b: function visible() { return 89; }
    assert.sameValue(eval('visible'), 83);
  }
  switch (0) {
    case 0:
      assert.sameValue(eval('visible'), 83);
      a: b: function visible() { return 97; }
      assert.sameValue(eval('visible'), 83);
  }
  return visible;
}
assert.sameValue(owner(), 83);
"#,
    );
}
