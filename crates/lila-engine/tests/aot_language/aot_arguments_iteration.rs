use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_wasm_true(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Arguments iteration must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn mapped_arguments_iterate_without_explicit_array_or_symbol_references() {
    assert_wasm_true(
        r#"
function collect(a, b, c) {
  var result = "";
  for (var value of arguments) result += value + ":";
  return result;
}
collect(2, 1, 3) === "2:1:3:" && collect() === "";
"#,
    );
}

#[test]
fn unmapped_and_non_simple_arguments_iterate() {
    assert_wasm_true(
        r#"
function strict(a, b) {
  "use strict";
  var result = "";
  for (const value of arguments) result += value + ":";
  return result;
}
function defaults(a = 9) {
  var result = "";
  for (let value of arguments) result += value + ":";
  return result;
}
strict(4, 5) === "4:5:" && strict() === "" &&
  defaults(6, 7) === "6:7:" && defaults() === "";
"#,
    );
}

#[test]
fn aliased_arguments_reach_the_runtime_exotic_branch() {
    assert_wasm_true(
        r#"
function collect(source) {
  var result = "";
  for (const value of source) result += value + ":";
  return result;
}
function mapped() { return arguments; }
function unmapped() { "use strict"; return arguments; }
var escaped = mapped(2, 3);
function captured() { return collect(escaped); }
captured() === "2:3:" && collect(unmapped(4, 5)) === "4:5:" &&
  collect([6, 7]) === "6:7:";
"#,
    );
}

#[test]
fn mapped_aliasing_and_live_length_survive_iteration() {
    assert_wasm_true(
        r#"
function mapped(a, b, c) {
  var result = "";
  for (var value of arguments) {
    result += value + ":";
    b = 8;
    arguments.length = 2;
  }
  return result;
}
function unmapped(a, b, c) {
  "use strict";
  var result = "";
  for (var value of arguments) {
    result += value + ":";
    b = 8;
    arguments.length = 2;
  }
  return result;
}
mapped(1, 2, 3) === "1:8:" && unmapped(1, 2, 3) === "1:2:";
"#,
    );
}

#[test]
fn arguments_index_reads_preserve_own_properties_and_inherited_receivers() {
    assert_wasm_true(
        r#"
function mapped(a, b, c) {
  return { source: arguments, set(value) { b = value; } };
}
function unmapped(a, b, c) {
  "use strict";
  return { source: arguments, set(value) { b = value; } };
}
var factories = [mapped, unmapped];
var ok = true;
for (var m = 0; m < factories.length; m++) {
  var state = factories[m](1, 2, 3);
  var source = state.source;
  var receiver = source;
  var reads = 0;
  var proto = Object.create(Object.getPrototypeOf(source));
  Object.defineProperty(proto, '1', { get() {
    reads++;
    if (this !== receiver) throw "wrong inherited receiver";
    return 9;
  } });
  Object.defineProperty(proto, '3', { get() {
    if (this !== source) throw "wrong extended receiver";
    return 13;
  } });
  Object.setPrototypeOf(source, proto);
  state.set(7);
  var expected = m === 0 ? 7 : 2;
  var key = '1';
  ok = ok && source[1] === expected && source[key] === expected &&
    Reflect.get(source, key) === expected && reads === 0;
  delete source[1];
  state.set(11);
  ok = ok && source[1] === 9 && source[key] === 9 &&
    Reflect.get(source, key) === 9 && reads === 3;
  receiver = {};
  ok = ok && Reflect.get(source, key, receiver) === 9 && reads === 4;
  receiver = source;
  Object.defineProperty(source, key, { value: undefined, configurable: true });
  ok = ok && source[1] === undefined && Reflect.get(source, key) === undefined && reads === 4;
  var ownReads = 0;
  Object.defineProperty(source, key, { configurable: true, get() {
    ownReads++;
    if (this !== source) throw "wrong own receiver";
    return undefined;
  } });
  ok = ok && source[1] === undefined && Reflect.get(source, key) === undefined &&
    ownReads === 2 && reads === 4;
  delete source[1];
  source.length = 4;
  var values = '';
  for (var value of source) values += value + ':';
  ok = ok && values === '1:9:3:13:' && reads === 5;
}
ok;
"#,
    );
}

#[test]
fn inherited_arguments_index_getters_preserve_thrown_values() {
    assert_wasm_true(
        r#"
function mapped(a, b) { return arguments; }
function unmapped(a, b) { "use strict"; return arguments; }
var factories = [mapped, unmapped];
var ok = true;
for (var m = 0; m < factories.length; m++) {
  var source = factories[m](1, 2);
  delete source[1];
  var marker = {};
  var reads = 0;
  var proto = Object.create(Object.getPrototypeOf(source));
  Object.defineProperty(proto, '1', { get() {
    reads++;
    if (this !== source) throw "wrong receiver";
    throw marker;
  } });
  Object.setPrototypeOf(source, proto);
  var caught = 0;
  try { source[1]; } catch (error) { if (error === marker) caught++; }
  try { Reflect.get(source, '1'); } catch (error) { if (error === marker) caught++; }
  var visited = '';
  try { for (var value of source) visited += value; }
  catch (error) { if (error === marker) caught++; }
  ok = ok && caught === 3 && reads === 3 && visited === '1';
}
ok;
"#,
    );
}

#[test]
fn adjacent_consumers_keep_arguments_values_and_exhaustion() {
    assert_wasm_true(
        r#"
function check(a, b, c) {
  var spread = [...arguments];
  var [first, second, third] = arguments;
  var iterator = arguments[Symbol.iterator]();
  var one = iterator.next();
  var two = iterator.next();
  arguments.length = 2;
  var done = iterator.next();
  arguments.length = 3;
  var stillDone = iterator.next();
  return spread.join(":") === "2:1:3" && first === 2 && second === 1 && third === 3 &&
    one.value === 2 && one.done === false && two.value === 1 && two.done === false &&
    done.value === undefined && done.done === true && stillDone.done === true;
}
check(2, 1, 3);
"#,
    );
}

#[test]
fn ordinary_and_proxy_iterators_keep_receiver_next_cache_and_close() {
    assert_wasm_true(
        r#"
var calls = "";
var source = {};
var iterator = {
  get next() {
    calls += "next;";
    return new Proxy(function() {
      calls += this === iterator ? "step;" : "wrong-step;";
      return { value: 7, done: false };
    }, {});
  },
  return() {
    calls += this === iterator ? "close;" : "wrong-close;";
    return {};
  }
};
Object.defineProperty(source, Symbol.iterator, { get() {
  calls += "method;";
  return new Proxy(function() {
    calls += this === source ? "call;" : "wrong-call;";
    return iterator;
  }, {});
} });
var value;
var count = 0;
for (value of source) { if (++count === 2) break; }
value === 7 && count === 2 && calls === "method;call;next;step;step;close;";
"#,
    );
}

#[test]
fn strings_use_the_symbol_protocol_and_nullish_sources_still_throw() {
    assert_wasm_true(
        r#"
var result = "";
var count = 0;
for (const value of "a\uD83D\uDE00b") { result += value; count++; }
var throws = 0;
try { for (const value of null) throws += 100; } catch (e) { if (e instanceof TypeError) throws++; }
try { for (const value of undefined) throws += 100; } catch (e) { if (e instanceof TypeError) throws++; }
result === "a\uD83D\uDE00b" && count === 3 && throws === 2;
"#,
    );
}

#[test]
fn sync_disposable_for_of_acquires_arguments_and_disposes_each_value() {
    assert_wasm_true(
        r#"
var trace = "";
var resource = { [Symbol.dispose]() { trace += "dispose;"; } };
function visit() {
  for (using value of arguments) { trace += "body;"; }
}
visit(resource, null);
trace === "body;dispose;body;";
"#,
    );
}

#[test]
fn async_disposable_for_of_acquires_arguments_before_suspending() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            r#"
var trace = "";
async function visit() {
  for (await using value of arguments) { trace += "body;"; }
  trace += "end;";
}
visit(null, undefined).then(function() {
  print(trace);
}, function(error) { print("rejected:" + error); });
void 0;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("await-using Arguments iteration must execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine("body;body;end;".to_string())]
    );
}

