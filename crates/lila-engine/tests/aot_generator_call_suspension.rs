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
        .expect("suspended calls must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}\nsource:\n{source}",
        outcome.completion
    );
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "source:\n{source}");
}

#[test]
fn async_generator_return_call_consumes_the_resumed_yield_value() {
    assert_trace(
        r#"
var calls = 0;
var gen = async function* () {
  calls++;
  return (function (arg) {
    var yield = arg + 1;
    return yield;
  }(yield));
};
var iterator = gen();
iterator.next().then(function (result) { print(result.value + ":" + result.done); });
iterator.next(42).then(function (result) { print(result.value + ":" + result.done); });
print("calls:" + calls);
"#,
        &["calls:1", "undefined:false", "43:true"],
    );
}

#[test]
fn yielded_method_key_and_argument_keep_the_getter_and_receiver() {
    assert_trace(
        r#"
var receiver = {
  value: 10,
  get method() {
    print("get");
    return function (first, second, third) {
      print("call:" + this.value + ":" + first + ":" + second + ":" + third);
      return this.value;
    };
  }
};
var key = { toString: function () { print("key"); return "method"; } };
function before() { print("before"); return 1; }
function after() { print("after"); return 3; }
function* gen() {
  return (receiver[yield "key?"])(before(), yield "argument?", after());
}
function report(result) { print(result.value + ":" + result.done); }
var iterator = gen();
report(iterator.next());
report(iterator.next(key));
Object.defineProperty(receiver, "method", {value: function () { print("replacement"); }});
receiver.value = 20;
receiver = {};
report(iterator.next(2));
"#,
        &[
            "key?:false",
            "key",
            "get",
            "before",
            "argument?:false",
            "after",
            "call:20:1:2:3",
            "20:true",
        ],
    );
}

#[test]
fn nested_discarded_calls_keep_each_activations_callee_and_arguments() {
    assert_trace(
        r#"
function check(value, expected) { print(value + ":" + expected); }
function* gen(label) {
  var receiver = { value: label, method: function () { return this.value; } };
  check(receiver[String(yield label)](), label);
  return label;
}
function report(result) { print(result.value + ":" + result.done); }
var left = gen("left");
var right = gen("right");
report(left.next());
report(right.next());
report(right.next("method"));
report(left.next("method"));
"#,
        &[
            "left:false",
            "right:false",
            "right:right",
            "right:true",
            "left:left",
            "left:true",
        ],
    );
}

#[test]
fn noncallable_method_checks_callability_after_all_arguments() {
    assert_trace(
        r#"
var receiver = { get method() { print("get"); return 1; } };
function after() { print("after"); return 3; }
function* gen() { return receiver.method(yield "argument?", after()); }
var iterator = gen();
print(iterator.next().value);
try { iterator.next(2); } catch (error) { print(error instanceof TypeError); }
"#,
        &["get", "argument?", "after", "true"],
    );
}

#[test]
fn nonsuspending_operands_retain_the_ordinary_expression_lowering() {
    assert_trace(
        r#"
function* gen() {
  return (function (object, value) { return object.read + value; })(
    { get read() { print("get"); return 40; } }, yield "pending"
  );
}
var iterator = gen();
print(iterator.next().value);
var result = iterator.next(2);
print(result.value + ":" + result.done);
"#,
        &["pending", "get", "42:true"],
    );
}

#[test]
fn computed_nullish_reference_evaluates_key_but_skips_its_coercion_and_arguments() {
    assert_trace(
        r#"
var key = { toString: function () { print("coerce"); return "method"; } };
function after() { print("argument"); return 1; }
function* gen() { return null[yield "key?"](after()); }
var iterator = gen();
print(iterator.next().value);
try { iterator.next(key); } catch (error) { print(error instanceof TypeError); }
"#,
        &["key?", "true"],
    );
}

#[test]
fn async_generator_call_return_awaits_thenables_and_keeps_activation_operands() {
    assert_trace(
        r#"
function make(label) {
  return function (value) {
    print("call:" + label + ":" + value);
    return { then: function (resolve) { print("then:" + label); resolve(label + value); } };
  };
}
async function* gen(label) { return make(label)(yield label); }
var left = gen("left");
var right = gen("right");
left.next().then(function (result) {
  print(result.value + ":" + result.done);
  return right.next();
}).then(function (result) {
  print(result.value + ":" + result.done);
  return right.next(2);
}).then(function (result) {
  print(result.value + ":" + result.done);
  return left.next(1);
}).then(function (result) { print(result.value + ":" + result.done); });
"#,
        &[
            "left:false",
            "right:false",
            "call:right:2",
            "then:right",
            "right2:true",
            "call:left:1",
            "then:left",
            "left1:true",
        ],
    );
}

#[test]
fn async_generator_abrupt_resume_skips_the_pending_call_and_later_arguments() {
    assert_trace(
        r#"
var marker = {};
function after() { print("argument"); }
function call() { print("call"); }
async function* gen() { return call(yield "pending", after()); }
var thrown = gen();
var returned = gen();
thrown.next().then(function (result) {
  print(result.value + ":" + result.done);
  return thrown.throw(marker);
}).then(function () { print("unexpected"); }, function (error) {
  print(error === marker);
  return returned.next();
}).then(function (result) {
  print(result.value + ":" + result.done);
  return returned.return(Promise.resolve(77));
}).then(function (result) { print(result.value + ":" + result.done); });
"#,
        &["pending:false", "true", "pending:false", "77:true"],
    );
}

