use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn run(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observed = Engine::new(RealmBuilder::new().build())
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
        .expect("constructor throws execute through Wasm AOT");
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observed.completion
    );
    assert_eq!(
        observed.output_events,
        [HostOutputEvent::PrintLine("ok".into())]
    );
}

#[test]
fn foreign_constructor_errors_keep_object_reads_after_a_string_sentinel() {
    run(r#"
const other = __lilaCreateRealm().global;
let later = 0;
try {
  new other.Intl.NumberFormat(null, {get style() { later++; }});
  throw 'missing constructor throw';
} catch (error) {
  if (error.name !== 'TypeError' || typeof error.message !== 'string') throw 'lost error properties';
  if (error.constructor !== other.TypeError || Object.getPrototypeOf(error) !== other.TypeError.prototype) throw 'lost foreign error identity';
}
if (later !== 0) throw 'observed options after failed locales';
try { new Array(-1); throw 'missing RangeError'; }
catch (error) { if (error.name !== 'RangeError' || error.constructor !== RangeError) throw 'lost precise error control'; }
print('ok');
"#);
}

#[test]
fn constructor_bodies_can_throw_every_language_value() {
    run(r#"
const marker = {answer: 42};
const symbol = Symbol('reason');
for (const reason of [undefined, null, false, 3, 4n, symbol, marker]) {
  function ThrowReason() { throw reason; }
  let reached = false;
  try { new ThrowReason(); reached = true; throw 'missing constructor throw'; }
  catch (error) { if (error !== reason) throw 'replaced constructor throw'; }
  if (reached) throw 'continued after constructor throw';
}
function ThrowObject() { throw marker; }
try { new ThrowObject(); throw 'missing constructor throw'; }
catch (error) { if (error.answer !== 42 || error !== marker) throw 'specialized object as string'; }
print('ok');
"#);
}

#[test]
fn constructor_hooks_preserve_arbitrary_abrupt_values() {
    run(r#"
const marker = {answer: 42};
const ThrowProxy = new Proxy(function () {}, {construct() { throw marker; }});
try { new ThrowProxy(); throw 1; }
catch (error) { if (error !== marker || error.answer !== 42) throw 'lost construct trap throw'; }
let later = false;
try {
  new Intl.NumberFormat('en-US', {get style() { throw undefined; }, get currency() { later=true; }});
  throw 'missing option throw';
} catch (error) { if (error !== undefined) throw 'lost undefined option throw'; }
if (later) throw 'continued options after throw';
print('ok');
"#);
}
