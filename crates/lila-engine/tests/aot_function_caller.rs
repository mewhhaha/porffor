use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

#[test]
fn legacy_caller_is_null_only_on_ordinary_sloppy_functions() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = r#"
function inner() { return inner.caller; }
function outer() { return inner(); }
function strictOuter() { "use strict"; return outer(); }
var descriptor = Object.getOwnPropertyDescriptor(inner, 'caller');
print(descriptor.value === null && !descriptor.writable && !descriptor.enumerable && !descriptor.configurable);
print(inner.caller === null && outer() === null && strictOuter() === null);
print(outer.call(null) === null && outer.apply(null) === null && outer.bind(null)() === null);
print(Function().caller === null);
function strictFunction() { "use strict"; }
var object = { method() {}, get accessor() {} };
var restricted = [
  strictFunction, () => {}, function* () {}, async function () {}, async () => {}, async function* () {},
  object.method, Object.getOwnPropertyDescriptor(object, 'accessor').get, class C {}, inner.bind(null),
  Function, Array, print
];
var absent = 0;
var throws = 0;
for (var index = 0; index < restricted.length; index++) {
  if (!Object.prototype.hasOwnProperty.call(restricted[index], 'caller')) absent++;
  try { restricted[index].caller; } catch (error) { if (error instanceof TypeError) throws++; }
}
print(absent === restricted.length && throws === restricted.length);
var poison = Object.getOwnPropertyDescriptor(Function.prototype, 'caller');
print(typeof poison.get === 'function' && poison.get === poison.set
  && !poison.enumerable && poison.configurable);
try { Function.prototype.caller; } catch (error) { print(error instanceof TypeError); }
try { Object.defineProperty(inner, 'caller', { value: strictFunction }); }
catch (error) { print(error instanceof TypeError && inner.caller === null); }
void 0;
"#;
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
        .expect("caller property policy executes through Wasm AOT");
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion
    );
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string()); 8]
    );
}