#[test]
fn arguments_own_iterator_override_preserves_receiver_and_close() {
    assert_wasm_true(
        r#"
function check() {
  var source = arguments;
  var trace = "";
  var iterator = {
    next() { trace += this === iterator ? "step;" : "wrong-step;"; return { value: 9, done: false }; },
    return() { trace += "close;"; return {}; }
  };
  source[Symbol.iterator] = function() {
    trace += this === source ? "call;" : "wrong-call;";
    return iterator;
  };
  var observed;
  for (var value of source) { observed = value; break; }
  return observed === 9 && trace === "call;step;close;";
}
check(2, 3);
"#,
    );
}

#[test]
fn arguments_iterator_getter_runs_once_and_propagates_throw() {
    assert_wasm_true(
        r#"
function check() {
  var source = arguments;
  var gets = 0;
  var called = 0;
  Object.defineProperty(source, Symbol.iterator, { configurable: true, get() {
    gets++;
    if (this !== source) throw "wrong-getter-receiver";
    return function() { called++; return [9][Symbol.iterator](); };
  } });
  var observed = "";
  for (var value of source) observed += value;
  if (gets !== 1 || called !== 1 || observed !== "9") return false;
  var marker = {};
  Object.defineProperty(source, Symbol.iterator, { get() { gets++; throw marker; } });
  try { for (var value of source) return false; } catch (error) { return error === marker && gets === 2; }
  return false;
}
check(2, 3);
"#,
    );
}

#[test]
fn arguments_own_non_callable_iterator_does_not_fall_back() {
    assert_wasm_true(
        r#"
function check(method) {
  var source = arguments;
  source[Symbol.iterator] = method;
  try { for (var value of source) return false; } catch (error) { return error instanceof TypeError; }
  return false;
}
function stringKey() {
  arguments["Symbol.iterator"] = function() { throw "wrong-key"; };
  var result = "";
  for (var value of arguments) result += value;
  return result;
}
check(undefined) && check(null) && check(1) && stringKey(2, 3) === "23";
"#,
    );
}
