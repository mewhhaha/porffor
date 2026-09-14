use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_constructor_error_realms(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observation = Engine::new(RealmBuilder::new().build())
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
        .expect("typed-array constructor errors execute through Wasm AOT");
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion,
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string())],
        "{source}",
    );
}

#[test]
fn all_typed_array_constructors_use_their_realm_for_call_and_iterator_errors() {
    assert_constructor_error_realms(
        r#"
var foreign = __lilaCreateRealm().global;
var names = ['Float16Array','Float32Array','Float64Array','Int8Array','Int16Array',
  'Int32Array','Uint8Array','Uint8ClampedArray','Uint16Array','Uint32Array',
  'BigInt64Array','BigUint64Array'];
var realms = [globalThis, foreign];
for (var realm of realms) {
  for (var name of names) {
    var constructor = realm[name];
    var prototype = realm.TypeError.prototype;
    var calls = [function(){ constructor(0); }];
    for (var callable of [false, true]) {
      for (var iterator of [1, function(){return 1;}, function(){return {next:1};},
          function(){return {next:function(){return 1;}};}]) {
        let source = callable ? function(){} : {};
        source[Symbol.iterator] = iterator;
        calls.push(function(){ new constructor(source); });
      }
    }
    for (var call of calls) {
      var caught;
      try { call(); } catch (error) { caught = error; }
      if (!caught || Object.getPrototypeOf(caught) !== prototype) throw name + ' error Realm';
    }
  }
}
print(true);
"#,
    );
}

#[test]
fn missing_new_precedes_observation_and_iterator_abrupt_values_keep_identity() {
    assert_constructor_error_realms(
        r#"
var foreign = __lilaCreateRealm().global;
var marker = Symbol('iterator');
var trace = '';
var source = {};
Object.defineProperty(source, Symbol.iterator, {get:function(){trace += 'get'; throw marker;}});
var constructor = foreign.Float16Array;
var errorPrototype = foreign.TypeError.prototype;
try { constructor(source); throw 'accepted call'; }
catch (error) { if (Object.getPrototypeOf(error) !== foreign.TypeError.prototype) throw error; }
if (trace !== '') throw 'observed source without new';
try { new constructor(source); throw 'lost getter abrupt'; }
catch (error) { if (error !== marker) throw error; }
if (trace !== 'get') throw 'getter order';
try { new constructor({[Symbol.iterator]:function(){throw marker;}}); throw 'lost call abrupt'; }
catch (error) { if (error !== marker) throw error; }
var LocalHalf = Float16Array;
Object.setPrototypeOf(constructor, null);
foreign.TypeError = null;
foreign.Float16Array = null;
try { Reflect.apply(constructor, null, []); throw 'accepted mutated constructor'; }
catch (error) { if (Object.getPrototypeOf(error) !== errorPrototype) throw 'mutable Realm lookup'; }
if (new LocalHalf([1.5])[0] !== 1.5 || new constructor([2])[0] !== 2) throw 'ordinary construction';
print(true);
"#,
    );
}
