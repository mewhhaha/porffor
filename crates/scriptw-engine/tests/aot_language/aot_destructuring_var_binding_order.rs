use scriptw_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, RealmBuilder, RunOptions,
};

fn assert_var_binding_order(source: &str) {
    scriptw_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let result = Engine::new(RealmBuilder::new().build())
        .observe_script(
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
        .expect("var BindingInitialization executes through emitted Wasm");
    assert_eq!(result.backend_used, ExecutionBackend::WasmAot);
    assert_eq!(
        result.completion,
        ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
        "{source}"
    );
}

#[test]
fn array_pattern_exhaustion_keeps_default_value_without_reading_value() {
    assert_var_binding_order(
        r#"
var log = [];
var iterator = {
  get next() {
    log.push('next');
    return function() {
      log.push('call');
      return {
        get done() { log.push('done'); return { valueOf() { throw 'coercion'; } }; },
        get value() { throw 'value after done'; }
      };
    };
  },
  get return() { throw 'close after done'; }
};
var source = { [Symbol.iterator]() { return iterator; } };
var [value = 'fallback'] = source;
value === 'fallback' && log.join(',') === 'next,call,done';
"#,
    );
}

#[test]
fn array_pattern_elisions_keep_cached_next_and_never_read_skipped_values() {
    assert_var_binding_order(
        r#"
var nextReads = 0, calls = 0, valueReads = 0, closes = 0;
var iterator = {
  get next() {
    nextReads++;
    return function() {
      calls++;
      Object.defineProperty(iterator, 'next', { get() { throw 'next reread'; } });
      return {
        done: false,
        get value() {
          valueReads++;
          if (calls !== 3) throw 'elision read value';
          return 7;
        }
      };
    };
  },
  return() { closes++; return {}; }
};
var source = { [Symbol.iterator]() { return iterator; } };
var [,, value] = source;
value === 7 && nextReads === 1 && calls === 3 && valueReads === 1 && closes === 1;
"#,
    );
}

#[test]
fn array_pattern_close_preserves_caller_error_realm_and_pending_throw() {
    assert_var_binding_order(
        r#"
var foreign = __scriptwCreateRealm();
var iterator = foreign.evalScript('({next() {return {done:false, value:1};}, return:1})');
var source = { [Symbol.iterator]() { return iterator; } };
var closeFailure;
try { var [first] = source; } catch (error) { closeFailure = error; }
var callerRealm = closeFailure instanceof TypeError &&
  !(closeFailure instanceof foreign.evalScript('TypeError'));
var second = foreign.evalScript('({next() {return {done:false};}, get return() {this.reads = 1; throw this.closeError;}})');
second.closeError = new RangeError('close');
var original = new Error('default');
function fail() { throw original; }
var caught;
try { var [value = fail()] = { [Symbol.iterator]() { return second; } }; }
catch (error) { caught = error; }
callerRealm && caught === original && second.reads === 1;
"#,
    );
}

#[test]
fn array_accumulation_keeps_cached_next_and_truthy_done_for_spread_and_typed_array() {
    assert_var_binding_order(
        r#"
function collect(typed) {
  var reads = 0, calls = 0, values = 0;
  var iterator = {
    get next() {
      reads++;
      return function() {
        if (this !== iterator) throw 'receiver';
        calls++;
        Object.defineProperty(iterator, 'next', { get() { throw 'next reread'; }, configurable: true });
        if (calls < 3) return { done: false, get value() { values++; return calls + 6; } };
        return {
          get done() { return { valueOf() { throw 'done coercion'; } }; },
          get value() { throw 'value after done'; }
        };
      };
    },
    get return() { throw 'unexpected close'; }
  };
  var source = { [Symbol.iterator]() { return iterator; } };
  var result = typed ? new Uint8Array(source) : [...source];
  return reads === 1 && calls === 3 && values === 2 &&
    result.length === 2 && result[0] === 7 && result[1] === 8;
}
collect(false) && collect(true);
"#,
    );
}

#[test]
fn array_accumulation_preserves_abrupt_identity_and_records_across_reentrant_gc() {
    assert_var_binding_order(
        r#"
var phases = ['next', 'done', 'value'], ok = true;
for (var phase of phases) {
  var marker = {}, closes = 0, later = 0, caught;
  var iterator = {
    next() {
      if (phase === 'next') throw marker;
      return {
        get done() { if (phase === 'done') throw marker; return false; },
        get value() { gc(); throw marker; }
      };
    },
    get return() { closes++; throw 'unexpected close'; }
  };
  try { var failed = [...{ [Symbol.iterator]() { return iterator; } }, ++later]; }
  catch (error) { caught = error; }
  ok = ok && caught === marker && closes === 0 && later === 0;
}
var object = {}, symbol = Symbol('retained'), big = 123456789012345678901234567890n;
var expected = [object, symbol, big], index = 0, nestedCalls = 0;
var outer = {
  [Symbol.iterator]() { return this; },
  next() {
    if (index === 3) return { done: true };
    var current = expected[index++];
    return { done: false, get value() {
      var innerIndex = 0;
      var inner = [...{
        [Symbol.iterator]() { return this; },
        next() { return innerIndex++ === 0 ? { done: false, value: current } : { done: true }; }
      }];
      gc();
      if (inner.length !== 1 || inner[0] !== current) throw 'reentrant record';
      nestedCalls++;
      return current;
    }};
  },
  get return() { throw 'unexpected close'; }
};
var result = [...outer];
ok && index === 3 && nestedCalls === 3 && result.length === 3 &&
  result[0] === object && result[1] === symbol && result[2] === big;
"#,
    );
}

#[test]
fn array_accumulation_preserves_caller_error_realm_and_foreign_throw_identity() {
    assert_var_binding_order(
        r#"
var foreign = __scriptwCreateRealm(), ForeignTypeError = foreign.evalScript('TypeError');
var foreignCollect = foreign.evalScript('(function(source) { try { return [...source]; } catch (error) { return error; } })');
function localCollect(source) { try { return [...source]; } catch (error) { return error; } }
var ok = true;
for (var invalidNext of [true, false]) {
  var foreignIterator = foreign.evalScript(invalidNext ? '({next: 1})' : '({next() { return 1; }})');
  var localIterator = invalidNext ? {next: 1} : {next() { return 1; }};
  var localError = localCollect({ [Symbol.iterator]() { return foreignIterator; } });
  var foreignError = foreignCollect({ [Symbol.iterator]() { return localIterator; } });
  var message = invalidNext ? 'array spread iterator next must be callable' :
    'array spread iterator next result must be object';
  ok = ok && localError instanceof TypeError && !(localError instanceof ForeignTypeError) &&
    foreignError instanceof ForeignTypeError && !(foreignError instanceof TypeError) &&
    localError.message === message && foreignError.message === message;
}
var marker = foreign.evalScript('({foreign: true})');
var throwing = foreign.evalScript('({next() { throw this.marker; }, get return() { throw "close"; }})');
throwing.marker = marker;
ok && localCollect({ [Symbol.iterator]() { return throwing; } }) === marker;
"#,
    );
}

#[test]
fn var_destructuring_computed_key_order() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/computed_key_order.js"
    ));
}

