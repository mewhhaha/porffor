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
fn finite_constructor_arrays_and_iterators_keep_all_four_execution_protocols() {
    let definitions = r#"
var Ordinary = Function;
var Generator = (function* () {}).constructor;
var Async = (async function () {}).constructor;
var AsyncGenerator = (async function* () {}).constructor;
var constructors = [Ordinary, Generator, Async, AsyncGenerator];
var iterable = { [Symbol.iterator]() {
  var index = 0;
  return { next() { return { done: index >= 4, value: constructors[index++] }; } };
} };
"#;
    for prefix in ["", "'use strict';"] {
        for iterable in ["constructors", "iterable"] {
            assert_function_trace(
                &format!(
                    r#"{prefix}{definitions}
var created = [];
for (const constructor of {iterable}) {{
  created.push(constructor('value', 'return value;'));
  try {{ constructor('#!\n_'); throw 'missing SyntaxError'; }}
  catch (error) {{ if (!(error instanceof SyntaxError)) throw error; }}
}}
print(created[0](41));
print(created[1](42).next().value);
created[2](43).then(function (value) {{ print(value); }});
created[3](44).next().then(function (result) {{ print(result.value); }});
"#
                ),
                &["41", "42", "43", "44"],
            );
        }
    }
}

#[test]
fn resolved_prototypes_keep_live_constructor_getters_replacements_and_abrupt_completion() {
    let source = r#"
function* source() {}
var read = Object.getPrototypeOf;
var prototype = read(source);
var constructor = prototype.constructor;
print(constructor('yield 31;')().next().value);
var descriptor = Object.getOwnPropertyDescriptor(prototype, 'constructor');
Object.defineProperty(prototype, 'constructor', {
  configurable: true,
  get: function () {
    print('get');
    return function (body) { print(body); return 32; };
  }
});
var replacement = prototype.constructor;
print(replacement('yield }'));
var marker = {};
Object.defineProperty(prototype, 'constructor', {
  configurable: true,
  get: function () { print('throw'); throw marker; }
});
var argument = { toString: function () { print('wrong conversion'); return 'yield 33;'; } };
try { prototype.constructor(argument); throw 'missing getter throw'; }
catch (error) { print(error === marker); }
Object.defineProperty(prototype, 'constructor', descriptor);
var restored = prototype.constructor;
var first = { toString: function () { print('first'); return 'value'; } };
var second = { toString: function () { print('second'); throw marker; } };
try { restored(first, second, 'yield value;'); throw 'missing conversion throw'; }
catch (error) { print(error === marker); }
"#;
    for prefix in ["", "'use strict';"] {
        assert_function_trace(
            &format!("{prefix}{source}"),
            &[
                "31", "get", "yield }", "32", "throw", "true", "first", "second", "true",
            ],
        );
    }
}

#[test]
fn constructor_values_keep_defining_realms_after_array_iteration() {
    let source = r#"
var foreign = __lilaCreateRealm().global;
var constructors = [Function, foreign.Function];
var index = 0;
for (const constructor of constructors) {
  var realm = index++ === 0 ? globalThis : foreign;
  var created = constructor('return globalThis;');
  print(created() === realm);
  print(Object.getPrototypeOf(created) === realm.Function.prototype);
  try { constructor('return }'); throw 'missing SyntaxError'; }
  catch (error) { print(Object.getPrototypeOf(error) === realm.SyntaxError.prototype); }
}
"#;
    for prefix in ["", "'use strict';"] {
        assert_function_trace(&format!("{prefix}{source}"), &["true"; 6]);
    }
}
