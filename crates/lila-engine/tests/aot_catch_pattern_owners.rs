use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_catch_pattern(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("catch pattern failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        outcome.completion
    );
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine("ok".to_string())],
        "{source}"
    );
}

#[test]
fn catch_object_class_name() {
    assert_catch_pattern(
        r#"
var result;
try { throw {}; }
catch ({ cls = class {}, named = class Inner { static self() { return Inner; } } }) {
  result = cls.name === 'cls' && named.name === 'Inner' && named.self() === named;
}
if (result !== true) throw 'catch object class names';
print('ok');
"#,
    );
}

#[test]
fn catch_array_class_name() {
    assert_catch_pattern(
        r#"
var result;
try { throw []; }
catch ([cls = class {}, named = class Inner { static self() { return Inner; } }]) {
  result = cls.name === 'cls' && named.name === 'Inner' && named.self() === named;
}
if (result !== true) throw 'catch array class names';
print('ok');
"#,
    );
}

#[test]
fn catch_parameter_and_body_scopes() {
    assert_catch_pattern(
        r#"
var parameterRead, bodyRead;
let value = 'outside';
try { throw []; }
catch ([unused = parameterRead = function () { return value; }]) {
  bodyRead = function () { return value; };
  let value = 'inside';
}
if (parameterRead() !== 'outside' || bodyRead() !== 'inside') throw 'catch environment capture';
print('ok');
"#,
    );
}

#[test]
fn catch_captured_earlier_and_later_bindings() {
    assert_catch_pattern(
        r#"
var readers;
try { throw {}; }
catch ({ first = 2, readFirst = () => first, readLast = function () { return last; }, last = 3 }) {
  readers = [readFirst, readLast];
  first = 4;
  last = 5;
}
if (readers[0]() !== 4 || readers[1]() !== 5) throw 'catch binding lifetime';
print('ok');
"#,
    );
}

#[test]
fn catch_computed_key_and_nested_patterns() {
    assert_catch_pattern(
        r#"
var keyLog = '', stored;
try { throw {value: {nested: []}}; }
catch ({[(() => { keyLog += 'key'; return 'value'; })()]: {nested: [fn = function* () { yield 7; }]}}) {
  stored = fn;
}
if (keyLog !== 'key' || stored.name !== 'fn' || stored().next().value !== 7) throw 'catch nested owner';
print('ok');
"#,
    );
}

#[test]
fn catch_initializer_tdz_and_abrupt() {
    assert_catch_pattern(
        r#"
var tdz = false, marker = {}, caught, reached = false;
try {
  try { throw {}; }
  catch ({ read = (() => later)(), later = 1 }) { reached = true; }
} catch (error) { tdz = error instanceof ReferenceError; }
try {
  try { throw []; }
  catch ([unused = (() => { throw marker; })()]) { reached = true; }
} catch (error) { caught = error; }
if (!tdz || caught !== marker || reached) throw 'catch abrupt initialization';
print('ok');
"#,
    );
}

#[test]
fn catch_class_method_retains_parameter() {
    assert_catch_pattern(
        r#"
var ClassValue;
try { throw [9]; }
catch ([value, cls = class Inner { static read() { return value; } static self() { return Inner; } }]) {
  ClassValue = cls;
  value = 10;
}
if (ClassValue.read() !== 10 || ClassValue.self() !== ClassValue) throw 'catch class captures';
print('ok');
"#,
    );
}

#[test]
fn catch_generator_body_suspension() {
    assert_catch_pattern(
        r#"
function* values() {
  let outside = 9;
  try { throw [2]; }
  catch ([value, read = () => value, readOutside = () => outside]) {
    let outside = 100;
    yield read() + readOutside();
    value = 4;
    yield read() + readOutside();
  }
}
var iterator = values();
if (iterator.next().value !== 11 || iterator.next().value !== 13 || !iterator.next().done)
  throw 'catch suspension scope';
print('ok');
"#,
    );
}

#[test]
fn catch_async_body_suspension() {
    assert_catch_pattern(
        r#"
async function valueAfterAwait() {
  let outside = 9;
  try { throw [2]; }
  catch ([value, read = () => value, readOutside = () => outside]) {
    let outside = 100;
    await Promise.resolve();
    value = 4;
    return read() + readOutside();
  }
}
valueAfterAwait().then(value => {
  if (value !== 13) throw 'catch async scope';
  print('ok');
}, error => { print('rejected:' + error); });
void 0;
"#,
    );
}

#[test]
fn catch_provided_value_and_simple_parameter() {
    assert_catch_pattern(
        r#"
var calls = 0, supplied = function () { return 5; }, saved;
try { throw {read: supplied}; }
catch ({read = (() => { calls += 1; return function () { return 6; }; })()}) { saved = read; }
if (calls !== 0 || saved !== supplied || saved() !== 5) throw 'default ran for present value';
var marker = {}, caught;
try { throw marker; } catch (error) { caught = error; }
if (caught !== marker) throw 'simple catch changed';
print('ok');
"#,
    );
}
