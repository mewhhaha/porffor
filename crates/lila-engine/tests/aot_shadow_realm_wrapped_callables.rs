use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, RealmBuilder, RunOptions,
};

fn assert_modes(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!(
            "{directive}function assert(value, message) {{ if (!value) throw new Error(message); }}\n{source}"
        );
        let observation = Engine::new(RealmBuilder::new().build())
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
            .expect("finite ShadowRealm callable sources compile and execute through Wasm AOT");
        assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observation.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{observation:?}\n{source}"
        );
    }
}

#[test]
fn wrapped_values_keep_primitives_and_create_fresh_nonconstructible_callables() {
    assert_modes(
        r#"
const realm = new ShadowRealm();
const echo = realm.evaluate('(function echo(value) { return value; })');
const token = Symbol('crossing');
for (const value of [undefined, null, false, true, -0, NaN, Infinity, 'text', 9007199254740993n, token]) {
    assert(Object.is(echo(value), value), 'primitive identity');
}
const local = function local(a, b) { return a + b; };
local.secret = 7;
const first = echo(local), second = echo(local);
assert(first !== local && second !== first && second !== local, 'fresh identity at each crossing');
assert(first(2, 3) === 5 && second(4, 5) === 9, 'real target execution');
assert(first.secret === undefined && !Object.hasOwn(first, 'prototype'), 'only wrapper metadata');
assert(Object.getPrototypeOf(first) === Function.prototype, 'destination Function prototype');
assert(Object.getOwnPropertyNames(first).join(',') === 'length,name', 'one metadata key each');
for (const name of ['length', 'name']) {
    const descriptor = Object.getOwnPropertyDescriptor(first, name);
    assert(!descriptor.writable && !descriptor.enumerable && descriptor.configurable, 'metadata attributes');
}
assert(first.length === 2 && first.name === 'local', 'copied metadata');
let error;
try { new first(); } catch (caught) { error = caught; }
assert(error instanceof TypeError, 'wrapper is not constructible');
const original = realm.evaluate('globalThis.sharedCount = 0; globalThis.shared = function shared() { return ++sharedCount; }; shared');
const again = realm.evaluate('shared');
assert(original !== again && original() === 1 && again() === 2, 'fresh wrappers share captured target state');
true;
"#,
    );
}

#[test]
fn metadata_uses_own_length_before_name_without_prototype_observation_or_coercion() {
    assert_modes(
        r#"
const realm = new ShadowRealm();
let trace = '';
function report(event) { trace += event + ';'; }
const factory = realm.evaluate(`(function(report) {
    return new Proxy(function target(a, b, c) { return a + b + c; }, {
        getPrototypeOf(target) { report('prototype'); return Reflect.getPrototypeOf(target); },
        getOwnPropertyDescriptor(target, key) { report('own:' + key); return Reflect.getOwnPropertyDescriptor(target, key); },
        get(target, key, receiver) {
            report('get:' + key);
            if (key === 'length') return 3.9;
            if (key === 'name') return 'renamed';
            return Reflect.get(target, key, receiver);
        }
    });
})`);
const wrapped = factory(report);
assert(trace === 'own:length;get:length;get:name;', 'ordered Proxy metadata without GetPrototypeOf');
assert(wrapped.length === 3 && wrapped.name === 'renamed' && wrapped(1, 2, 3) === 6, 'Proxy remains callable');
trace = '';
const inherited = realm.evaluate(`(function(report) {
    function target() {}
    delete target.length;
    delete target.name;
    Object.setPrototypeOf(target, {
        get length() { report('inherited length'); return 99; },
        get name() { report('inherited name'); return 'inherited'; }
    });
    return target;
})`)(report);
assert(inherited.length === 0 && inherited.name === 'inherited', 'own length only and ordinary name Get');
assert(trace === 'inherited name;', 'inherited length not read');

const lengthOf = realm.evaluate('(function(callback) { return callback.length; })');
const cases = [Infinity, -Infinity, NaN, -0, 3.9, -3.9, 4294967296, '9', null, undefined, 1n];
const expected = [Infinity, 0, 0, 0, 3, 0, 4294967296, 0, 0, 0, 0];
function callback() {}
for (let index = 0; index < cases.length; index++) {
    Object.defineProperty(callback, 'length', {value: cases[index], configurable: true});
    assert(Object.is(lengthOf(callback), expected[index]), 'number normalization');
}
let coercions = 0;
const metadata = { valueOf() { coercions++; return 9; }, toString() { coercions++; return 'wrong'; } };
Object.defineProperty(callback, 'length', {value: metadata});
Object.defineProperty(callback, 'name', {value: metadata});
const describe = realm.evaluate('(function(callback) { return callback.length === 0 && callback.name === ""; })');
assert(describe(callback) && coercions === 0, 'metadata values are not coerced');
true;
"#,
    );
}

