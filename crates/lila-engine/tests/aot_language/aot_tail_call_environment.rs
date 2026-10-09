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
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
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
let disposalTrace = '';
const disposalError = {};
function returnsNormally() { disposalTrace += 'call;'; return marker; }
function normalDisposed() {
  "use strict";
  using resource = { [Symbol.dispose]() { disposalTrace += 'dispose;'; } };
  return returnsNormally();
}
if (normalDisposed() !== marker || disposalTrace !== 'call;dispose;')
  throw new Error('normal return before disposal');
function disposalReplacesReturn() {
  "use strict";
  using resource = { [Symbol.dispose]() { disposalTrace += 'throw;'; throw disposalError; } };
  return returnsNormally();
}
received = undefined;
try { disposalReplacesReturn(); } catch (error) { received = error; }
if (received !== disposalError || disposalTrace !== 'call;dispose;call;throw;')
  throw new Error('disposal error replaces normal return');
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

#[test]
fn ordinary_call_forms_and_intrinsic_forwarders_tail_call_one_hundred_thousand_times() {
    assert_tail_calls(
        r#"
const marker = {};
const receiver = {};
const holder = { step };
const bound = step.bind(receiver);
const expected = [undefined, undefined, holder, holder, receiver, receiver, receiver, receiver];
function step(n, route) {
  "use strict";
  if (n === 0) return this === expected[route] ? marker : null;
  if (route === 0) return step(n - 1, route);
  if (route === 1) return (0, step)(n - 1, route);
  if (route === 2) return holder.step(n - 1, route);
  if (route === 3) return holder.step?.(n - 1, route);
  if (route === 4) return bound(n - 1, route);
  if (route === 5) return step.call(receiver, n - 1, route);
  if (route === 6) return step.apply(receiver, [n - 1, route]);
  return Reflect.apply(step, receiver, [n - 1, route]);
}
for (let route = 0; route < expected.length; route++) {
  if (step(100000, route) !== marker) throw new Error('ordinary tail form ' + route);
}
let argumentsRead = 0;
function optionalAbsent() { "use strict"; return holder.absent?.(argumentsRead++); }
if (optionalAbsent() !== undefined || argumentsRead !== 0) throw new Error('optional short circuit');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn tail_results_preserve_native_values_source_fallthrough_and_constructor_continuations() {
    assert_tail_calls(
        r#"
const marker = {};
const map = new Map();
function nativeNumber() { "use strict"; return Number('42'); }
function nativeString() { "use strict"; return String(42); }
function nativeObject() { "use strict"; return Object(marker); }
function nativeUndefined() { "use strict"; return map.clear(); }
function sourceObject() { "use strict"; return marker; }
function fallthrough() { "use strict"; Object(marker); 99; }
function forward(target) { "use strict"; return target(); }
if (forward(nativeNumber) !== 42 || forward(nativeString) !== '42'
    || forward(nativeObject) !== marker || forward(nativeUndefined) !== undefined
    || forward(sourceObject) !== marker || forward(fallthrough) !== undefined)
  throw new Error('normal Call value');
function BaseObject() { "use strict"; return Object(marker); }
function BasePrimitive() { "use strict"; this.created = true; return Number('7'); }
function BaseFallthrough() { "use strict"; this.created = true; Object(marker); }
class BaseClass { constructor() { return Object(marker); } }
class DerivedObject extends Object { constructor() { return Object(marker); } }
class DerivedPrimitive extends Object { constructor() { return Number('7'); } }
class DerivedUndefined extends Object {
  constructor() { super(); this.created = true; return map.clear(); }
}
class MissingThis extends Object { constructor() { return map.clear(); } }
if (new BaseObject() !== marker || new BaseClass() !== marker || new DerivedObject() !== marker)
  throw new Error('object constructor return');
if (new BasePrimitive().created !== true || new BaseFallthrough().created !== true
    || new DerivedUndefined().created !== true)
  throw new Error('constructor this continuation');
for (const [Constructor, ErrorType] of [[DerivedPrimitive, TypeError], [MissingThis, ReferenceError]]) {
  let error;
  try { new Constructor(); } catch (caught) { error = caught; }
  if (error === undefined || Object.getPrototypeOf(error) !== ErrorType.prototype)
    throw new Error('derived constructor validation');
}
let constructs = 0;
const Wrapped = new Proxy(BasePrimitive, { construct(target, args, newTarget) {
  constructs++;
  return Reflect.construct(target, args, newTarget);
} });
if (new Wrapped().created !== true || constructs !== 1) throw new Error('construct trap continuation');
const Invalid = new Proxy(BaseObject, { construct() { return nativeNumber(); } });
let invalidError;
try { new Invalid(); } catch (error) { invalidError = error; }
if (invalidError === undefined || Object.getPrototypeOf(invalidError) !== TypeError.prototype)
  throw new Error('construct trap object validation');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn normal_call_boundaries_restore_the_caller_realm_after_tail_success_and_throw() {
    assert_tail_calls(
        r#"
const other = __lilaCreateRealm().global;
const foreignTail = other.eval("(function(target, argument) { 'use strict'; return target(argument); })");
const foreignObject = other.eval("(function() { 'use strict'; return Object(); })");
if (Object.getPrototypeOf(foreignTail(foreignObject)) !== other.Object.prototype)
  throw new Error('tail callee realm');
if (Object.getPrototypeOf({}) !== Object.prototype) throw new Error('normal caller realm');
const marker = {};
function throws() { throw marker; }
let received;
try { foreignTail(throws); } catch (error) { received = error; }
if (received !== marker || Object.getPrototypeOf({}) !== Object.prototype)
  throw new Error('throw caller realm');
received = undefined;
try { foreignTail(other.Number, Symbol()); } catch (error) { received = error; }
if (received === undefined || Object.getPrototypeOf(received) !== other.TypeError.prototype)
  throw new Error('native tail error realm');
if (Object.getPrototypeOf({}) !== Object.prototype) throw new Error('native throw caller realm');
received = undefined;
try { foreignTail(Number, Symbol()); } catch (error) { received = error; }
if (received === undefined || Object.getPrototypeOf(received) !== TypeError.prototype)
  throw new Error('local native error realm');
if (Object.getPrototypeOf({}) !== Object.prototype) throw new Error('local throw caller realm');
true;
"#,
        HostSurfacePolicy::Test262,
    );
}
