use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn run_boolean(source: &str) {
    run_boolean_with_options(source, CompileOptions::default());
}

fn run_boolean_with_realms(source: &str) {
    run_boolean_with_options(
        source,
        CompileOptions {
            host_surface_policy: HostSurfacePolicy::Test262,
            ..CompileOptions::default()
        },
    );
}

fn run_boolean_with_options(source: &str, options: CompileOptions) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            source,
            options,
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("callable Proxy bind should execute through Wasm");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn bound_foreign_targets_use_the_local_callers_realm_for_proxy_dispatch() {
    run_boolean_with_realms(
        r#"
var other = __lilaCreateRealm().global;
var callArrayIsLocal = false;
var constructArrayIsLocal = false;
var callProxy = new Proxy(other.Math.sin, {
  apply: function (target, receiver, args) {
    callArrayIsLocal = Object.getPrototypeOf(args) === Array.prototype;
    return 23;
  }
});
var constructProxy = new Proxy(other.Array, {
  construct: function (target, args, newTarget) {
    constructArrayIsLocal = Object.getPrototypeOf(args) === Array.prototype;
    return { count: args.length };
  }
});
var callBound = Function.prototype.bind.call(callProxy, null);
var constructBound = Function.prototype.bind.call(constructProxy, null);
var called = callBound() === 23;
var made = new constructBound(1);
var revokedCall = Proxy.revocable(other.Math.sin, {});
var revokedCallBound = Function.prototype.bind.call(revokedCall.proxy, null);
revokedCall.revoke();
var callError;
try { revokedCallBound(); } catch (error) { callError = error; }
var revokedConstruct = Proxy.revocable(other.Array, {});
var revokedConstructBound = Function.prototype.bind.call(revokedConstruct.proxy, null);
revokedConstruct.revoke();
var constructError;
try { new revokedConstructBound(); } catch (error) { constructError = error; }
called && made.count === 1 && callArrayIsLocal && constructArrayIsLocal
  && Object.getPrototypeOf(callError) === TypeError.prototype
  && Object.getPrototypeOf(constructError) === TypeError.prototype
  && !(callError instanceof other.TypeError)
  && !(constructError instanceof other.TypeError);
"#,
    );
}

#[test]
fn foreign_reflect_callers_keep_their_realm_through_bound_proxy_dispatch() {
    run_boolean_with_realms(
        r#"
var other = __lilaCreateRealm().global;
var callArrayIsForeign = false;
var constructArrayIsForeign = false;
var callProxy = new Proxy(function () {}, {
  apply: function (target, receiver, args) {
    callArrayIsForeign = Object.getPrototypeOf(args) === other.Array.prototype;
    return 29;
  }
});
var constructProxy = new Proxy(function () {}, {
  construct: function (target, args, newTarget) {
    constructArrayIsForeign = Object.getPrototypeOf(args) === other.Array.prototype;
    return { count: args.length };
  }
});
var callBound = Function.prototype.bind.call(callProxy, null);
var constructBound = Function.prototype.bind.call(constructProxy, null);
var called = other.Reflect.apply(callBound, null, []) === 29;
var made = other.Reflect.construct(constructBound, [1]);
var revokedCall = Proxy.revocable(function () {}, {});
var revokedCallBound = Function.prototype.bind.call(revokedCall.proxy, null);
revokedCall.revoke();
var callError;
try { other.Reflect.apply(revokedCallBound, null, []); }
catch (error) { callError = error; }
var revokedConstruct = Proxy.revocable(function () {}, {});
var revokedConstructBound = Function.prototype.bind.call(revokedConstruct.proxy, null);
revokedConstruct.revoke();
var constructError;
try { other.Reflect.construct(revokedConstructBound, []); }
catch (error) { constructError = error; }
called && made.count === 1 && callArrayIsForeign && constructArrayIsForeign
  && Object.getPrototypeOf(callError) === other.TypeError.prototype
  && Object.getPrototypeOf(constructError) === other.TypeError.prototype
  && !(callError instanceof TypeError)
  && !(constructError instanceof TypeError);
"#,
    );
}

#[test]
fn bind_observes_proxy_prototype_and_metadata_before_calling_original_proxy() {
    run_boolean(
        r#"
var events = [];
var boundThis = { marker: 100 };
var boundPrototype = { marker: "bound prototype" };
function target(a, b) { return this.marker + a * 10 + b; }
var proxy = new Proxy(target, {
  getPrototypeOf: function (t) {
    events.push("prototype");
    return boundPrototype;
  },
  getOwnPropertyDescriptor: function (t, key) {
    events.push("own:" + key);
    return Reflect.getOwnPropertyDescriptor(t, key);
  },
  get: function (t, key, receiver) {
    events.push("get:" + key);
    if (key === "name") return "proxy name";
    return Reflect.get(t, key, receiver);
  },
  apply: function (t, receiver, args) {
    events.push("apply");
    if (receiver !== boundThis || args.length !== 2 || args[0] !== 2 || args[1] !== 3)
      throw new Error("wrong bound invocation");
    return Reflect.apply(t, receiver, args);
  }
});
var bound = Function.prototype.bind.call(proxy, boundThis, 2);
var metadata = Object.getPrototypeOf(bound) === boundPrototype
  && bound.length === 1 && bound.name === "bound proxy name";
var result = bound(3);
metadata && result === 123
  && events.join(";") === "prototype;own:length;get:length;get:name;apply";
"#,
    );
}

#[test]
fn bound_proxy_preserves_construct_trap_and_proxy_new_target() {
    run_boolean(
        r#"
var constructCount = 0;
var sawOriginalNewTarget = false;
function Target(a, b) {
  this.a = a;
  this.b = b;
  this.newTarget = new.target;
}
var proxy = new Proxy(Target, {
  construct: function (target, args, newTarget) {
    constructCount++;
    sawOriginalNewTarget = newTarget === proxy;
    return Reflect.construct(target, args, newTarget);
  }
});
var Bound = Function.prototype.bind.call(proxy, null, 4);
var value = new Bound(5);
constructCount === 1 && sawOriginalNewTarget
  && value instanceof Target && value.a === 4 && value.b === 5
  && value.newTarget === proxy;
"#,
    );
}

#[test]
fn bound_proxy_of_nonconstructor_can_call_but_cannot_construct() {
    run_boolean(
        r#"
var calls = 0;
var proxy = new Proxy(Math.sin, {
  apply: function (target, receiver, args) {
    calls++;
    return Reflect.apply(target, receiver, args);
  }
});
var bound = Function.prototype.bind.call(proxy, null, 0);
var result = bound();
var error;
try { new bound(); } catch (caught) { error = caught; }
result === 0 && calls === 1 && error instanceof TypeError;
"#,
    );
}

#[test]
fn bind_routes_proxy_trap_exceptions_in_spec_order() {
    run_boolean(
        r#"
var error = {};
var events = [];
var target = function () {};
var first = new Proxy(target, {
  getPrototypeOf: function () { events.push("prototype"); throw error; },
  getOwnPropertyDescriptor: function () { events.push("own"); return undefined; }
});
var firstError;
try { Function.prototype.bind.call(first); } catch (caught) { firstError = caught; }
var second = new Proxy(target, {
  getPrototypeOf: function () { events.push("prototype2"); return Function.prototype; },
  getOwnPropertyDescriptor: function () { events.push("own2"); throw error; },
  get: function () { events.push("get2"); return 1; }
});
var secondError;
try { Function.prototype.bind.call(second); } catch (caught) { secondError = caught; }
firstError === error && secondError === error
  && events.join(";") === "prototype;prototype2;own2";
"#,
    );
}
