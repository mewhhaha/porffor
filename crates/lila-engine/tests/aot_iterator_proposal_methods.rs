//! `Iterator.prototype.chunks`, `windows`, `includes` and `join` through the
//! Wasm AOT product path, plus the `take`/`drop` limit ceiling that landed with
//! the same Test262 update.

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
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
        .expect("iterator proposal method must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion
    );
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "source:\n{source}");
}

#[test]
fn chunks_yields_fresh_arrays_and_completes_like_an_iterator_helper() {
    assert_trace(
        r#"
function* values(count) { for (let index = 0; index < count; ++index) yield index; }
function show(iterator) {
  return Array.from(iterator).map(function (chunk) { return '[' + chunk.join(',') + ']'; }).join('');
}
print(show(values(5).chunks(2)));
print(show(values(4).chunks(2)));
print(show(values(2).chunks(5)));
print(show(values(0).chunks(1)));
var helper = values(3).chunks(1);
print(Object.getPrototypeOf(helper) === Object.getPrototypeOf(Iterator.from([0]).map(function (x) { return x; })));
print(helper instanceof Iterator);

var returns = 0;
class Counting extends Iterator {
  next() { return { done: false, value: 1 }; }
  return() { ++returns; return {}; }
}
var started = new Counting().chunks(2);
started.return();
started.return();
print('suspended-start return:' + returns);
var yielded = new Counting().chunks(2);
yielded.next();
yielded.return();
print('suspended-yield return:' + returns);
print(JSON.stringify(yielded.next()));

var closed = 0;
var closable = {
  __proto__: Iterator.prototype,
  get next() { throw new Error('next must not be read'); },
  return() { ++closed; return {}; },
};
for (const size of [undefined, '1', NaN, 1.5, Infinity, 0, -0, 2 ** 32]) {
  try { closable.chunks(size); print('accepted'); }
  catch (error) { print(error.constructor.name + ':' + closed); }
}
values(1).chunks(2 ** 32 - 1);
print('max accepted');

var nextCalls = 0;
var throwing = {
  next() { ++nextCalls; throw new Error('boom'); },
  return() { throw new Error('return must not be called'); },
};
var failing = Iterator.prototype.chunks.call(throwing, 2);
try { failing.next(); } catch (error) { print(error.message); }
print(JSON.stringify(failing.next()) + ':' + nextCalls);

var reentrant;
reentrant = Iterator.prototype.chunks.call({
  next() { reentrant.next(); return { done: false, value: 0 }; },
}, 1);
try { reentrant.next(); } catch (error) { print(error.constructor.name); }
"#,
        &[
            "[0,1][2,3][4]",
            "[0,1][2,3]",
            "[0,1]",
            "",
            "true",
            "true",
            "suspended-start return:1",
            "suspended-yield return:2",
            "{\"done\":true}",
            "TypeError:1",
            "TypeError:2",
            "TypeError:3",
            "TypeError:4",
            "TypeError:5",
            "RangeError:6",
            "RangeError:7",
            "RangeError:8",
            "max accepted",
            "boom",
            "{\"done\":true}:1",
            "TypeError",
        ],
    );
}

#[test]
fn windows_slides_a_private_buffer_and_validates_undersized() {
    assert_trace(
        r#"
function* values(count) { for (let index = 0; index < count; ++index) yield index; }
function show(iterator) {
  return Array.from(iterator).map(function (window) { return '[' + window.join(',') + ']'; }).join('');
}
print(show(values(5).windows(2)));
print(show(values(4).windows(3)));
print(show(values(3).windows(1)));
print(show(values(3).windows(5)));
print(show(values(3).windows(5, 'only-full')));
print(show(values(3).windows(5, 'allow-partial')));
print(show(values(3).windows(2, 'allow-partial')));

var sliding = values(4).windows(2);
var first = sliding.next().value;
first[1] = 'mutated';
first.length = 0;
print('[' + sliding.next().value.join(',') + ']');
print('[' + sliding.next().value.join(',') + ']');

var effects = [];
var observed = {
  get next() { effects.push('get next'); return function () { return { done: true }; }; },
  get return() { effects.push('get return'); return undefined; },
};
for (const undersized of [null, '', 'something else', 0, true, {}, Symbol()]) {
  try { Iterator.prototype.windows.call(observed, 1, undersized); print('accepted'); }
  catch (error) { print(error.constructor.name); }
}
try { Iterator.prototype.windows.call(observed, 0, 'bad'); } catch (error) { print(error.constructor.name); }
print(effects.join(','));
effects.length = 0;
Iterator.prototype.windows.call(observed, 1);
print(effects.join(','));
"#,
        &[
            "[0,1][1,2][2,3][3,4]",
            "[0,1,2][1,2,3]",
            "[0][1][2]",
            "",
            "",
            "[0,1,2]",
            "[0,1][1,2]",
            "[1,2]",
            "[2,3]",
            "TypeError",
            "TypeError",
            "TypeError",
            "TypeError",
            "TypeError",
            "TypeError",
            "TypeError",
            "RangeError",
            "get return,get return,get return,get return,get return,get return,get return,get return",
            "get next",
        ],
    );
}

