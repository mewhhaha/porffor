use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, RealmBuilder, RunOptions,
};
use lila_runtime::ModuleLoadingPolicy;

fn assert_script(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &format!("{directive}{source}"),
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    module_loading_policy: ModuleLoadingPolicy::RejectAll,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("computed Script imports execute through the compiled graph");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{}; output events: {:?}",
            observed.note,
            observed.output_events
        );
        assert_eq!(
            observed.output_events,
            expected
                .iter()
                .map(|line| HostOutputEvent::PrintLine((*line).into()))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn abrupt_get_value_is_synchronous_and_precedes_options_evaluation() {
    assert_script(
        r#"
var marker = {}, events = [];
var holder = { get specifier() { events.push('get'); throw marker; } };
var caught;
try { import(holder.specifier, (events.push('options'), {})); }
catch (error) { caught = error; }
if (caught !== marker || events.join(',') !== 'get') throw 'GetValue order';
var reference;
try { import(unboundSpecifier, (events.push('unreached'), {})); }
catch (error) { reference = error; }
if (!(reference instanceof ReferenceError) || events.join(',') !== 'get') throw 'unbound GetValue';
print('synchronous operand failures');
true;
"#,
        &["synchronous operand failures"],
    );
}

#[test]
fn tostring_abrupt_rejects_after_both_operands_without_reading_attributes() {
    assert_script(
        r#"
var marker = {}, events = [];
var specifier = { toString() { events.push('coerce'); throw marker; } };
var options = { get with() { events.push('with'); return {}; } };
var promise;
try { promise = import((events.push('specifier'), specifier), (events.push('options'), options)); }
catch (error) { throw 'coercion must reject'; }
if (!(promise instanceof Promise) || events.join(',') !== 'specifier,options,coerce') throw 'operand order';
promise.then(() => { throw 'fulfilled abrupt coercion'; }, error => {
  if (error !== marker || events.join(',') !== 'specifier,options,coerce') throw 'rejection identity';
  print('coercion rejection');
});
print('Script body');
true;
"#,
        &["Script body", "coercion rejection"],
    );
}

#[test]
fn unknown_computed_targets_get_fresh_promises_and_asynchronous_rejections() {
    assert_script(
        r#"
var events = [], specifier = './never-loaded.js';
var options = { get with() { events.push('with'); return {}; } };
var first, second;
try { first = import((events.push('specifier'), specifier), (events.push('options'), options)); }
catch (error) { throw 'unknown target must reject'; }
second = import(specifier);
if (!(first instanceof Promise) || !(second instanceof Promise) || first === second) throw 'fresh promises';
if (events.join(',') !== 'specifier,options,with') throw 'attribute order';
first.then(() => { throw 'unknown target fulfilled'; }, error => {
  if (!(error instanceof TypeError) || events.join(',') !== 'specifier,options,with,body') throw 'first rejection';
  print('first rejection');
});
second.then(() => { throw 'second target fulfilled'; }, error => {
  if (!(error instanceof TypeError)) throw 'second rejection';
  print('second rejection');
});
events.push('body');
print('Script body');
true;
"#,
        &["Script body", "first rejection", "second rejection"],
    );
}

#[test]
fn object_and_class_methods_named_import_keep_ordinary_script_semantics() {
    assert_script(
        r#"
var object = { import(value) { return value + 1; } };
class Holder { import(value) { return value + 2; } }
if (object.import(3) !== 4 || new Holder().import(3) !== 5) throw 'ordinary methods';
print('ordinary import methods');
true;
"#,
        &["ordinary import methods"],
    );
}