#[test]
fn var_destructuring_static_selected_object() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/static_selected_object.js"
    ));
}

#[test]
fn var_destructuring_repeated_targets() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/repeated_targets.js"
    ));
}

#[test]
fn var_destructuring_array_and_rest_order() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/array_and_rest_order.js"
    ));
}

#[test]
fn var_destructuring_abrupt_resolution() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/abrupt_resolution.js"
    ));
}

#[test]
fn var_destructuring_unscopables_and_nested_fallback() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/unscopables_and_nested_fallback.js"
    ));
}

#[test]
fn var_destructuring_lexical_and_function_cutoff() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/lexical_and_function_cutoff.js"
    ));
}

#[test]
fn var_destructuring_var_loop_heads() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/var_loop_heads.js"
    ));
}

#[test]
fn var_destructuring_unscopables_abrupt() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/unscopables_abrupt.js"
    ));
}

#[test]
fn var_destructuring_direct_eval_reference() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/direct_eval_reference.js"
    ));
}

#[test]
fn var_destructuring_conditional_rest_fallback_facts() {
    assert_var_binding_order(include_str!(
        "../fixtures/destructuring_var_binding_order/conditional_rest_fallback_facts.js"
    ));
}

#[test]
fn parenthesized_assignment_targets_keep_evaluation_order_and_rest_values() {
    assert_var_binding_order(
        r#"
var log = '', target = {}, value;
function base() { log += 'b'; return target; }
function key() { log += 'k'; return 'value'; }
var iterable = {
    [Symbol.iterator]() { return this; },
    next() { log += 'n'; return {value: 7, done: false}; },
    return() { log += 'r'; return {}; }
};
[((base()[key()]))] = iterable;
var order = log === 'bknr' && target.value === 7;
[(value), ...((target.arrayRest))] = [1, 2, 3];
({value: ((target.value)), ...((target.objectRest))} = {value: 4, extra: 5});
for ([(target.loop)] of [[6], [8]]) {}
for ({value: (target.objectLoop)} of [{value: 9}]) {}
order && value === 1 && target.arrayRest.length === 2 && target.arrayRest[1] === 3 &&
    target.value === 4 && target.objectRest.extra === 5 &&
    !Object.hasOwn(target.objectRest, 'value') && target.loop === 8 && target.objectLoop === 9;
"#,
    );
}

#[test]
fn parenthesized_super_targets_use_the_original_receiver() {
    assert_var_binding_order(
        r#"
var seen = [], prototype = {
    set value(v) { seen.push(this, v); },
    set rest(v) { seen.push(this, v.extra); }
};
var object = {
    __proto__: prototype,
    assign(source) {
        [(super.value)] = [11];
        ({...((super.rest))} = source);
    }
};
object.assign({extra: 12});
seen.length === 4 && seen[0] === object && seen[1] === 11 &&
    seen[2] === object && seen[3] === 12 && !Object.hasOwn(object, 'value');
"#,
    );
}
