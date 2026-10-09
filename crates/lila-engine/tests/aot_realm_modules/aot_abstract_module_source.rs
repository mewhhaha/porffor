use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn run(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    for strict in ["", "'use strict';\n"] {
        let source = format!("{strict}{source}");
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
            .expect("native abstract module source runs through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
    }
}

#[test]
fn abstract_module_source_is_one_native_intrinsic_with_exact_prototype_descriptors() {
    run(r#"
function check(value) { if (!value) throw new Error('intrinsic descriptor'); }
var C = __lilaGetAbstractModuleSource();
check(C === __lilaGetAbstractModuleSource());
check(typeof C === 'function' && !Object.hasOwn(globalThis, 'AbstractModuleSource'));
check(Object.getPrototypeOf(C) === Function.prototype);
check(Function.prototype.toString.call(C) === 'function AbstractModuleSource() { [native code] }');
var name = Object.getOwnPropertyDescriptor(C, 'name');
check(name.value === 'AbstractModuleSource' && !name.writable && !name.enumerable && name.configurable);
var length = Object.getOwnPropertyDescriptor(C, 'length');
check(length.value === 0 && !length.writable && !length.enumerable && length.configurable);
var prototype = Object.getOwnPropertyDescriptor(C, 'prototype');
check(!prototype.writable && !prototype.enumerable && !prototype.configurable);
check(Object.getPrototypeOf(prototype.value) === Object.prototype);
var constructor = Object.getOwnPropertyDescriptor(prototype.value, 'constructor');
check(constructor.value === C && constructor.writable && !constructor.enumerable && constructor.configurable);
var tag = Object.getOwnPropertyDescriptor(prototype.value, Symbol.toStringTag);
check(typeof tag.get === 'function' && tag.set === undefined && !tag.enumerable && tag.configurable);
check(tag.get.name === 'get [Symbol.toStringTag]' && tag.get.length === 0);
check(Function.prototype.toString.call(tag.get) === 'function get [Symbol.toStringTag]() { [native code] }');
check(!Object.hasOwn(tag.get, 'prototype'));
for (var value of [undefined, null, 262, 'text', {}, prototype.value, Object.create(prototype.value)]) {
  check(tag.get.call(value) === undefined);
}
var pair = Proxy.revocable({}, {});
pair.revoke();
check(tag.get.call(pair.proxy) === undefined);
gc();
check(__lilaGetAbstractModuleSource() === C && C.prototype === prototype.value);
true;
"#);
}

#[test]
fn abstract_module_source_call_and_construct_throw_intrinsic_errors_before_new_target_properties() {
    run(r#"
var C = __lilaGetAbstractModuleSource();
var originalTypeError = TypeError;
var marker = { value: 9007199254740993n };
marker.self = marker;
var reads = 0;
var newTarget = new Proxy(function () {}, {
  get: function () { reads++; throw marker; }
});
globalThis.TypeError = function () { throw marker; };
for (var mode = 0; mode < 3; mode++) {
  var caught;
  try {
    if (mode === 0) C();
    else if (mode === 1) new C();
    else Reflect.construct(C, [], newTarget);
  } catch (error) { caught = error; }
  if (!(caught instanceof originalTypeError) || caught === marker) throw 'intrinsic error';
}
if (reads !== 0) throw 'native constructor read newTarget properties';
gc();
if (__lilaGetAbstractModuleSource() !== C) throw 'native identity after global mutation';
true;
"#);
}

#[test]
fn created_realms_retain_distinct_abstract_module_source_intrinsics_and_defining_realm_errors() {
    run(r#"
var root = __lilaGetAbstractModuleSource();
var other = __lilaCreateRealm();
var C = other.AbstractModuleSource;
if (C === root || C.prototype === root.prototype) throw 'shared realm identity';
if (other.global.$262.AbstractModuleSource !== C) throw 'created host hook identity';
if (Object.getPrototypeOf(C) !== other.global.Function.prototype) throw 'function realm';
if (Object.getPrototypeOf(C.prototype) !== other.global.Object.prototype) throw 'prototype realm';
var get = Object.getOwnPropertyDescriptor(C.prototype, Symbol.toStringTag).get;
if (Object.getPrototypeOf(get) !== other.global.Function.prototype) throw 'getter realm';
var expected = other.global.TypeError.prototype;
other.global.TypeError = function () { throw 'poisoned global'; };
for (var mode = 0; mode < 2; mode++) {
  var caught;
  try { if (mode === 0) C(); else new C(); } catch (error) { caught = error; }
  if (Object.getPrototypeOf(caught) !== expected) throw 'error realm';
}
gc();
if (other.AbstractModuleSource !== C || get.call(C.prototype) !== undefined) throw 'retained realm';
true;
"#);
}