#[test]
fn arguments_cross_before_this_and_a_failed_crossing_stops_later_observations() {
    assert_modes(
        r#"
const realm = new ShadowRealm();
const call = realm.evaluate(`globalThis.calls = 0; (function(a, b) {
    'use strict';
    calls++;
    return a !== b && this !== a && Object.getPrototypeOf(a) === Function.prototype
        && Object.getPrototypeOf(b) === Function.prototype
        && Object.getPrototypeOf(this) === Function.prototype;
})`);
let trace = '';
function marked(label) {
    const target = function() {};
    Object.defineProperty(target, 'length', {get() { trace += label + 'L'; return 0; }});
    Object.defineProperty(target, 'name', {get() { trace += label + 'N'; return label; }});
    return target;
}
const argument = marked('A'), receiver = marked('T'), later = marked('Z');
assert(Reflect.apply(call, receiver, [argument, argument]), 'each argument receives a new inner wrapper');
assert(trace === 'ALANALANTLTN' && realm.evaluate('calls') === 1, 'arguments then this then Call');
trace = '';
let error;
try { Reflect.apply(call, receiver, [argument, {}, later]); } catch (caught) { error = caught; }
assert(error instanceof TypeError && trace === 'ALAN', 'ordinary object stops later argument and this metadata');
assert(realm.evaluate('calls') === 1, 'target was not invoked');
trace = '';
const marker = {};
Object.defineProperty(argument, 'length', {get() { trace += 'throw'; throw marker; }});
error = undefined;
try { Reflect.apply(call, receiver, [argument, later]); } catch (caught) { error = caught; }
assert(error instanceof TypeError && error !== marker && trace === 'throw', 'metadata throw is replaced and stops the boundary');
assert(realm.evaluate('calls') === 1, 'metadata failure precedes Call');
true;
"#,
    );
}

#[test]
fn wrapper_defining_realm_owns_errors_and_thrown_values_are_never_inspected() {
    assert_modes(
        r#"
const foreign = __lilaCreateRealm().global;
const realm = new foreign.ShadowRealm();
const noop = realm.evaluate('(function() {})');
const objectResult = realm.evaluate('(function() { return {}; })');
const throws = realm.evaluate(`globalThis.errorReads = 0; (function() {
    throw new Proxy({}, {
        get() { errorReads++; throw 'forbidden Get'; },
        getPrototypeOf() { errorReads++; throw 'forbidden prototype'; }
    });
})`);
assert(Object.getPrototypeOf(noop) === foreign.Function.prototype, 'method Realm selects wrapper prototype');
let previous;
for (const invoke of [() => noop({}), () => objectResult(), () => throws()]) {
    let error;
    try { invoke(); } catch (caught) { error = caught; }
    assert(Object.getPrototypeOf(error) === foreign.TypeError.prototype, 'wrapper defining Realm error');
    assert(error !== previous, 'fresh TypeError');
    previous = error;
}
assert(realm.evaluate('errorReads') === 0, 'CreateTypeErrorCopy performs no user operations');
const local = ShadowRealm.prototype.evaluate.call(realm, '(function() { return {}; })');
assert(Object.getPrototypeOf(local) === Function.prototype, 'borrowed local evaluate changes wrapper Realm');
let error;
try { local(); } catch (caught) { error = caught; }
assert(Object.getPrototypeOf(error) === TypeError.prototype, 'borrowed method wrapper keeps local Realm');
true;
"#,
    );
}

#[test]
fn proxy_metadata_failures_are_converted_and_revocation_precedes_argument_metadata() {
    assert_modes(
        r#"
const realm = new ShadowRealm();
const failMetadata = realm.evaluate(`(function(report) {
    return new Proxy(function() {}, {
        getOwnPropertyDescriptor() { report('own'); throw {}; },
        get() { report('get'); throw 'late'; }
    });
})`);
let trace = '';
let error;
try { failMetadata(event => { trace += event; }); } catch (caught) { error = caught; }
assert(error instanceof TypeError && trace === 'own', 'own descriptor abrupt is converted before Get');
const wrapped = realm.evaluate('globalThis.revocable = Proxy.revocable(function target() {}, {}); revocable.proxy');
realm.evaluate('revocable.revoke()');
const argument = function() {};
Object.defineProperty(argument, 'length', {get() { trace += 'argument'; return 0; }});
trace = '';
error = undefined;
try { wrapped(argument); } catch (caught) { error = caught; }
assert(error instanceof TypeError && trace === '', 'GetFunctionRealm rejects revoked target first');
const bound = realm.evaluate('(function target(a, b) { return a + b; }).bind(null, 4)');
assert(bound.length === 1 && bound.name === 'bound target' && bound(5) === 9, 'bound callable target retains own metadata and execution');
true;
"#,
    );
}
