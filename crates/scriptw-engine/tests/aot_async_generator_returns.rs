use scriptw_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn run_both_modes(body: &str) {
    scriptw_engine::configure_compilation_jobs(1).unwrap();
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{body}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
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
            .unwrap();
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "mode {directive:?}: {:?}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine("async-return:ok".into())],
            "mode {directive:?}"
        );
    }
}

#[test]
fn pinned_implicit_and_explicit_undefined_return_tick_order_in_both_modes() {
    const BODY: &str = include_str!("../../../test262/vendor/test262/test/language/statements/async-generator/return-undefined-implicit-and-explicit.js");
    const STA: &str = include_str!("../../../test262/vendor/test262/harness/sta.js");
    const ASSERT: &str = include_str!("../../../test262/vendor/test262/harness/assert.js");
    const COMPARE: &str = include_str!("../../../test262/vendor/test262/harness/compareArray.js");
    run_both_modes(&format!(
        "{STA}\n{ASSERT}\n{COMPARE}\nfunction $DONE(error) {{ print(error === undefined ? 'async-return:ok' : 'async-return:fail'); }}\n{BODY}"
    ));
}

#[test]
fn body_completion_keeps_expression_protocol_thenable_and_finally_await_owners() {
    run_both_modes(
        r#"
function check(value) { if (!value) throw new Error('async return completion'); }
function same(actual, expected) { check(actual.join('|') === expected.join('|')); }
async function* bare() { return; }
async function* shadow(undefined) { return undefined; }
async function* finalizer() { try { return; } finally { await 0; } }
async function* unfinished() { try { yield 1; } finally { yield 2; } }
async function main() {
  const order = [];
  const ticks = Promise.resolve().then(() => order.push('tick1'))
    .then(() => order.push('tick2')).then(() => order.push('tick3'));
  const direct = bare().next().then(r => { check(r.done && r.value === undefined); order.push('body'); });
  const protocol = bare().return(undefined).then(r => { check(r.done && r.value === undefined); order.push('protocol'); });
  const restored = finalizer().next().then(r => { check(r.done && r.value === undefined); order.push('finally'); });
  const explicit = shadow(void 0).next().then(r => { check(r.done && r.value === undefined); order.push('shadow'); });
  await Promise.all([ticks, direct, protocol, restored, explicit]);
  same(order, ['tick1', 'body', 'tick2', 'protocol', 'finally', 'shadow', 'tick3']);

  const events = [];
  let proxy;
  proxy = new Proxy({then(resolve) { check(this === proxy); events.push('call'); resolve(17); }}, {
    get(target, key, receiver) { if (key === 'then') events.push('get'); return Reflect.get(target, key, receiver); }
  });
  const pending = shadow(proxy).next();
  same(events, ['get']);
  const result = await pending;
  check(result.done && result.value === 17);
  same(events, ['get', 'call']);

  events.length = 0;
  const suspended = unfinished();
  check((await suspended.next()).value === 1);
  const returning = suspended.return(proxy);
  same(events, ['get']);
  const middle = await returning;
  check(!middle.done && middle.value === 2);
  const end = await suspended.next();
  check(end.done && end.value === 17);
  same(events, ['get', 'call']);

  events.length = 0;
  const marker = {};
  const throwing = new Proxy({}, {get(target, key) {
    if (key === 'then') { events.push('get:throw'); throw marker; }
    return target[key];
  }});
  async function* rejected() { try { return throwing; } finally { events.push('finally'); } }
  const rejection = rejected().next();
  events.push('after-call');
  try { await rejection; throw new Error('return unexpectedly fulfilled'); }
  catch (error) { check(error === marker); }
  same(events, ['get:throw', 'after-call', 'finally']);

  const incomplete = unfinished();
  await incomplete.next();
  const yielded = await incomplete.return(undefined);
  check(!yielded.done && yielded.value === 2);
  const completed = await incomplete.next();
  check(completed.done && completed.value === undefined);
}
main().then(() => print('async-return:ok'), () => print('async-return:fail'));
"#,
    );
}
