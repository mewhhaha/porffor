use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_function_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
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
        .expect("prepared Function source must compile and execute through Wasm AOT");
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
fn nested_global_source_candidates_preserve_syntax_errors_and_live_callees() {
    assert_function_trace(
        r#"
eval(null);
var Constructor = Object.getPrototypeOf(async function* () {}).constructor;
Constructor('x = await 42');
Constructor('x = yield');
eval(null);
function rejectAwait() { Constructor('x = await 42', ''); }
function rejectYield() { Constructor('x = yield', ''); }
try { rejectAwait(); } catch (error) { print(error instanceof SyntaxError); }
try { rejectYield(); } catch (error) { print(error instanceof SyntaxError); }
var original = Constructor;
Constructor = function () { return 31; };
function replacement() { return Constructor('x = await 42', ''); }
print(replacement());
function shadow(Constructor) { return Constructor('x = yield', ''); }
print(shadow(function () { return 32; }));
Constructor = original;
async function* rejectAsyncAwait() { Constructor('x = await 43', ''); }
async function* rejectAsyncYield() { Constructor('x = yield 44', ''); }
rejectAsyncAwait().next().then(function () { print('wrong'); }, function (error) {
  print(error instanceof SyntaxError);
  return rejectAsyncYield().next();
}).then(function () { print('wrong'); }, function (error) { print(error instanceof SyntaxError); });
var body = 'return 9;';
function build() { return Function(body); }
print(build()());
"#,
        &["true", "true", "31", "32", "9", "true", "true"],
    );
}

#[test]
fn static_constructor_bodies_are_fresh_global_functions_with_real_parameters() {
    assert_function_trace(
        r#"
var outer = 3;
let lexical = 7;
function create() {
  let outer = 99;
  return Function('value', 'return value + outer + lexical;');
}
var first = create();
var second = create();
print(first(2));
print(first !== second);
print(first.name + ':' + first.length);
print(Function('return typeof anonymous;')());
var closes = Function('value', 'return function () { return value; };');
print(closes(8)());
print(closes(9)());
var Nested = Function('return Function("return 11;");');
print(Nested()());
"#,
        &["12", "true", "anonymous:1", "undefined", "8", "9", "11"],
    );
}

#[test]
fn prepared_generator_async_and_async_generator_use_normal_execution_protocols() {
    assert_function_trace(
        r#"
var GeneratorFunction = (function* () {}).constructor;
var AsyncFunction = (async function () {}).constructor;
var AsyncGeneratorFunction = (async function* () {}).constructor;
var generator = GeneratorFunction('value', 'yield value; return value + 1;')(4);
print(generator.next().value);
print(generator.next().value);
AsyncFunction('value', 'return await value + 1;')(6).then(function (value) { print(value); });
var asyncGenerator = AsyncGeneratorFunction('value', 'yield await value; return value + 1;')(8);
asyncGenerator.next().then(function (result) { print(result.value); return asyncGenerator.next(); }).then(function (result) { print(result.value); });
"#,
        &["4", "5", "7", "8", "9"],
    );
}

#[test]
fn prepared_grammar_errors_are_catchable_and_replacement_identity_is_preserved() {
    assert_function_trace(
        r#"
print('before');
try { Function('a)', 'return 1;'); }
catch (error) { print(error instanceof SyntaxError); }
try { Function('a,a', '"use strict"; return a;'); }
catch (error) { print(error instanceof SyntaxError); }
print(Function('')());
var Original = Function;
Function = function (body) { print(body); return 42; };
print(Function('return 4;'));
Function = Original;
print(Function('return 5;')());
"#,
        &[
            "before",
            "true",
            "true",
            "undefined",
            "return 4;",
            "42",
            "5",
        ],
    );
}

#[test]
fn prepared_function_source_preserves_reflection_and_constructor_behavior() {
    assert_function_trace(
        r#"
var Constructor = Function('value', 'this.value = value;');
var instance = new Constructor(13);
print(instance.value);
print(instance instanceof Constructor);
print(Constructor.toString() === 'function anonymous(value\n) {\nthis.value = value;\n}');
var strict = Function('"use strict"; return this;');
print(strict() === undefined);
"#,
        &["13", "true", "true", "true"],
    );
}

#[test]
fn runtime_argument_coercions_precede_prepared_source_selection() {
    assert_function_trace(
        r#"
Function('value', 'return value;');
try { Function('bad)', 'return value;'); } catch (error) {}
var trace = '';
var compiled = Reflect.construct(Function, [
  { toString() { trace += 'parameter;'; return 'value'; } },
  { toString() { trace += 'body;'; return 'return value;'; } }
]);
print(trace);
print(compiled(17));
trace = '';
try {
  Reflect.construct(Function, [
    { toString() { trace += 'parameter;'; return 'bad)'; } },
    { toString() { trace += 'body;'; return 'return value;'; } }
  ]);
} catch (error) { print(error instanceof SyntaxError); }
print(trace);
"#,
        &["parameter;body;", "17", "true", "parameter;body;"],
    );
}

#[test]
fn unregistered_source_arguments_still_coerce_in_order_and_propagate_abrupt_completion() {
    assert_function_trace(
        r#"
var trace = '';
var sentinel = {};
try {
  Function(
    { toString() { trace += 'first;'; return 'x'; } },
    { toString() { trace += 'second;'; throw sentinel; } },
    { toString() { trace += 'third;'; return ''; } }
  );
} catch (error) { print(error === sentinel); }
print(trace);
"#,
        &["true", "first;second;"],
    );
}

