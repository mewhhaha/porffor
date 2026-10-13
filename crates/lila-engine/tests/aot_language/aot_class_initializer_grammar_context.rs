use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, ObservedJsValue,
    PromiseRejectionPolicy, RealmBuilder, RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';"] {
        let source = format!("{directive}\n{source}\ntrue;");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    promise_rejection_policy: PromiseRejectionPolicy::FailRun,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("class initializer grammar reaches real Wasm execution");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{source}"
        );
        assert_eq!(
            observed.output_events,
            expected
                .iter()
                .map(|line| HostOutputEvent::PrintLine((*line).into()))
                .collect::<Vec<_>>(),
            "{source}"
        );
    }
}

#[test]
fn class_evaluation_references_are_strict_inside_a_sloppy_activation() {
    assert_trace(
        r#"
function check(operation) {
  try { operation(); } catch (error) {
    if (!(error instanceof TypeError)) throw error;
    print('strict');
    return;
  }
  throw 'class evaluation suppressed a strict reference error';
}
check(function () { class C { [Object.preventExtensions({}).key = 1]() {} } });
check(function () { return class { [Object.preventExtensions({}).key = 1]() {} }; });
check(function () { class C extends (Object.preventExtensions({}).key = 1) {} });
check(function () { return class extends (Object.preventExtensions({}).key = 1) {}; });
check(function () { return class { [eval('Object.preventExtensions({}).key = 1')]() {} }; });
"#,
        &["strict", "strict", "strict", "strict", "strict"],
    );
}

#[test]
fn class_evaluation_restores_the_enclosing_reference_strictness() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = r#"
var object = Object.preventExtensions({});
class Outer { [class Inner { method() {} }]() {} }
object.key = 1;
function sloppy() {
  var value = class { [class {}]() {} };
  object.key = 2;
  return object.key === undefined;
}
sloppy() && object.key === undefined;
"#;
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
        .expect("sloppy references after class evaluation still execute");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn async_class_field_initializers_read_the_outer_await_binding_at_initialization() {
    assert_trace(
        r#"
var await = 7;
async function make() {
  class C {
    value = await;
    #private = await;
    static value = await;
    static #staticPrivate = await;
    accessor item = await;
    accessor #item = await;
    static accessor other = await;
    static accessor #other = await;
    read() { return this.#private + ':' + this.#item; }
    static read() { return this.#staticPrivate + ':' + this.#other; }
  }
  return await Promise.resolve(C);
}
var ready = make();
await = 9;
ready.then(C => {
  var value = new C();
  print(C.value + ':' + C.other + ':' + C.read());
  print(value.value + ':' + value.item + ':' + value.read());
});
"#,
        &["7:7:7:7", "9:9:9:9"],
    );
}

#[test]
fn computed_keys_and_nested_functions_suspend_in_their_own_contexts() {
    assert_trace(
        r#"
var await = 11;
function* make() {
  return class {
    [yield 'key'] = await;
    nested = async () => await Promise.resolve(13);
    generator = function* () { yield 17; };
  };
}
var iterator = make();
print(iterator.next().value);
var C = iterator.next('value').value;
var value = new C();
print(value.value + ':' + value.generator().next().value);
async function run() {
  class D { [await Promise.resolve('value')] = 19; }
  print((await value.nested()) + ':' + new D().value);
}
run();
"#,
        &["key", "11:17", "13:19"],
    );
}
