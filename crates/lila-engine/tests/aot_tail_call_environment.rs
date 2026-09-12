use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_tail_calls(source: &str, host_surface_policy: HostSurfacePolicy) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions {
                host_surface_policy,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("tail-call execution failed: {error}\n{source}"));
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn dynamically_introduced_eval_binding_tail_calls_one_hundred_thousand_times() {
    assert_tail_calls(
        r#"
var callCount = 0;
(function() {
  function f(n) {
    "use strict";
    if (n === 0) { callCount += 1; return; }
    return eval(n - 1);
  }
  eval("var eval = f;");
  f(100000);
})();
callCount === 1;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn environment_calls_keep_tail_position_through_nested_return_expressions() {
    assert_tail_calls(
        r#"
(function() {
  var visits = 0;
  var terminal = 'done';
  function f(n) {
    "use strict";
    return n === 0 ? terminal : (visits++, null ?? (false || (true && eval(n - 1))));
  }
  eval("var eval = f;");
  if (f(100000) !== 'done' || visits !== 100000) throw new Error('nested tail position');

  // The left operand of ?? must keep the continuation that checks its result.
  var fallbacks = 0;
  function afterCall(n) {
    "use strict";
    return n === 0 ? terminal : (visits++, (false || (true && eval(n - 1))) ?? (++fallbacks, 'fallback'));
  }
  terminal = null;
  if (afterCall(1) !== 'fallback' || fallbacks !== 1)
    throw new Error('nullish call result lost fallback');
  terminal = 0;
  if (afterCall(1) !== 0 || fallbacks !== 1)
    throw new Error('falsy call result must skip nullish fallback');
  if (visits !== 100002) throw new Error('call continuation repeated effects');
})();
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn captured_object_environment_preserves_the_selected_receiver_and_get_order() {
    assert_tail_calls(
        r#"
var gets = 0;
var argumentReads = 0;
var target;
var scope = {
  get eval() { gets++; return target; },
  get next() { argumentReads++; return 0; }
};
with (scope) {
  target = function(n) {
    "use strict";
    if (this !== scope) throw new Error('lost environment receiver');
    if (n === 0) return 'done';
    return eval(n === 1 ? next : n - 1);
  };
}
if (scope.eval(100000) !== 'done' || gets !== 100001 || argumentReads !== 1)
  throw new Error('environment call order');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn proxy_forwarding_and_apply_traps_do_not_retain_dispatcher_frames() {
    assert_tail_calls(
        r#"
var plainProxy;
var trappedProxy;
var traps = 0;
function plain(n) {
  "use strict";
  return n === 0 ? 'plain' : plainProxy(n - 1);
}
function trapped(n) {
  "use strict";
  return n === 0 ? 'trapped' : trappedProxy(n - 1);
}
plainProxy = new Proxy(plain, {});
const handler = {
  apply(target, receiver, args) {
    "use strict";
    if (this !== handler || receiver !== undefined) throw new Error('proxy call receiver');
    traps++;
    return target(args[0]);
  }
};
trappedProxy = new Proxy(trapped, handler);
if (plain(100000) !== 'plain' || trapped(100000) !== 'trapped' || traps !== 100000)
  throw new Error('proxy tail forwarding');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn genuine_eval_keeps_its_caller_and_replacement_calls_hold_the_original_callee() {
    assert_tail_calls(
        r#"
const marker = {};
const receiver = {
  read() {
    "use strict";
    let local = 23;
    return eval('this === receiver && local === 23 && new.target === undefined');
  },
  identity() { "use strict"; return eval(marker); }
};
if (receiver.read() !== true || receiver.identity() !== marker) throw new Error('direct eval caller');
(function() {
  var originalCalls = 0;
  var replacements = 0;
  function replacement() { replacements++; return 'wrong'; }
  function changeBinding() { eval = replacement; return 0; }
  function f(n) {
    "use strict";
    if (n === 0) { originalCalls++; return marker; }
    return eval(...[changeBinding()]);
  }
  eval('var eval = f;');
  if (f(1) !== marker || originalCalls !== 1 || replacements !== 0)
    throw new Error('callee changed during argument evaluation');
})();
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn active_catch_finally_and_disposal_keep_their_post_call_work() {
    assert_tail_calls(
        r#"
const marker = {};
let trace = '';
function target() { throw marker; }
function caught() {
  "use strict";
  try { return target(); } catch (error) { trace += 'c'; return error; }
}
function finalized() {
  "use strict";
  try { return target(); } finally { trace += 'f'; }
}
function disposed() {
  "use strict";
  using resource = { [Symbol.dispose]() { trace += 'd'; } };
  return target();
}
if (caught() !== marker) throw new Error('active catch');
for (const operation of [finalized, disposed]) {
  let received;
  try { operation(); } catch (error) { received = error; }
  if (received !== marker) throw new Error('post-call throw identity');
}
if (trace !== 'cfd') throw new Error('post-call cleanup order');
function argumentThrow() { trace += 'a'; throw marker; }
function returning() { "use strict"; return target(argumentThrow()); }
let received;
try { returning(); } catch (error) { received = error; }
if (received !== marker || trace !== 'cfda') throw new Error('argument throw identity');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn foreign_tail_calls_keep_the_calling_realm_for_non_callable_errors() {
    assert_tail_calls(
        r#"
const other = __lilaCreateRealm().global;
const tail = other.eval("(function(callee) { 'use strict'; return callee(); })");
let received;
try { tail(1); } catch (error) { received = error; }
if (received === undefined || Object.getPrototypeOf(received) !== other.TypeError.prototype)
  throw new Error('tail call error realm');
const marker = {};
function throws() { throw marker; }
received = undefined;
try { tail(throws); } catch (error) { received = error; }
if (received !== marker) throw new Error('tail callee throw identity');
true;
"#,
        HostSurfacePolicy::Test262,
    );
}
