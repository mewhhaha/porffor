use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_with_binding(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
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
        .expect("With HasBinding executes through emitted Wasm");
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn with_has_binding_orders_proxy_protocol_and_preserves_receivers() {
    assert_with_binding(
        r#"
var trace = '';
var receivers = true;
var exclusionsTarget = { selected: false };
var exclusions;
var exclusionsHandler = { get(target, key, receiver) {
  trace += 'b';
  receivers = receivers && this === exclusionsHandler
    && target === exclusionsTarget && receiver === exclusions && key === 'selected';
  return target[key];
} };
exclusions = new Proxy(exclusionsTarget, exclusionsHandler);
var target = { selected: 9, [Symbol.unscopables]: exclusions };
var scope;
var handler = {
  has(object, key) {
    trace += 'h';
    receivers = receivers && this === handler && object === target && key === 'selected';
    return Reflect.has(object, key);
  },
  get(object, key, receiver) {
    trace += key === Symbol.unscopables ? 'u' : 'v';
    receivers = receivers && this === handler && object === target && receiver === scope;
    return Reflect.get(object, key, receiver);
  }
};
scope = new Proxy(target, handler);
function read(scope, selected) { with (scope) { return selected; } }
read(scope, 3) === 9 && trace === 'hubhv' && receivers;
"#,
    );
}

#[test]
fn with_has_binding_short_circuits_absence_and_uses_boolean_conversion_only() {
    assert_with_binding(
        r#"
function read(scope, selected) { with (scope) { return selected; } }
var trace = '';
var absent = new Proxy({}, {
  has(target, key) { trace += 'h'; return false; },
  get() { throw 'unscopables must not be read after absent HasProperty'; }
});
var first = read(absent, 3) === 3 && trace === 'h';
var coercions = 0;
var blocked = { valueOf() { coercions++; throw 'ToBoolean does not coerce'; } };
var selected = read({ selected: 9, [Symbol.unscopables]: { selected: blocked } }, 3);
first && selected === 3 && coercions === 0;
"#,
    );
}

#[test]
fn with_has_binding_accepts_callable_and_htmldda_exclusions_but_ignores_primitives() {
    assert_with_binding(
        r#"
function read(scope, selected) { with (scope) { return selected; } }
function callable() {}
callable.selected = true;
var htmlDda = __lilaCreateHTMLDDA();
htmlDda.selected = true;
var scope = { selected: 9, [Symbol.unscopables]: callable };
var ordinary = read(scope, 3) === 3;
scope[Symbol.unscopables] = htmlDda;
var exotic = read(scope, 3) === 3;
scope[Symbol.unscopables] = null;
var nullValue = read(scope, 3) === 9;
Object.defineProperty(String.prototype, 'selected', {
  configurable: true, get() { throw 'primitive exclusions must not be boxed'; }
});
scope[Symbol.unscopables] = 'excluded';
var primitive = read(scope, 3) === 9;
delete String.prototype.selected;
ordinary && exotic && nullValue && primitive;
"#,
    );
}

#[test]
fn with_has_binding_propagates_every_hook_throw_through_nested_finally() {
    assert_with_binding(
        r#"
function read(scope, selected) { with (scope) { return selected; } }
var marker = Symbol('marker');
var hasThrow = new Proxy({}, { has() { throw marker; } });
var unscopablesThrow = { selected: 9, get [Symbol.unscopables]() { throw marker; } };
var blockedThrow = { selected: 9,
  [Symbol.unscopables]: { get selected() { throw marker; } } };
var caught = 0;
var finalized = 0;
function check(scope) {
  try {
    try { read(scope, 3); }
    finally { finalized++; }
  } catch (error) { if (error === marker) caught++; }
}
check(hasThrow);
check(unscopablesThrow);
check(blockedThrow);
caught === 3 && finalized === 3;
"#,
    );
}

#[test]
fn with_has_binding_keeps_selected_object_across_rhs_and_getter_mutation() {
    assert_with_binding(
        r#"
var trace = '';
var target = { selected: 1, [Symbol.unscopables]: { selected: false } };
var scope = new Proxy(target, {
  has(object, key) { if (key === 'selected') trace += 'h'; return Reflect.has(object, key); },
  get(object, key, receiver) {
    if (key === Symbol.unscopables) trace += 'u';
    return Reflect.get(object, key, receiver);
  },
  set(object, key, value, receiver) {
    if (key === 'selected') trace += 's';
    return Reflect.set(object, key, value, receiver);
  }
});
function rhs() {
  trace += 'r';
  target[Symbol.unscopables].selected = true;
  return 7;
}
function write(scope, selected) { with (scope) { selected = rhs(); } return selected; }
var fallback = write(scope, 3);
var writeOrder = trace === 'hurhs' && fallback === 3 && target.selected === 7;
var exclusionsReads = 0;
var selectedValue = 2;
var updateTarget = {
  get [Symbol.unscopables]() { exclusionsReads++; return { selected: exclusionsReads > 1 }; },
  get selected() { return selectedValue; },
  set selected(value) { selectedValue = value; }
};
function update(scope, selected) { with (scope) { return selected++; } }
var before = update(updateTarget, 3);
writeOrder && before === 2 && selectedValue === 3 && exclusionsReads === 1;
"#,
    );
}

#[test]
fn with_get_binding_value_rechecks_after_unscopables_deletes_the_property() {
    assert_with_binding(
        r#"
var reads = 0;
var target = { selected: 9, get [Symbol.unscopables]() {
  delete this.selected;
  return {};
} };
var scope = new Proxy(target, {
  has(object, key) { if (key === 'selected') reads++; return Reflect.has(object, key); }
});
function read(scope, selected) { with (scope) { return selected; } }
read(scope, 3) === undefined && reads === 2;
"#,
    );
}

#[test]
fn with_has_binding_preserves_completion_and_named_eval_resolution() {
    assert_with_binding(
        r#"
var first = (0, eval)('with ({ selected: 1 }) { 23; var selected = 9; }');
var second = (0, eval)('with ({ selected: 1 }) { 24; var selected; }');
function named(scope, selected) {
  with (scope) { return eval('selected'); }
}
var blocked = { selected: 9, [Symbol.unscopables]: { selected: true } };
var allowed = { selected: 9, [Symbol.unscopables]: { selected: false } };
first === 23 && second === 24 && named(blocked, 3) === 3 && named(allowed, 3) === 9;
"#,
    );
}

#[test]
fn with_has_binding_generated_errors_belong_to_the_executing_realm() {
    assert_with_binding(
        r#"
var realm = __lilaCreateRealm();
realm.evalScript(`
  function read(scope, selected) { with (scope) { return selected; } }
  var count = 0;
  var badHas = new Proxy({ selected: 9 }, { has: 1 });
  var badGet = new Proxy({ selected: 9 }, { get: 1 });
  var badBlockedGet = { selected: 9, [Symbol.unscopables]: new Proxy({}, { get: 1 }) };
  try { read(badHas, 3); } catch (error) { if (error instanceof TypeError) count++; }
  try { read(badGet, 3); } catch (error) { if (error instanceof TypeError) count++; }
  try { read(badBlockedGet, 3); } catch (error) { if (error instanceof TypeError) count++; }
  try { read(new Proxy(badGet, {}), 3); } catch (error) { if (error instanceof TypeError) count++; }
  try { read(Object.create(badGet), 3); } catch (error) { if (error instanceof TypeError) count++; }
  try { with (null) {} } catch (error) { if (error instanceof TypeError) count++; }
  count === 6;
`);
"#,
    );
}

#[test]
fn with_entry_boxes_once_and_rejects_nullish_before_an_empty_body() {
    assert_with_binding(
        r#"
var evaluations = 0;
var caught = 0;
function head() { evaluations++; return null; }
try { with (head()) {} } catch (error) { if (error instanceof TypeError) caught++; }
try { with (undefined) {} } catch (error) { if (error instanceof TypeError) caught++; }
function primitive(value) { with (value) { return valueOf(); } }
var symbol = Symbol('value');
var values = primitive(12) === 12 && primitive(false) === false
  && primitive('abc') === 'abc' && primitive(symbol) === symbol && primitive(14n) === 14n;
Object.defineProperty(Number.prototype, 'bindingObject', {
  configurable: true, get() { return this; }
});
function boxes() { with (12) { return [bindingObject, bindingObject]; } }
var pair = boxes();
delete Number.prototype.bindingObject;
var target = { get bindingObject() { return this; } };
function sameObject(value) { with (value) { return bindingObject; } }
evaluations === 1 && caught === 2 && values && pair[0] === pair[1]
  && pair[0].valueOf() === 12 && sameObject(target) === target;
"#,
    );
}

#[test]
fn captured_with_record_keeps_the_boxed_binding_object() {
    assert_with_binding(
        r#"
var read;
with ('abc') { read = function read() { return length; }; }
read() === 3;
"#,
    );
}

#[test]
fn with_dynamic_array_writes_preserve_setters_and_assignment_results() {
    assert_with_binding(
        r#"
function write(scope, selected, value) { with (scope) { return selected = value; } }
var calls = 0;
var receiver;
var assigned;
Object.defineProperty(Array.prototype, 'selected', {
  configurable: true,
  set(value) { calls++; receiver = this; assigned = value; }
});
var array = [];
var marker = {};
var result = write(array, 1, marker);
delete Array.prototype.selected;
var setter = calls === 1 && receiver === array && assigned === marker
  && result === marker && !Object.prototype.hasOwnProperty.call(array, 'selected');
Object.defineProperty(array, 'selected', { value: 2, writable: false });
var blocked = write(array, 1, 3) === 3 && array.selected === 2;
var callable = function() {};
callable.selected = 4;
setter && blocked && write(callable, 1, 5) === 5 && callable.selected === 5;
"#,
    );
}

#[test]
fn with_dynamic_array_length_writes_convert_twice_and_preserve_abrupt_values() {
    assert_with_binding(
        r#"
function write(scope, length, value) { with (scope) { return length = value; } }
var array = [1, 2, 3];
var conversions = 0;
var value = { valueOf() { conversions++; return 1; } };
var result = write(array, 10, value);
var normal = result === value && conversions === 2 && array.length === 1;
var marker = Symbol('marker');
var caught = false;
try { write(array, 10, { valueOf() { throw marker; } }); }
catch (error) { caught = error === marker; }
normal && caught && array.length === 1;
"#,
    );
}
