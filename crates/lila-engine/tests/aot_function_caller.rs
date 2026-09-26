use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

#[test]
fn legacy_caller_and_arguments_track_ordinary_sloppy_activations() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = r#"
function inner() { return inner.caller; }
function outer() { return inner(); }
function strictOuter() { "use strict"; return outer(); }
function strictImmediate() { "use strict"; return inner(); }
var descriptor = Object.getOwnPropertyDescriptor(inner, 'caller');
var argumentsDescriptor = Object.getOwnPropertyDescriptor(inner, 'arguments');
print(descriptor.value === null && !descriptor.writable && !descriptor.enumerable && !descriptor.configurable
  && argumentsDescriptor.value === null && !argumentsDescriptor.writable
  && !argumentsDescriptor.enumerable && !argumentsDescriptor.configurable);
print(inner.caller === null && outer() === outer && strictOuter() === outer && strictImmediate() === null);
print(outer.call(null) === outer && outer.apply(null) === outer && outer.bind(null)() === outer);
print(Function().caller === null);
function argumentsProbe(value) {
  return argumentsProbe.arguments === arguments && argumentsProbe.arguments[0] === value
    && Object.getOwnPropertyDescriptor(argumentsProbe, 'arguments').value === arguments;
}
print(argumentsProbe(5) && argumentsProbe.arguments === null);
function namedParameter(arguments) {
  return arguments === 7 && namedParameter.arguments[0] === 7
    && namedParameter.arguments !== arguments;
}
function defaultParameter(arguments = 5) {
  return arguments === 5 && defaultParameter.arguments.length === 0;
}
function restParameter(...arguments) {
  return arguments[0] === 9 && restParameter.arguments[0] === 9
    && restParameter.arguments !== arguments;
}
function reassignedArguments(value) {
  var original = reassignedArguments.arguments;
  arguments = "changed";
  return reassignedArguments.arguments === original && original[0] === value;
}
function localFunctionShadow() {
  function arguments() {}
  return typeof arguments === "function" && localFunctionShadow.arguments.length === 0;
}
print(namedParameter(7) && defaultParameter() && restParameter(9)
  && reassignedArguments(11) && localFunctionShadow());
function recursive(depth) {
  if (depth === 2) return recursive.arguments[0] === 2 && recursive.caller === recursive;
  var outerArguments = recursive.arguments;
  return recursive(2) && recursive.arguments === outerArguments && recursive.caller === null;
}
print(recursive(1) && recursive.arguments === null);
function throwing() { throw 1; }
function catchThrow() {
  try { throwing(); } catch (error) {
    return error === 1 && throwing.arguments === null && throwing.caller === null
      && catchThrow.arguments !== null;
  }
  return false;
}
print(catchThrow() && catchThrow.arguments === null);
function parameterCaller() { return parameterCaller.caller; }
function parameterOwner(value = parameterCaller()) {
  return value === parameterOwner && parameterOwner.arguments[0] === undefined;
}
print(parameterOwner() && parameterOwner.arguments === null);
function descriptorInner() { return Object.getOwnPropertyDescriptor(descriptorInner, 'caller').value; }
function descriptorOuter() { return descriptorInner() === descriptorOuter; }
print(descriptorOuter() && Object.getOwnPropertyDescriptor(descriptorInner, 'caller').value === null);
function callback() { return callback.caller; }
function nativeOuter() { return [0].map(callback)[0] === null && Reflect.apply(callback, null, []) === null; }
print(nativeOuter());
var boundCallback = callback.bind(null);
function boundOuter() { return boundCallback() === boundOuter; }
print(boundOuter());
function generatorChild() { return generatorChild.caller; }
function* generatorBody() { yield generatorChild(); }
function generatorOuter() { return generatorBody().next().value === null && generatorOuter.caller === null; }
print(generatorOuter());
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
        vec![HostOutputEvent::PrintLine("true".to_string()); 17]
    );
}
