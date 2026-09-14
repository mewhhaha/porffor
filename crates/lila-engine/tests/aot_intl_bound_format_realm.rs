use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_bound_format_realm(source: &str) {
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
        .expect("Intl bound format Realm regression must execute through Wasm AOT");
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
fn bound_format_uses_its_getters_realm_and_preserves_function_descriptors() {
    assert_bound_format_realm(
        r#"
var foreign = __lilaCreateRealm().global;
var constructors = [Intl.DateTimeFormat, foreign.Intl.DateTimeFormat];
var prototypes = [Function.prototype, foreign.Function.prototype];
for (var i = 0; i < constructors.length; i++) {
  var formatter = new constructors[i]('en', {timeZone:'UTC'});
  var format = formatter.format;
  if (Object.getPrototypeOf(format) !== prototypes[i] || format !== formatter.format) throw 'Realm or cache';
  if (format.name !== '' || format.length !== 1 || Object.prototype.hasOwnProperty.call(format,'prototype')) throw 'function shape';
  for (var name of ['name','length']) {
    var descriptor = Object.getOwnPropertyDescriptor(format,name);
    if (descriptor.writable || descriptor.enumerable || !descriptor.configurable) throw 'function descriptor';
  }
  var output = format(0);
  if (typeof output !== 'string' || output.length === 0 || format.call(null,0) !== output || format.call({},0) !== output) throw 'captured formatter';
  try { new format(0); throw 'accepted construction'; }
  catch (error) { if (!(error instanceof TypeError)) throw error; }
}
print(true);
"#,
    );
}

#[test]
fn borrowed_format_getter_realm_wins_only_on_first_materialization() {
    assert_bound_format_realm(
        r#"
var foreign = __lilaCreateRealm().global;
var third = __lilaCreateRealm().global;
var localGetter = Object.getOwnPropertyDescriptor(Intl.DateTimeFormat.prototype,'format').get;
var foreignGetter = Object.getOwnPropertyDescriptor(foreign.Intl.DateTimeFormat.prototype,'format').get;
var thirdGetter = Object.getOwnPropertyDescriptor(third.Intl.DateTimeFormat.prototype,'format').get;
var localFormatter = new Intl.DateTimeFormat('en', {timeZone:'UTC'});
var foreignFormatter = new foreign.Intl.DateTimeFormat('en', {timeZone:'UTC'});
var foreignFormat = foreignGetter.call(localFormatter);
var localFormat = localGetter.call(foreignFormatter);
if (Object.getPrototypeOf(foreignFormat) !== foreign.Function.prototype ||
    Object.getPrototypeOf(localFormat) !== Function.prototype) throw 'borrowed getter Realm';
if (localGetter.call(localFormatter) !== foreignFormat ||
    thirdGetter.call(localFormatter) !== foreignFormat ||
    foreignGetter.call(foreignFormatter) !== localFormat ||
    thirdGetter.call(foreignFormatter) !== localFormat) throw 'cached identity changed';
if (foreignFormat(0) !== localFormat(0)) throw 'borrowed capture';
var fresh = new Intl.DateTimeFormat('en', {timeZone:'UTC'});
var canonical = third.Function.prototype;
Object.setPrototypeOf(thirdGetter, null);
third.Intl = null;
third.Function = null;
var format = Reflect.apply(thirdGetter, fresh, []);
if (Object.getPrototypeOf(format) !== canonical || typeof format(0) !== 'string') throw 'mutable public Realm source';
print(true);
"#,
    );
}

#[test]
fn bound_format_errors_and_abrupt_coercion_use_the_cached_function_realm() {
    assert_bound_format_realm(
        r#"
var foreign = __lilaCreateRealm().global;
var localGetter = Object.getOwnPropertyDescriptor(Intl.DateTimeFormat.prototype,'format').get;
var foreignGetter = Object.getOwnPropertyDescriptor(foreign.Intl.DateTimeFormat.prototype,'format').get;
var formatter = new Intl.DateTimeFormat('en', {timeZone:'UTC'});
var foreignFormat = foreignGetter.call(formatter);
var localFormat = localGetter.call(new foreign.Intl.DateTimeFormat('en', {timeZone:'UTC'}));
if (localGetter.call(formatter) !== foreignFormat) throw 'cache';
function throwsWithPrototype(format, input, prototype) {
  try { format(input); throw 'accepted invalid input'; }
  catch (error) { if (Object.getPrototypeOf(error) !== prototype) throw 'error Realm'; }
}
for (var input of [NaN, Infinity, -Infinity]) {
  throwsWithPrototype(foreignFormat, input, foreign.RangeError.prototype);
  throwsWithPrototype(localFormat, input, RangeError.prototype);
}
for (var input of [Symbol('time'), 1n, {[Symbol.toPrimitive]: function(){return Symbol('time');}}]) {
  throwsWithPrototype(foreignFormat, input, foreign.TypeError.prototype);
  throwsWithPrototype(localFormat, input, TypeError.prototype);
}
var marker = Symbol('coercion');
var trace = '';
var abrupt = {[Symbol.toPrimitive]: function(hint){trace += hint; throw marker;}};
try { foreignFormat(abrupt); throw 'lost abrupt completion'; }
catch (error) { if (error !== marker) throw 'changed thrown value'; }
if (trace !== 'number' || typeof foreignFormat(0) !== 'string') throw 'coercion order or retained capture';
print(true);
"#,
    );
}
