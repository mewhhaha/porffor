use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_native_effects(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for strict in [false, true] {
        let source = format!(
            "{}function assert(v,m){{if(!v)throw new Error(m);}}\n{source}",
            if strict { "\"use strict\";\n" } else { "" }
        );
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
            .expect("native caller effects use actual Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
    }
}

#[test]
fn synchronous_builtin_hooks_update_captured_binding_types_and_object_properties() {
    assert_native_effects(
        r#"
let observed = 0;
let property = { value: 1 };
String({ toString() { observed = "string"; property.value = "string"; return "x"; } });
assert(observed === "string" && property.value === "string", "String conversion effects");
new Date({ valueOf() { observed = Symbol.iterator; property.value = undefined; return 0; } });
assert(observed === Symbol.iterator && property.value === undefined, "Date conversion effects");
Symbol({ toString() { observed = null; property.value = false; return "description"; } });
assert(observed === null && property.value === false, "Symbol description effects");
new RegExp({ get [Symbol.match]() { observed = "regexp"; property.value = "regexp"; return false; }, toString() { return "x"; } });
assert(observed === "regexp" && property.value === "regexp", "RegExp acquisition effects");
let array = [3, 1];
Object.defineProperty(array, "0", { get() { observed = true; property.value = true; return 3; }, configurable: true });
assert(array.at(0) === 3, "Array result");
assert(observed === true && property.value === true, "Array indexed getter effects");
let typed = new Uint8Array([7]);
assert(typed.at({ valueOf() { observed = "typed"; property.value = "typed"; return 0; } }) === 7, "TypedArray result");
assert(observed === "typed" && property.value === "typed", "TypedArray index conversion effects");
new Intl.NumberFormat("en", { get style() { observed = 9; property.value = 9; return "decimal"; } });
assert(observed === 9 && property.value === 9, "Intl option effects");
let prepared = Function("return 17;");
assert(prepared() === 17, "prepared source exists");
let converted = Function({ toString() { observed = "function"; property.value = "function"; return "return 17;"; } });
assert(observed === "function" && property.value === "function" && converted() === 17, "prepared Function conversion effects");
"#,
    );
}

#[test]
fn super_construction_observes_prototype_getter_changes_before_continuing() {
    assert_native_effects(
        r#"
let state = 1;
let afterSuper;
class BooleanDerived extends Boolean { constructor() { super(); afterSuper = state; } }
let booleanTarget = new Proxy(function() {}, { get(t, k, r) { if (k === "prototype") { state = "boolean"; return Boolean.prototype; } return Reflect.get(t, k, r); } });
Reflect.construct(BooleanDerived, [], booleanTarget);
assert(afterSuper === "boolean", "Boolean super prototype effects");
state = 2;
class ArrayDerived extends Array { constructor() { super(); afterSuper = state; } }
let arrayTarget = new Proxy(function() {}, { get(t, k, r) { if (k === "prototype") { state = Symbol.iterator; return Array.prototype; } return Reflect.get(t, k, r); } });
Reflect.construct(ArrayDerived, [], arrayTarget);
assert(afterSuper === Symbol.iterator, "Array super prototype effects");
state = 3;
class ObjectDerived extends Object { constructor() { super(); afterSuper = state; } }
let objectTarget = new Proxy(function() {}, { get(t, k, r) { if (k === "prototype") { state = undefined; return Object.prototype; } return Reflect.get(t, k, r); } });
Reflect.construct(ObjectDerived, [], objectTarget);
assert(afterSuper === undefined, "Object super prototype effects");
"#,
    );
}

#[test]
fn bigint_and_symbol_preserve_constructor_admission_and_body_rejection_order() {
    assert_native_effects(
        r#"
for (let intrinsic of [Symbol, BigInt]) {
  let converted = false;
  let threw = false;
  let input = { toString() { converted = true; return "1"; } };
  try { new intrinsic(input); }
  catch (error) { threw = error instanceof TypeError; }
  assert(threw && !converted, "native constructor body rejects before conversion");
  let reads = "";
  threw = false;
  try { Reflect.construct(intrinsic, { get length() { reads += "length"; return 1; }, get 0() { reads += "0"; return input; } }); }
  catch (error) { threw = error instanceof TypeError; }
  assert(threw && reads === "length0" && !converted, "argument list is acquired before native body rejection");
  let prototypeRead = false;
  let bodyRan = false;
  let newTarget = new Proxy(intrinsic, { get(t, k, r) { if (k === "prototype") prototypeRead = true; return Reflect.get(t, k, r); } });
  let object = Reflect.construct(function() { bodyRan = true; }, [], newTarget);
  assert(prototypeRead && bodyRan && Object.getPrototypeOf(object) === intrinsic.prototype, "intrinsic admits newTarget and prototype acquisition");
  let intercepted = new Proxy(intrinsic, { construct() { return { intercepted: true }; } });
  assert(new intercepted().intercepted === true, "intrinsic constructor capability admits Proxy construct trap");
  class Admitted extends intrinsic { constructor() { return { admitted: true }; } }
  assert(new Admitted().admitted === true, "class extends admits intrinsic");
  class RejectedSuper extends intrinsic {}
  threw = false;
  try { new RejectedSuper(); }
  catch (error) { threw = error instanceof TypeError; }
  assert(threw, "inherited native constructor body still rejects super");
}
"#,
    );
}