#[test]
fn includes_compares_with_same_value_zero_and_closes_on_match() {
    assert_trace(
        r#"
function* values() { yield 1; yield NaN; yield -0; yield 'x'; }
print(values().includes(NaN));
print(values().includes(0));
print(values().includes('x'));
print(values().includes('y'));
print([4, 5, 6, 7].values().includes(4, 1));
print([4, 5, 6, 7].values().includes(5, 1));
print([4, 5, 6, 7].values().includes(5, -0));

var nextCalls = 0;
var closes = 0;
var counting = {
  __proto__: Iterator.prototype,
  next() { ++nextCalls; return nextCalls < 4 ? { done: false, value: nextCalls } : { done: true }; },
  return() { ++closes; return {}; },
};
print(counting.includes(2) + ':' + nextCalls + ':' + closes);
nextCalls = 0;
print(counting.includes(1, Infinity) + ':' + nextCalls + ':' + closes);

var closed = 0;
var closable = {
  __proto__: Iterator.prototype,
  get next() { throw new Error('next must not be read'); },
  return() { ++closed; return {}; },
};
for (const skipped of [NaN, 0.5, '1', null, 1n, -1, -Infinity, Number.MAX_SAFE_INTEGER + 1]) {
  try { closable.includes(0, skipped); print('accepted'); }
  catch (error) { print(error.constructor.name + ':' + closed); }
}
"#,
        &[
            "true",
            "true",
            "true",
            "false",
            "false",
            "true",
            "true",
            "true:2:1",
            "false:4:1",
            "TypeError:1",
            "TypeError:2",
            "TypeError:3",
            "TypeError:4",
            "TypeError:5",
            "RangeError:6",
            "RangeError:7",
            "RangeError:8",
        ],
    );
}

#[test]
fn join_coerces_separator_first_and_closes_on_coercion_failure() {
    assert_trace(
        r#"
print('<' + [].values().join() + '>');
print(['one', 'two', 'three'].values().join());
print(['one', 'two', 'three'].values().join(''));
print(['one', 'two'].values().join(null));
print(['a', null, 'b', undefined].values().join('-'));
print([{ toString() { return 'value'; } }, 0, true].values().join());

var effects = [];
var separator = { toString() { effects.push('toString'); return '&&'; } };
var tracked = {
  get next() {
    effects.push('get next');
    var count = 0;
    return function () { ++count; return count < 3 ? { done: false, value: count } : { done: true }; };
  },
};
print(Iterator.prototype.join.call(tracked, separator));
print(effects.join(','));

var closed = 0;
var throwingSeparator = { toString() { throw new Error('separator'); } };
var closable = {
  get next() { throw new Error('next must not be read'); },
  return() { ++closed; },
};
try { Iterator.prototype.join.call(closable, throwingSeparator); }
catch (error) { print(error.message + ':' + closed); }

var throwingValue = { toString() { throw new Error('value'); } };
var yielding = {
  next() { return { done: false, value: throwingValue }; },
  return() { ++closed; },
};
try { Iterator.prototype.join.call(yielding); }
catch (error) { print(error.message + ':' + closed); }

var protocolViolation = { next() { return null; }, get return() { throw new Error('closed'); } };
try { Iterator.prototype.join.call(protocolViolation); }
catch (error) { print(error.constructor.name); }
"#,
        &[
            "<>",
            "one,two,three",
            "onetwothree",
            "onenulltwo",
            "a--b-",
            "value,0,true",
            "1&&2",
            "toString,get next",
            "separator:1",
            "value:2",
            "TypeError",
        ],
    );
}

#[test]
fn take_and_drop_reject_finite_limits_above_max_safe_integer() {
    assert_trace(
        r#"
function* values() { yield 1; yield 2; }
for (const method of ['take', 'drop']) {
  var closed = 0;
  var closable = {
    __proto__: Iterator.prototype,
    get next() { throw new Error('next must not be read'); },
    return() { ++closed; return {}; },
  };
  try { closable[method](Number.MAX_SAFE_INTEGER + 1); print('accepted'); }
  catch (error) { print(method + ':' + error.constructor.name + ':' + closed); }
  print(method + ':' + values()[method](Number.MAX_SAFE_INTEGER).toArray().length);
  print(method + ':' + values()[method](Infinity).toArray().length);
}
"#,
        &[
            "take:RangeError:1",
            "take:2",
            "take:2",
            "drop:RangeError:1",
            "drop:0",
            "drop:0",
        ],
    );
}
