use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_wasm_true(source: &str) {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("String construction compiles and executes through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn string_call_and_construct_preserve_primitive_and_exotic_string_results() {
    assert_wasm_true(
        r#"
var boxed = new String('A\uD83D\uDE00\uD800');
var length = Object.getOwnPropertyDescriptor(boxed, 'length');
var first = Object.getOwnPropertyDescriptor(boxed, '0');
String() === '' && String(undefined) === 'undefined' &&
  String(null) === 'null' && String(123n) === '123' &&
  new String().valueOf() === '' && new String(undefined).valueOf() === 'undefined' &&
  Object.getPrototypeOf(boxed) === String.prototype &&
  boxed.valueOf() === 'A\uD83D\uDE00\uD800' && boxed.length === 4 &&
  boxed[1] === '\uD83D' && boxed[2] === '\uDE00' && boxed[3] === '\uD800' &&
  length.value === 4 && !length.writable && !length.enumerable && !length.configurable &&
  first.value === 'A' && !first.writable && first.enumerable && !first.configurable;
"#,
    );
}

#[test]
fn descriptive_symbol_conversion_is_exclusive_to_plain_calls() {
    assert_wasm_true(
        r#"
var symbol = Symbol('value'), failures = 0;
var bound = String.bind(null, symbol);
try { new String(symbol); } catch (error) { if (error instanceof TypeError) failures++; }
try { new bound(); } catch (error) { if (error instanceof TypeError) failures++; }
try { String(Object(symbol)); } catch (error) { if (error instanceof TypeError) failures++; }
try { new String(Object(symbol)); } catch (error) { if (error instanceof TypeError) failures++; }
Symbol.prototype.toString = function () { throw 'primitive hook must not run'; };
String(symbol) === 'Symbol(value)' && bound() === 'Symbol(value)' && failures === 4;
"#,
    );
}

#[test]
fn argument_conversion_precedes_observable_new_target_prototype_lookup() {
    assert_wasm_true(
        r#"
var trace = '', prototype = {};
var target = new Proxy(function () {}, {
  get: function (object, key) {
    if (key === 'prototype') { trace += 'prototype;'; return prototype; }
    return object[key];
  }
});
var input = { [Symbol.toPrimitive]: function (hint) { trace += hint + ';'; return 'text'; } };
var boxed = Reflect.construct(String, [input], target);
var ordered = trace === 'string;prototype;' && Object.getPrototypeOf(boxed) === prototype &&
  String.prototype.valueOf.call(boxed) === 'text';
var failures = 0;
trace = '';
try { Reflect.construct(String, [Symbol('value')], target); }
catch (error) { if (error instanceof TypeError && trace === '') failures++; }
for (var marker of [undefined, {}]) {
  trace = '';
  try {
    Reflect.construct(String, [{ toString: function () { trace += 'convert;'; throw marker; } }], target);
  } catch (error) { if (error === marker && trace === 'convert;') failures++; }
}
ordered && failures === 3;
"#,
    );
}

#[test]
fn conversion_type_errors_belong_to_the_called_string_function_realm() {
    assert_wasm_true(
        r#"
var foreign = __lilaCreateRealm().global;
var symbol = Symbol('value'), failures = 0;
try { new foreign.String(symbol); }
catch (error) { if (Object.getPrototypeOf(error) === foreign.TypeError.prototype) failures++; }
try { foreign.String(Object(symbol)); }
catch (error) { if (Object.getPrototypeOf(error) === foreign.TypeError.prototype) failures++; }
try { new foreign.String({ [Symbol.toPrimitive]: function () { return {}; } }); }
catch (error) { if (Object.getPrototypeOf(error) === foreign.TypeError.prototype) failures++; }
var marker = new TypeError('retained');
try { new foreign.String({ toString: function () { throw marker; } }); }
catch (error) { if (error === marker) failures++; }
failures === 4 && foreign.String(symbol) === 'Symbol(value)';
"#,
    );
}

#[test]
fn fallback_prototype_uses_the_original_new_target_realm_through_bound_and_proxy_targets() {
    assert_wasm_true(
        r#"
var foreign = __lilaCreateRealm().global;
var bound = foreign.Array.bind(null), failures = 0;
for (var prototype of [undefined, null, false, 0, 'prototype', Symbol('prototype')]) {
  Object.defineProperty(bound, 'prototype', {value:prototype, writable:true, configurable:true});
  var first = Reflect.construct(String, ['text'], bound);
  var second = Reflect.construct(String, ['text'], new Proxy(new Proxy(bound, {}), {}));
  if (Object.getPrototypeOf(first) !== foreign.String.prototype ||
      Object.getPrototypeOf(second) !== foreign.String.prototype ||
      String.prototype.valueOf.call(first) !== 'text') failures++;
}
var foreignBound = foreign.String.bind(null, 'bound');
var boxed = new foreignBound();
failures === 0 && Object.getPrototypeOf(boxed) === foreign.String.prototype && boxed.valueOf() === 'bound';
"#,
    );
}

#[test]
fn subclass_and_prototype_getter_errors_preserve_converted_value_and_abrupt_identity() {
    assert_wasm_true(
        r#"
class Text extends String { constructor(value) { super(value); this.extra = 7; } }
var boxed = new Text({ toString: function () { return 'subclass'; } });
var marker, trace = '', caught = false;
var target = new Proxy(function () {}, {
  get: function (object, key) {
    if (key === 'prototype') { trace += 'prototype;'; throw marker; }
    return object[key];
  }
});
try { Reflect.construct(String, [{ toString: function () { trace += 'convert;'; return 'x'; } }], target); }
catch (error) { caught = error === marker; }
boxed instanceof Text && boxed instanceof String && boxed.extra === 7 &&
  boxed.valueOf() === 'subclass' && caught && trace === 'convert;prototype;';
"#,
    );
}

#[test]
fn unchanged_pinned_symbol_conversion_regression_executes() {
    const STA: &str = include_str!("../../../test262/vendor/test262/harness/sta.js");
    const ASSERT: &str = include_str!("../../../test262/vendor/test262/harness/assert.js");
    const CASE: &str =
        include_str!("../../../test262/vendor/test262/test/staging/sm/Symbol/conversions.js");
    assert_wasm_true(&format!("{STA}\n{ASSERT}\n{CASE}\ntrue;"));
}