#[test]
fn nested_assignment_retains_yielded_base_key_rhs_and_expression_value() {
    assert_trace(
        r#"
var first = { label: 'first', set slot(value) { print('set:' + this.label + ':' + value); } };
var selected = first;
function after() { print('after'); return 10; }
function consume(value, other) { print('call:' + value + ':' + other); return value; }
function* gen() {
  return consume((yield 'target?')[String(yield 'key?')] = yield 'rhs?', after());
}
var iterator = gen();
print(iterator.next().value);
print(iterator.next(selected).value);
selected = { label: 'replacement' };
print(iterator.next('slot').value);
var result = iterator.next(9);
print(result.value + ':' + result.done);
print(Object.hasOwn(selected, 'slot'));

function* standalone() { first[yield 'standalone-key?'] = yield 'standalone-rhs?'; }
iterator = standalone();
print(iterator.next().value);
print(iterator.next('slot').value);
print(iterator.next(11).done);

function check(value, expected) { print(value + ':' + expected); }
function* accessor() {
  let C = class { set [yield 'class-key?'](value) { print('accessor:' + value); } };
  var instance = new C();
  check(instance[yield 'assigned-key?'] = 9, 9);
}
iterator = accessor();
print(iterator.next().value);
print(iterator.next('slot').value);
print(iterator.next('slot').done);
"#,
        &[
            "target?",
            "key?",
            "rhs?",
            "set:first:9",
            "after",
            "call:9:10",
            "9:true",
            "false",
            "standalone-key?",
            "standalone-rhs?",
            "set:first:11",
            "true",
            "class-key?",
            "assigned-key?",
            "accessor:9",
            "9:9",
            "true",
        ],
    );
}

#[test]
fn assignment_keeps_raw_key_until_rhs_and_checks_nullish_base_after_rhs() {
    assert_trace(
        r#"
var state = 'before';
var key = { toString() { print('key:' + state); return 'slot'; } };
var receiver = { get slot() { throw new Error('assignment must not Get'); },
  set slot(value) { print('set:' + value); } };
function rhs() { state = 'after'; print('rhs'); return 7; }
function consume(value) { print('call:' + value); return value; }
function* gen() { return consume(receiver[yield 'key?'] = rhs()); }
var iterator = gen();
print(iterator.next().value);
print(iterator.next(key).value);

function* nullish() { return consume(null[yield 'null-key?'] = rhs()); }
iterator = nullish();
print(iterator.next().value);
try { iterator.next(key); } catch (error) { print(error instanceof TypeError); }

var marker = {};
function thrownRhs() { print('rhs-throw'); throw marker; }
function* abrupt() { return consume(null[yield 'throw-key?'] = thrownRhs()); }
iterator = abrupt();
print(iterator.next().value);
try { iterator.next(key); } catch (error) { print(error === marker); }
"#,
        &[
            "key?",
            "rhs",
            "key:after",
            "set:7",
            "call:7",
            "7",
            "null-key?",
            "rhs",
            "true",
            "throw-key?",
            "rhs-throw",
            "true",
        ],
    );
}

#[test]
fn abrupt_assignment_resume_skips_key_coercion_set_and_remaining_call() {
    assert_trace(
        r#"
var key = { toString() { print('key'); return 'slot'; } };
var receiver = { set slot(value) { print('set'); } };
function after() { print('after'); }
function consume(value) { print('call'); }
function* gen() { return consume(receiver[key] = yield 'rhs?', after()); }
var thrown = gen();
var returned = gen();
var marker = {};
print(thrown.next().value);
try { thrown.throw(marker); } catch (error) { print(error === marker); }
print(returned.next().value);
var result = returned.return(23);
print(result.value + ':' + result.done);

function* pendingKey() { return consume(receiver[yield 'key?'] = after()); }
var iterator = pendingKey();
print(iterator.next().value);
try { iterator.throw(marker); } catch (error) { print(error === marker); }
"#,
        &["rhs?", "true", "rhs?", "23:true", "key?", "true"],
    );
}

#[test]
fn suspended_assignment_preserves_symbol_setter_receiver_strictness_and_throw_identity() {
    assert_trace(
        r#"
var key = Symbol('slot');
var written = {};
var prototype = {};
var receiver = Object.create(prototype);
Object.defineProperty(prototype, key, {
  get() { throw new Error('assignment must not Get'); },
  set(value) { print(this === receiver); print(value === written); return 'ignored'; }
});
function consume(value) { print(value === written); return value; }
function* gen() { 'use strict'; return consume(receiver[yield 'symbol?'] = written); }
var iterator = gen();
print(iterator.next().value);
print(iterator.next(key).value === written);

var locked = {};
Object.defineProperty(locked, 'slot', { value: 1 });
function rhs() { print('rhs'); return 2; }
function* strict() { 'use strict'; return locked[yield 'strict?'] = rhs(); }
iterator = strict();
print(iterator.next().value);
try { iterator.next('slot'); } catch (error) { print(error instanceof TypeError); }
function* sloppy() { return locked[yield 'sloppy?'] = rhs(); }
iterator = sloppy();
print(iterator.next().value);
print(iterator.next('slot').value + ':' + locked.slot);

var marker = {};
var throwing = { set slot(value) { throw marker; } };
function* throws() { 'use strict'; return throwing[yield 'setter?'] = written; }
iterator = throws();
print(iterator.next().value);
try { iterator.next('slot'); } catch (error) { print(error === marker); }
"#,
        &[
            "symbol?", "true", "true", "true", "true", "strict?", "rhs", "true", "sloppy?", "rhs",
            "2:1", "setter?", "true",
        ],
    );
}