#[test]
fn undefined_source_candidates_preserve_void_operand_effects_and_hoisted_binding_values() {
    assert_function_trace(
        r#"
var calls = 0;
print(Function(void ++calls)());
print(calls);
var generated = Function(hoisted);
var hoisted;
print(generated());
var sentinel = {};
try { Function(void (function () { throw sentinel; })()); }
catch (error) { print(error === sentinel); }
"#,
        &["undefined", "1", "undefined", "true"],
    );
}

#[test]
fn boxed_source_and_literal_coercion_hooks_execute_before_guarded_code_selection() {
    assert_function_trace(
        r#"
var boxed = Object("return 'boxed';");
print(Function(boxed)());
let number = new Number(1);
print(Function(number)());
var calls = 0;
var body = {toString() { ++calls; return "return 'hook';"; }};
print(Function(body)());
print(calls);
try { Function({}); } catch (error) { print(error instanceof SyntaxError); }
Function("return 'changed';");
Object.defineProperty(boxed, 'toString', {get() {
  ++calls;
  return function () { return "return 'changed';"; };
}});
print(Function(boxed)());
print(calls);
"#,
        &["boxed", "undefined", "hook", "1", "true", "changed", "2"],
    );
}

#[test]
fn reflected_foreign_constructor_preserves_argument_source_and_new_target_getter() {
    assert_function_trace(
        r#"
var other = __lilaCreateRealm().global;
other.marker = 21;
var reads = 0;
var newTarget = function () {}.bind(null);
Object.defineProperty(newTarget, 'prototype', { get() { reads++; return null; } });
var created = Reflect.construct(other.Function, ['return marker;'], newTarget);
print(created());
print(reads);
print(Object.getPrototypeOf(created) === Function.prototype);
"#,
        &["21", "1", "true"],
    );
}

#[test]
fn function_family_construction_reads_prototype_once_after_source_validation() {
    assert_function_trace(
        r#"
var GeneratorFunction = Object.getPrototypeOf(function* () {}).constructor;
var AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;
var reads = 0;
var newTarget = function () {}.bind(null);
Object.defineProperty(newTarget, 'prototype', {get() { ++reads; return null; }});
Reflect.construct(Function, [], newTarget);
print(reads);
Reflect.construct(GeneratorFunction, [], newTarget);
print(reads);
Reflect.construct(AsyncFunction, [], newTarget);
print(reads);
Reflect.construct(AsyncGeneratorFunction, [], newTarget);
print(reads);
try { Reflect.construct(Function, ['a)', 'return 1;'], newTarget); }
catch (error) { print(error instanceof SyntaxError); }
print(reads);
var sentinel = {};
try {
  Reflect.construct(Function, [{toString() { throw sentinel; }}], newTarget);
} catch (error) { print(error === sentinel); }
print(reads);
Function('return 17;');
var trace = '';
var orderedNewTarget = function () {}.bind(null);
Object.defineProperty(orderedNewTarget, 'prototype', {
  get() { trace += 'prototype;'; return null; }
});
var created = Reflect.construct(Function, [{
  toString() { trace += 'source;'; return 'return 17;'; }
}], orderedNewTarget);
print(trace);
print(created());
"#,
        &[
            "1",
            "2",
            "3",
            "4",
            "true",
            "4",
            "true",
            "4",
            "source;prototype;",
            "17",
        ],
    );
}

#[test]
fn derived_constructor_candidates_survive_effects_without_replacing_the_live_alias() {
    assert_function_trace(
        r#"
eval(null);
var Constructor = Object.getPrototypeOf(async function* () {}).constructor;
eval(null);
var created = Constructor('yield 1;');
created().next().then(function (result) { print(result.value); });
Constructor = function (body) { print(body); return 42; };
print(Constructor('yield 2;'));
void 0;
"#,
        &["yield 2;", "42", "1"],
    );
}

#[test]
fn finite_source_tables_preserve_getters_callback_dispatch_and_live_tuple_selection() {
    assert_function_trace(
        r#"
const entries = [{source: '41'}, {source: '42'}];
var reads = 0;
Object.defineProperty(entries[0], 'source', {get() { ++reads; return '42'; }});
entries.forEach(entry => print(Function('return ' + entry.source)()));
print(reads);
entries.forEach = function (callback) { print('replacement'); };
entries.forEach(entry => print(Function('return ' + entry.source)()));
print(reads);
"#,
        &["42", "42", "1", "replacement", "1"],
    );
}

#[test]
fn primitive_wrappers_observe_conversion_hook_order_and_abrupt_completion() {
    assert_function_trace(
        r#"
var wrappers = [Object('text'), Object(1), Object(true)];
var log = '';
for (var wrapper of wrappers) {
  Object.defineProperty(wrapper, 'toString', {get() {
    log += 'get;';
    return function () { log += 'call;'; return {}; };
  }});
  wrapper.valueOf = function () { log += 'value;'; return 'converted'; };
  print(String(wrapper));
}
print(log);
var sentinel = {};
var boxed = Object(1);
Object.defineProperty(boxed, Symbol.toPrimitive, {get() { throw sentinel; }});
try { Function(boxed); } catch (error) { print(error === sentinel); }
"#,
        &[
            "converted",
            "converted",
            "converted",
            "get;call;value;get;call;value;get;call;value;",
            "true",
        ],
    );
}
