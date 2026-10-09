use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn created_realm_arrays_publish_their_values_method_as_the_default_iterator() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = r#"
var realm = __lilaCreateRealm();
var other = realm.global;
var second = __lilaCreateRealm().global;
var descriptor = Object.getOwnPropertyDescriptor(other.Array.prototype, Symbol.iterator);
print(descriptor.value === other.Array.prototype.values);
print(descriptor.writable && !descriptor.enumerable && descriptor.configurable);
print(descriptor.value !== Array.prototype.values
  && descriptor.value !== second.Array.prototype.values);
print(Object.getPrototypeOf(descriptor.value) === other.Function.prototype);
print(!Object.prototype.hasOwnProperty.call(other.Array.prototype, 'Symbol.iterator'));
var iterator = descriptor.value.call([3, 5]);
var first = iterator.next();
var last = iterator.next();
print(first.value === 3 && first.done === false
  && last.value === 5 && last.done === false && iterator.next().done === true);
print(realm.evalScript('var sum = 0; for (var value of [3, 5]) { sum += value; } sum;') === 8);
var foreign = realm.evalScript('[7, 11];');
var sum = 0;
for (var value of foreign) { sum += value; }
print(sum === 18);
void 0;
"#;
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("created-realm array iteration executes through Wasm");
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion,
    );
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string()); 8],
    );
}
