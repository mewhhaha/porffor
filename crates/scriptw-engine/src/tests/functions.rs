use super::*;

#[test]
fn wasm_backend_repeated_exact_call_contexts_preserve_nested_callback_targets() {
    let source = r#"
function invoke(callback, constructor) {
  callback(constructor);
}

function forEachConstructor(callback) {
  invoke(callback, Int8Array);
  invoke(callback, Uint8Array);
}

let copiedValues = "";
forEachConstructor(function (Source) {
  forEachConstructor(function (Target) {
    const source = new Source(new SharedArrayBuffer(1));
    source[0] = 7;
    const copy = new Target(source);
    copiedValues += copy[0];
  });
});
copiedValues === "7777";
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("repeated exact call contexts should preserve nested callback targets");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_supports_hoisted_function_calls() {
    let outcome = engine()
        .run_script(
            "add(1, 2); function add(x, y) { return x + y; }",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should run hoisted function call");
    assert!(outcome.note.contains("number(3"));
}

#[test]
fn wasm_backend_supports_function_values_and_indirect_calls() {
    let alias_outcome = engine()
        .run_script(
            "function inc(x) { return x + 1; } let g = inc; g(2);",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should run indirect call through alias");
    assert!(alias_outcome.note.contains("number(3"));

    let passthrough_outcome = engine()
            .run_script(
                "function inc(x) { return x + 1; } function pick() { return inc; } let g = pick(); g(2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run function pass-through");
    assert!(passthrough_outcome.note.contains("number(3"));
}

#[test]
fn wasm_backend_cross_realm_iterator_constructor_call_throws_other_realm_type_error() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let C = other.Iterator;
let iteratorPrototype = C.prototype;
let constructorDesc = Object.getOwnPropertyDescriptor(iteratorPrototype, "constructor");
let mainConstructorDesc = Object.getOwnPropertyDescriptor(Iterator.prototype, "constructor");
let tagDesc = Object.getOwnPropertyDescriptor(iteratorPrototype, Symbol.toStringTag);
let mainTagDesc = Object.getOwnPropertyDescriptor(Iterator.prototype, Symbol.toStringTag);
let helperNames = [
  "toArray", "forEach", "every", "some", "find", "reduce",
  "map", "filter", "flatMap", "take", "drop"
];
let helperResults = [];
let thrown = "none";
let toArrayThrow = "none";
let disposeThrow = "none";
for (let i = 0; i < helperNames.length; i++) {
  let name = helperNames[i];
  helperResults.push(
    name + ":" +
    (typeof iteratorPrototype[name]) + ":" +
    (iteratorPrototype[name] === Iterator.prototype[name])
  );
}
try {
  C();
} catch (error) {
  thrown = [
    Object.getPrototypeOf(error) === other.TypeError.prototype,
    error instanceof other.TypeError,
    error instanceof TypeError
  ].join(":");
}
try {
  iteratorPrototype.toArray.call(null);
} catch (error) {
  toArrayThrow = [
    Object.getPrototypeOf(error) === other.TypeError.prototype,
    error instanceof other.TypeError,
    error instanceof TypeError
  ].join(":");
}
try {
  iteratorPrototype[Symbol.dispose].call({ return: 1 });
} catch (error) {
  disposeThrow = [
    Object.getPrototypeOf(error) === other.TypeError.prototype,
    error instanceof other.TypeError,
    error instanceof TypeError
  ].join(":");
}
[
  typeof C,
  C === Iterator,
  C.prototype === Iterator.prototype,
  Object.getPrototypeOf(C) === other.Function.prototype,
  Object.getPrototypeOf(iteratorPrototype) === other.Object.prototype,
  typeof iteratorPrototype[Symbol.iterator],
  iteratorPrototype[Symbol.iterator] === Iterator.prototype[Symbol.iterator],
  iteratorPrototype[Symbol.iterator]() === iteratorPrototype,
  thrown,
  typeof iteratorPrototype.toArray,
  iteratorPrototype.toArray === Iterator.prototype.toArray,
  toArrayThrow,
  typeof C.from,
  C.from === Iterator.from,
  iteratorPrototype.constructor === C,
  typeof constructorDesc.get,
  typeof constructorDesc.set,
  constructorDesc.get === mainConstructorDesc.get,
  constructorDesc.set === mainConstructorDesc.set,
  typeof iteratorPrototype[Symbol.dispose],
  iteratorPrototype[Symbol.dispose] === Iterator.prototype[Symbol.dispose],
  iteratorPrototype[Symbol.toStringTag],
  typeof tagDesc.get,
  typeof tagDesc.set,
  tagDesc.get === mainTagDesc.get,
  tagDesc.set === mainTagDesc.set,
  disposeThrow,
  helperResults.join(",")
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("cross-realm Iterator constructor call should throw in defining realm");
    assert!(
            outcome.note.contains(
                "string(function|false|false|true|true|function|false|true|true:true:false|function|false|true:true:false|function|false|true|function|function|false|false|function|false|Iterator|function|function|false|false|true:true:false|toArray:function:false,forEach:function:false,every:function:false,some:function:false,find:function:false,reduce:function:false,map:function:false,filter:function:false,flatMap:function:false,take:function:false,drop:function:false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_cross_realm_error_constructor_fallback_uses_new_target_realm() {
    let outcome = engine()
        .run_script(
            r#"
let otherRealm = __scriptwCreateRealm();
let other = otherRealm.global;
let originalOtherErrorPrototype = other.Error.prototype;
// An ordinary function has a writable prototype; Error.prototype does not.
let otherConstructor = otherRealm.evalScript("function Other() {} Other;");
otherConstructor.prototype = 7;
if (otherConstructor.prototype !== 7) throw new Error("fallback precondition");
let error = Reflect.construct(Error, ["m"], otherConstructor);
let proxyError = Reflect.construct(Error, ["m"], new Proxy(otherConstructor, {}));
[
  Object.getPrototypeOf(error) === originalOtherErrorPrototype,
  Object.getPrototypeOf(error) === Error.prototype,
  Object.getPrototypeOf(error) === otherConstructor.prototype,
  Object.getPrototypeOf(proxyError) === originalOtherErrorPrototype,
  Object.getPrototypeOf(proxyError) === Error.prototype
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("cross-realm Error construction should fall back to newTarget realm intrinsic");
    assert!(
        outcome.note.contains("string(true|false|false|true|false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_call_argument_lists_are_snapshotted_observably() {
    let outcome = engine()
        .run_script(
            r#"
var calls = [];
var argumentsList = {
  get length() {
    calls.push("length");
    return 2;
  },
  get 0() {
    calls.push("0");
    return 1;
  },
  get 1() {
    calls.push("1");
    return 2;
  }
};
function Sum(left, right) {
  this.value = left + right;
}
var result = Reflect.construct(Sum, argumentsList);

var abrupt = false;
try {
  Reflect.construct(Sum, {
    get length() {
      throw 42;
    }
  });
} catch (error) {
  abrupt = error === 42;
}

var primitiveError = false;
try {
  Reflect.construct(Sum, 1);
} catch (error) {
  primitiveError = error instanceof TypeError;
}

var applyCalls = [];
var applyArguments = {
  get length() {
    applyCalls.push("length");
    return 2;
  },
  get 0() {
    applyCalls.push("0");
    return "left";
  },
  get 1() {
    applyCalls.push("1");
    return "right";
  }
};
function join(left, right) {
  return left + ":" + right;
}
var reflectedApply = Reflect.apply(join, null, applyArguments);
var prototypeApply = join.apply(null, applyArguments);
var nullishApply = join.apply(null, null);

var indexedMarker = {};
var indexedAbrupt = false;
try {
  Reflect.apply(join, null, {
    length: 2,
    0: "left",
    get 1() {
      throw indexedMarker;
    }
  });
} catch (error) {
  indexedAbrupt = error === indexedMarker;
}

var proxyReads = [];
var proxiedArguments = new Proxy({
  length: 2,
  0: "proxy",
  1: "arguments"
}, {
  get: function(target, key, receiver) {
    proxyReads.push(key);
    return Reflect.get(target, key, receiver);
  }
});
var proxiedApply = Reflect.apply(join, null, proxiedArguments);

var callableProxy = join;
for (var proxyDepth = 0; proxyDepth < 12; proxyDepth++) {
  callableProxy = new Proxy(callableProxy, {});
}
var reflectedProxyApply = Reflect.apply(callableProxy, null, ["deep", "proxy"]);
var prototypeProxyApply = callableProxy.apply(null, ["prototype", "proxy"]);

var constructableProxy = new Proxy(Sum, {});
var proxyConstruct = Reflect.construct(constructableProxy, [4, 5]);
var revokedConstructor = Proxy.revocable(Sum, {});
revokedConstructor.revoke();
var revokedArgumentsObserved = false;
var revokedConstructError = false;
try {
  Reflect.construct(revokedConstructor.proxy, {
    get length() {
      revokedArgumentsObserved = true;
      return 0;
    }
  });
} catch (error) {
  revokedConstructError = error instanceof TypeError;
}

var invalidTargetObservedArguments = false;
try {
  Reflect.apply(1, null, {
    get length() {
      invalidTargetObservedArguments = true;
      return 0;
    }
  });
} catch (error) {
  primitiveError = primitiveError && error instanceof TypeError;
}

var reflectNullishError = false;
var prototypePrimitiveError = false;
try {
  Reflect.apply(join, null, null);
} catch (error) {
  reflectNullishError = error instanceof TypeError;
}
try {
  join.apply(null, 1);
} catch (error) {
  prototypePrimitiveError = error instanceof TypeError;
}

[
  result.value,
  calls.join(","),
  abrupt,
  primitiveError,
  reflectedApply,
  prototypeApply,
  nullishApply,
  applyCalls.join(","),
  invalidTargetObservedArguments,
  indexedAbrupt,
  proxiedApply,
  proxyReads.join(","),
  reflectedProxyApply,
  prototypeProxyApply,
  proxyConstruct.value,
  revokedArgumentsObserved,
  revokedConstructError,
  reflectNullishError,
  prototypePrimitiveError
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("call argument lists should be snapshotted observably");
    assert!(
            outcome.note.contains(
                "string(3|length,0,1|true|true|left:right|left:right|undefined:undefined|length,0,1,length,0,1|false|true|proxy:arguments|length,0,1|deep:proxy|prototype:proxy|9|true|true|true|true)"
            ),
            "note: {}",
            outcome.note
        );
}

// Global function properties are writable, non-enumerable, configurable
// data properties (ECMA-262 clause 18, "ECMAScript Standard Built-in
// Objects") with the lengths and names of 19.2 and B.2.1. The other
// Realm's `eval` is a built-in whose call context takes its own [[Realm]]
// (10.3.1 [[Call]] via BuiltinCallOrConstruct, 10.3.3), and indirect eval
// evaluates in that current Realm's global environment (19.2.1 eval and
// 19.2.1.1 PerformEval). A compile-time-known source is therefore ordinary
// other-Realm eval code: `"1 + 1"` completes with 2, `this` and `Array`
// resolve in the other Realm, and GetValue on a null base (6.2.5.5, ToObject)
// throws the other Realm's TypeError.
#[test]
fn wasm_backend_create_realm_exposes_global_function_properties() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let infinityDesc = Object.getOwnPropertyDescriptor(other, "Infinity");
let nanDesc = Object.getOwnPropertyDescriptor(other, "NaN");
let undefinedDesc = Object.getOwnPropertyDescriptor(other, "undefined");
let globalThisDesc = Object.getOwnPropertyDescriptor(other, "globalThis");
let evalSum = other.eval("1 + 1");
let evalRealm =
  (other.eval("this") === other) + ":" +
  (other.eval("Array") === other.Array) + ":" +
  (other.eval("Array") === Array);
let evalThrow = "missing";
try {
  other.eval("null.x");
} catch (error) {
  evalThrow =
    (Object.getPrototypeOf(error) === other.TypeError.prototype) + ":" +
    (error instanceof other.TypeError) + ":" +
    (error instanceof TypeError);
}
let functionPropertyMismatches = [];
for (let [name, length] of [
  ["eval", 1], ["isFinite", 1], ["isNaN", 1], ["parseFloat", 1], ["parseInt", 2],
  ["decodeURI", 1], ["decodeURIComponent", 1], ["encodeURI", 1],
  ["encodeURIComponent", 1], ["escape", 1], ["unescape", 1]
]) {
  let desc = Object.getOwnPropertyDescriptor(other, name);
  if (!desc || !desc.writable || desc.enumerable || !desc.configurable ||
      typeof desc.value !== "function" || desc.value.length !== length ||
      desc.value.name !== name || desc.value === globalThis[name] ||
      Object.getPrototypeOf(desc.value) !== other.Function.prototype) {
    functionPropertyMismatches.push(name);
  }
}
[
  other.Infinity === Infinity,
  other.NaN === other.NaN,
  other.undefined === undefined,
  other.globalThis === other,
  other.globalThis === globalThis,
  infinityDesc.writable,
  infinityDesc.enumerable,
  infinityDesc.configurable,
  nanDesc.writable,
  nanDesc.enumerable,
  nanDesc.configurable,
  undefinedDesc.writable,
  undefinedDesc.enumerable,
  undefinedDesc.configurable,
  globalThisDesc.writable,
  globalThisDesc.enumerable,
  globalThisDesc.configurable,
  other.eval === eval,
  typeof other.eval,
  other.eval(7),
  evalSum,
  evalRealm,
  evalThrow,
  functionPropertyMismatches.length === 0 ? "ok" : functionPropertyMismatches.join(","),
  other.isFinite === isFinite,
  other.isNaN === isNaN,
  other.escape === escape,
  other.unescape === unescape,
  typeof other.isFinite,
  typeof other.isNaN,
  typeof other.escape,
  typeof other.unescape,
  other.isFinite("3"),
  other.isFinite("x"),
  other.isNaN("x"),
  other.isNaN("3"),
  other.unescape(other.escape("a b")) === "a b"
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm global object should expose global function properties");
    assert!(
            outcome.note.contains(
                "string(true|false|true|true|false|false|false|false|false|false|false|false|false|false|true|false|true|false|function|7|2|true:true:false|true:true:false|ok|false|false|false|false|function|function|function|function|true|false|true|false|true)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_script_var_and_function_properties_are_enumerable() {
    let outcome = engine()
        .run_script(
            r#"
var variable;
function declared() {}
let variableDescriptor = Object.getOwnPropertyDescriptor(globalThis, "variable");
let functionDescriptor = Object.getOwnPropertyDescriptor(globalThis, "declared");
[
  variableDescriptor.writable,
  variableDescriptor.enumerable,
  variableDescriptor.configurable,
  functionDescriptor.writable,
  functionDescriptor.enumerable,
  functionDescriptor.configurable
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("script global declarations should create ordinary global properties");
    assert!(
        outcome
            .note
            .contains("string(true|true|false|true|true|false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_regexp_subclass_exec_override_is_observable() {
    let outcome = engine()
        .run_script(
            r#"
class FakeRegExp extends RegExp {
  exec() {
    let result = ["ab", "a"];
    result.index = 0;
    result.groups = { a: "b" };
    return result;
  }
}

let regexp = new FakeRegExp();
let result = regexp.exec("ab");
[result.groups.a, "ab".replace(regexp, "$<a>")].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("RegExp subclass exec override should remain observable");
    assert!(
        outcome.note.contains("string(b|b)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_cross_realm_function_to_string_throws_other_realm_type_error() {
    let outcome = engine()
            .run_script(
                r#"
let other = __scriptwCreateRealm();
let method = other.global.Function.prototype.toString;
let bound = method.bind({});
let direct = "missing";
let boundResult = "missing";

try {
  method.call({});
} catch (error) {
  direct =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

try {
  bound();
} catch (error) {
  boundResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

direct + "|" + boundResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("borrowed and bound cross-realm Function.prototype.toString should throw in defining realm");
    assert!(
        outcome
            .note
            .contains("string(true:true:false|true:true:false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_string_replace_all_handles_literal_and_function_replacements() {
    let outcome = engine()
        .run_script(
            r#"
let calls = [];
let functional = "a,b,".replaceAll(",", function(match, position, string) {
  calls.push(match + ":" + position + ":" + string);
  return "$&";
});
let substitution = "aba".replaceAll("a", "$`|$&|$'|$$|$1|$<x>");
let empty = "ab".replaceAll("", "-");
let first = "aaa".replace("a", "x");
let regexpCalls = [];
let regexp = /(a)/g[Symbol.replace]("aaa abc", function(match, capture, position, string) {
  regexpCalls.push(match + ":" + capture + ":" + position + ":" + string);
  return "z";
});
[functional, calls.join(";"), substitution, empty, first, regexp, regexpCalls.join(";")].join("\n");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should perform literal string replacements");
    assert!(
            outcome.note.contains(
                "string(a$&b$&\n,:1:a,b,;,:3:a,b,\n|a|ba|$|$1|$<x>bab|a||$|$1|$<x>\n-a-b-\nxaa\nzzz zbc\na:a:0:aaa abc;a:a:1:aaa abc;a:a:2:aaa abc;a:a:4:aaa abc)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_supports_script_closure_capture() {
    let outcome = engine()
        .run_script(
            "let x = 1; function f() { return x; } f();",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("script closure should run");
    assert!(outcome.note.contains("number(1"));
}

#[test]
fn wasm_backend_supports_nested_function_declaration() {
    let outcome = engine()
            .run_script(
                "function outer() { let x = 1; function inner() { return x + 1; } return inner(); } outer();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("nested function declaration should run");
    assert!(outcome.note.contains("number(2"));
}

#[test]
fn wasm_backend_supports_closure_mutation() {
    let outcome = engine()
            .run_script(
                "function outer() { let x = 1; function inc() { x = x + 1; return x; } inc(); return inc(); } outer();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("closure mutation should run");
    assert!(outcome.note.contains("number(3"));
}

#[test]
fn wasm_backend_supports_anonymous_function_expression() {
    let outcome = engine()
        .run_script(
            "let f = function (x) { return x + 1; }; f(2);",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("anonymous function expression should run");
    assert!(outcome.note.contains("number(3"));
}

#[test]
fn wasm_backend_supports_returned_closure_call() {
    let outcome = engine()
            .run_script(
                "function outer() { let x = 2; return function (y) { return x + y; }; } let f = outer(); f(3);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("returned closure should run");
    assert!(outcome.note.contains("number(5"));
}

#[test]
fn wasm_backend_supports_var_closure_capture() {
    let outcome = engine()
            .run_script(
                "function outer() { var x = 1; return function () { return x; }; } let f = outer(); f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("var closure should run");
    assert!(outcome.note.contains("number(1"));
}

#[test]
fn wasm_backend_supports_nested_recursive_closure_call() {
    let outcome = engine()
            .run_script(
                "function outer(n) { function loop(x) { if (x === 0) { return 0; } return loop(x - 1) + 1; } return loop(n); } outer(3);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("nested recursive function should run");
    assert!(outcome.note.contains("number(3"));
}

#[test]
fn wasm_backend_supports_arrow_function_basic() {
    let outcome = engine()
        .run_script(
            "let f = x => x + 1; f(2);",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("arrow function should run");
    assert!(outcome.note.contains("number(3"));
}

#[test]
fn wasm_backend_supports_arrow_function_block_body() {
    let outcome = engine()
        .run_script(
            "let f = x => { return x + 1; }; f(2);",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("arrow block body should run");
    assert!(outcome.note.contains("number(3"));
}

#[test]
fn wasm_backend_supports_named_function_expression_recursion() {
    let outcome = engine()
            .run_script(
                "let f = function fact(n) { if (n === 0) { return 1; } return n * fact(n - 1); }; f(4);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("named function expression should run");
    assert!(outcome.note.contains("number(24"));
}

#[test]
fn wasm_backend_supports_returned_arrow_closure_call() {
    let outcome = engine()
        .run_script(
            "function outer(x) { return y => x + y; } let f = outer(2); f(3);",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("returned arrow closure should run");
    assert!(outcome.note.contains("number(5"));
}

#[test]
fn wasm_backend_supports_implicit_undefined_function_return() {
    let outcome = engine()
        .run_script(
            "function f() { let x = 1; } f();",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should run implicit undefined function");
    assert!(outcome.note.contains("undefined("));
}

#[test]
fn wasm_backend_supports_class_inheritance_and_private_core() {
    for (source, expected) in [
        (
            "class A { constructor(v) { this.x = v; } } class B extends A { constructor() { super(3); } } new B().x;",
            "number(3)",
        ),
        (
            "class C { #x = 1; getX() { return this.#x; } } new C().getX();",
            "number(1)",
        ),
        (
            "class C { #m() { return 2; } getX() { return this.#m(); } } new C().getX();",
            "number(2)",
        ),
        (
            "class C { get #x() { return 3; } read() { return this.#x; } } new C().read();",
            "number(3)",
        ),
        (
            "class C { static #x = 4; static read() { return C.#x; } } C.read();",
            "number(4)",
        ),
        (
            "class C { #x; static has(obj) { return #x in obj; } } let c = new C(); C.has(c);",
            "boolean(true)",
        ),
        (
            "class A { m() { return 1; } } class B extends A { m() { return super.m() + 1; } } new B().m();",
            "number(2)",
        ),
        (
            "let C = class Self { static make() { return new Self(); } }; C.make() instanceof C;",
            "boolean(true)",
        ),
    ] {
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("phase 22B class case should run for `{source}`: {err:?}")
            });
        assert!(
            outcome.note.contains(expected),
            "source: {source}, note: {}",
            outcome.note
        );
    }
}

#[test]
fn wasm_backend_supports_phase_twenty_eight_bind_and_error_tostring() {
    for (source, expected) in [
        (
            "function add(x, y) { return x + y; } let inc = add.bind(null, 1); inc(2);",
            "number(3)",
        ),
        (
            "function f() { return this; } let o = { v: 2 }; let g = f.bind(o, 3); g() === o;",
            "boolean(true)",
        ),
        (
            "function f() { return this; } let o = { v: 2 }; let g = f.bind(o, 3); g.call({ v: 9 }, 4) === o;",
            "boolean(true)",
        ),
        (
            "function outer() { return (() => this.x).bind({ x: 9 }); } let o = { x: 3, f: outer }; let g = o.f(); g();",
            "number(3)",
        ),
        (
            "function F(x) { this.x = x; } let G = F.bind(null, 2); new G().x;",
            "number(2)",
        ),
        (
            "function F() {} let G = F.bind(null); new G() instanceof F;",
            "boolean(true)",
        ),
        (
            "class C {} let B = C.bind(null); try { B(); } catch (e) { e instanceof TypeError; }",
            "boolean(true)",
        ),
        ("Error(\"x\").toString();", "string(Error: x)"),
        ("TypeError(\"x\").toString();", "string(TypeError: x)"),
        ("let e = new Error(); e.toString();", "string(Error)"),
        (
            "Error.prototype.toString.call({ name: \"X\", message: \"y\" });",
            "string(X: y)",
        ),
        (
            "Error.prototype.toString.call({ name: \"\", message: \"y\" });",
            "string(y)",
        ),
    ] {
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("phase 28 bind/error-string case should run for `{source}`: {err:?}")
            });
        assert!(
            outcome.note.contains(expected),
            "source: {source}, note: {}",
            outcome.note
        );
    }
    let outcome = engine()
        .run_script(
            "try { \"x\" in 1; } catch (e) { e.toString(); }",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("phase 28 TypeError toString case should run: {err:?}"));
    assert!(
        outcome.note.contains("string(TypeError:"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_phase_twenty_four_class_edges_follow_the_specification() {
    // These edges used to be locked as unsupported. The two class
    // definitions below now compile and are checked against their
    // specified results instead.
    for (source, expected) in [
        // A heritage chosen at run time between an ordinary constructor and
        // a non-constructor still yields an ordinary derived instance.
        (
            "let H; if (true) { H = function() {}; } else { H = print; } class C extends H {} \
                 let c = new C(); \
                 c instanceof C && c instanceof H && Object.getPrototypeOf(C) === H \
                 && Object.getPrototypeOf(C.prototype) === H.prototype;",
            "boolean(true)",
        ),
        // `delete super.x` is valid syntax; the delete operator throws a
        // ReferenceError for a super Reference only when evaluated, after
        // the Reference and any computed key have been evaluated, including
        // when the home object's prototype is null.
        (
            "class C extends Object { m() { delete super.x; } } \
                 let count = 0; \
                 class D { static m() { delete super[(count++, 'x')]; } } \
                 Object.setPrototypeOf(D, null); \
                 let results = []; \
                 for (let call of [() => new C().m(), () => D.m()]) { \
                   try { call(); results.push('returned'); } \
                   catch (error) { results.push(error.constructor === ReferenceError); } \
                 } \
                 results.join() + ':' + count;",
            "string(true,true:1)",
        ),
    ] {
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| panic!("class edge should run for `{source}`: {err:?}"));
        assert!(
            outcome.note.contains(expected),
            "source: {source}, note: {}",
            outcome.note
        );
    }

    // `extends null` installs %Function.prototype% as the constructor
    // parent, so the implicit `super(...args)` of the derived default
    // constructor throws a TypeError.
    let err = engine()
        .run_script(
            "let H; if (true) { H = null; } else { H = Object; } class C extends H {} new C();",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect_err("constructing a class that extends null should throw");
    assert_eq!(
        err.wasm_javascript_exception_constructor_name(),
        Some("TypeError"),
        "err: {}",
        err.message()
    );

    // Early errors remain parse-time SyntaxErrors.
    for source in ["new.target;", "class C { #x; m(obj) { delete obj.#x; } }"] {
        let err = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("an early error must reject the script");
        assert!(
            err.message().contains("parse error"),
            "source: {source}, err: {}",
            err.message()
        );
    }
}

#[test]
fn wasm_backend_map_constructor_and_methods_enforce_internal_slots() {
    let source = r#"
            var map = new Map(null);
            var undefinedMap = new Map(undefined);
            class DerivedMap extends Map {}
            var derived = new DerivedMap();
            var checks = [
                typeof Map,
                Object.getPrototypeOf(map) === Map.prototype,
                undefinedMap.size === 0,
                Object.getPrototypeOf(derived) === DerivedMap.prototype,
                map instanceof Map,
                map.size === 0
            ];
            try { Map(); checks.push(false); }
            catch (error) { checks.push(error instanceof TypeError); }
            try { Map.prototype.get.call({}); checks.push(false); }
            catch (error) { checks.push(error instanceof TypeError); }
            try {
                Object.getOwnPropertyDescriptor(Map.prototype, 'size').get.call([]);
                checks.push(false);
            } catch (error) { checks.push(error instanceof TypeError); }
            checks.join('|');
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Map constructor and methods should enforce their runtime contracts");
    assert!(
        outcome
            .note
            .contains("string(function|true|true|true|true|true|true|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_set_predicates_compare_values_and_expose_spec_function_shapes() {
    let outcome = engine()
        .run_script(
            r#"
var left = new Set([1, 2, NaN, -0]);
var subset = new Set([2, +0]);
var notConstructors = true;
for (var method of [
  Set.prototype.isDisjointFrom,
  Set.prototype.isSubsetOf,
  Set.prototype.isSupersetOf
]) {
  try { new method(new Set()); notConstructors = false; } catch (error) {
    notConstructors = notConstructors && error instanceof TypeError;
  }
}
[
  Set.prototype.isDisjointFrom.name,
  Set.prototype.isDisjointFrom.length,
  Set.prototype.isSubsetOf.name,
  Set.prototype.isSubsetOf.length,
  Set.prototype.isSupersetOf.name,
  Set.prototype.isSupersetOf.length,
  left.isDisjointFrom(new Set([3, 4])),
  left.isDisjointFrom(new Set([4, NaN])),
  subset.isSubsetOf(left),
  left.isSubsetOf(subset),
  left.isSupersetOf(subset),
  subset.isSupersetOf(left),
  left.isSupersetOf(new Map([[1, "one"], [2, "two"]])),
  notConstructors
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Set predicates should compare values with Set semantics");
    assert!(
            outcome.note.contains(
                "string(isDisjointFrom|1|isSubsetOf|1|isSupersetOf|1|true|false|true|false|true|false|true|true)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_set_constructor_and_methods_enforce_internal_slots() {
    let source = r#"
            var set = new Set(null);
            var undefinedSet = new Set(undefined);
            class DerivedSet extends Set {}
            var derived = new DerivedSet();
            var checks = [
                typeof Set,
                Object.getPrototypeOf(set) === Set.prototype,
                undefinedSet.size === 0,
                Object.getPrototypeOf(derived) === DerivedSet.prototype,
                set instanceof Set,
                set.size === 0
            ];
            try { Set(); checks.push(false); }
            catch (error) { checks.push(error instanceof TypeError); }
            try { Set.prototype.add.call({}); checks.push(false); }
            catch (error) { checks.push(error instanceof TypeError); }
            try { Set.prototype.has.call(new Map()); checks.push(false); }
            catch (error) { checks.push(error instanceof TypeError); }
            try {
                Object.getOwnPropertyDescriptor(Set.prototype, 'size').get.call([]);
                checks.push(false);
            } catch (error) { checks.push(error instanceof TypeError); }
            checks.join('|');
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Set constructor and methods should enforce their runtime contracts");
    assert!(
        outcome
            .note
            .contains("string(function|true|true|true|true|true|true|true|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_promise_constructor_calls_executor_synchronously() {
    let source = r#"
            let resolve;
            let reject;
            let calls = 0;
            let promise = new Promise(function (onFulfilled, onRejected) {
                calls += 1;
                resolve = onFulfilled;
                reject = onRejected;
            });
            calls === 1
                && typeof promise === "object"
                && Object.getPrototypeOf(promise) === Promise.prototype
                && promise.constructor === Promise
                && typeof resolve === "function"
                && typeof reject === "function"
                && resolve !== reject
                && resolve.length === 1
                && reject.length === 1
                && resolve(42) === undefined
                && reject(9) === undefined;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("Promise constructor should run: {err:?}"));
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_promise_constructor_rejects_invalid_invocations() {
    let source = r#"
            let nonCallableThrows = false;
            let missingNewThrows = false;
            let prototypeAccessed = false;
            let orderIsCorrect = false;
            try { new Promise(1); } catch (error) {
                nonCallableThrows = error instanceof TypeError;
            }
            try { Promise(function () {}); } catch (error) {
                missingNewThrows = error instanceof TypeError;
            }
            let newTarget = (function () {}).bind();
            Object.defineProperty(newTarget, "prototype", {
                get: function () {
                    prototypeAccessed = true;
                    throw new Error();
                }
            });
            try { Reflect.construct(Promise, [], newTarget); } catch (error) {
                orderIsCorrect = error instanceof TypeError && !prototypeAccessed;
            }
            nonCallableThrows && missingNewThrows && orderIsCorrect;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("invalid Promise constructor uses should run: {err:?}"));
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_async_function_returns_promise_before_its_reaction_runs() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            async function immediate() { return 7; }
            let returned = immediate();
            print(returned instanceof Promise);
            returned.then(function (value) { print("fulfilled:" + value); });
            print("sync");
        "#;
    engine_with_captured_prints(Arc::clone(&lines))
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("async function should return a Promise: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &[
            "true".to_string(),
            "sync".to_string(),
            "fulfilled:7".to_string(),
        ]
    );
}

#[test]
fn wasm_backend_async_function_converts_synchronous_throw_to_rejection() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            async function thrown() { throw "sync"; }
            thrown().then(undefined, function (reason) { print("rejected:" + reason); });
            print("sync");
        "#;
    engine_with_captured_prints(Arc::clone(&lines))
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("async throw should reject its result Promise: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["sync".to_string(), "rejected:sync".to_string()]
    );
}

#[test]
fn wasm_backend_async_function_expression_resumes_lexical_state() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            let expression = async function (value) {
                let retained = value;
                await Promise.resolve();
                return retained + 1;
            };
            let returned = expression(41);
            print(returned instanceof Promise);
            returned.then(function (value) { print("fulfilled:" + value); });
            print("sync");
        "#;
    engine_with_captured_prints(Arc::clone(&lines))
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("async function expression should resume: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &[
            "true".to_string(),
            "sync".to_string(),
            "fulfilled:42".to_string(),
        ]
    );
}

#[test]
fn wasm_backend_async_class_methods_preserve_receiver_super_and_method_semantics() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            class Base {
                method() { return this.marker; }
                static staticMethod() { return this.marker; }
            }
            class Derived extends Base {
                async method(delta) {
                    await Promise.resolve();
                    return super.method() + delta;
                }
                static async staticMethod(delta) {
                    await Promise.resolve();
                    return super.staticMethod() + delta;
                }
            }
            const instance = new Derived();
            instance.marker = 40;
            Derived.marker = 41;

            print(instance.method.name);
            print(Derived.staticMethod.name);
            const instanceResult = instance.method(2);
            const staticResult = Derived.staticMethod(2);
            print(instanceResult instanceof Promise);
            print(staticResult instanceof Promise);
            try { new instance.method(); } catch (error) {
                print(error instanceof TypeError);
            }
            try { new Derived.staticMethod(); } catch (error) {
                print(error instanceof TypeError);
            }
            instanceResult.then(function (value) { print("instance:" + value); });
            staticResult.then(function (value) { print("static:" + value); });
            print("sync");
        "#;
    engine_with_captured_prints(Arc::clone(&lines))
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("async class methods should execute: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &[
            "method".to_string(),
            "staticMethod".to_string(),
            "true".to_string(),
            "true".to_string(),
            "true".to_string(),
            "true".to_string(),
            "sync".to_string(),
            "instance:42".to_string(),
            "static:43".to_string(),
        ]
    );
}

#[test]
fn wasm_backend_promise_catch_propagates_lookup_and_call_errors() {
    let source = r#"
            let catchMethod = Promise.prototype.catch;
            let lookupError = {};
            let callError = {};
            let lookupResult;
            let callResult;
            let nullThrows = false;
            try { catchMethod.call(null); } catch (error) {
                nullThrows = error instanceof TypeError;
            }
            let poisoned = {};
            Object.defineProperty(poisoned, "then", {
                get: function () { throw lookupError; }
            });
            try { catchMethod.call(poisoned); } catch (error) {
                lookupResult = error;
            }
            try {
                catchMethod.call({
                    then: function () { throw callError; }
                });
            } catch (error) {
                callResult = error;
            }
            nullThrows && lookupResult === lookupError && callResult === callError;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("Promise catch errors should propagate: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_promise_then_uses_default_constructor_for_nullish_species() {
    let source = r#"
            let constructorReads = 0;
            let defaultConstructor = new Promise(function () {});
            Object.defineProperty(defaultConstructor, "constructor", {
                get: function () {
                    constructorReads += 1;
                    return Promise;
                }
            });
            let undefinedConstructorReads = 0;
            let undefinedConstructor = new Promise(function () {});
            Object.defineProperty(undefinedConstructor, "constructor", {
                get: function () {
                    undefinedConstructorReads += 1;
                    return undefined;
                }
            });

            let nullSpeciesReads = 0;
            let nullSpeciesConstructor = {};
            Object.defineProperty(nullSpeciesConstructor, Symbol.species, {
                get: function () {
                    nullSpeciesReads += 1;
                    return null;
                }
            });
            let nullSpecies = new Promise(function () {});
            nullSpecies.constructor = nullSpeciesConstructor;

            let undefinedSpeciesConstructor = {};
            undefinedSpeciesConstructor[Symbol.species] = undefined;
            let undefinedSpecies = new Promise(function () {});
            undefinedSpecies.constructor = undefinedSpeciesConstructor;

            let first = defaultConstructor.then();
            let second = undefinedConstructor.then();
            let third = nullSpecies.then();
            let fourth = undefinedSpecies.then();
            constructorReads === 1
                && undefinedConstructorReads === 1
                && nullSpeciesReads === 1
                && first instanceof Promise
                && second instanceof Promise
                && third instanceof Promise
                && fourth instanceof Promise;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("nullish Promise species defaults should run: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_promise_then_propagates_constructor_and_species_errors() {
    let source = r#"
            let constructorError = {};
            let speciesError = {};
            let poisonedConstructor = new Promise(function () {});
            Object.defineProperty(poisonedConstructor, "constructor", {
                get: function () { throw constructorError; }
            });
            let poisonedSpeciesConstructor = {};
            Object.defineProperty(poisonedSpeciesConstructor, Symbol.species, {
                get: function () { throw speciesError; }
            });
            let poisonedSpecies = new Promise(function () {});
            poisonedSpecies.constructor = poisonedSpeciesConstructor;
            let nullConstructor = new Promise(function () {});
            nullConstructor.constructor = null;
            let invalidSpecies = new Promise(function () {});
            invalidSpecies.constructor = {};
            invalidSpecies.constructor[Symbol.species] = {};
            let constructionError = {};
            let throwingSpecies = new Promise(function () {});
            throwingSpecies.constructor = {};
            throwingSpecies.constructor[Symbol.species] = function () {
                throw constructionError;
            };

            let constructorResult;
            let speciesResult;
            let constructionResult;
            let nullThrows = false;
            let invalidSpeciesThrows = false;
            try { poisonedConstructor.then(); } catch (error) { constructorResult = error; }
            try { poisonedSpecies.then(); } catch (error) { speciesResult = error; }
            try { nullConstructor.then(); } catch (error) {
                nullThrows = error instanceof TypeError;
            }
            try { invalidSpecies.then(); } catch (error) {
                invalidSpeciesThrows = error instanceof TypeError;
            }
            try { throwingSpecies.then(); } catch (error) { constructionResult = error; }
            constructorResult === constructorError
                && speciesResult === speciesError
                && constructionResult === constructionError
                && nullThrows
                && invalidSpeciesThrows;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("Promise species errors should propagate: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_promise_static_methods_validate_their_constructor_receiver() {
    let source = r#"
            let resolveThrows = false;
            let rejectThrows = false;
            try { Promise.resolve.call({}, 1); } catch (error) {
                resolveThrows = error instanceof TypeError;
            }
            try { Promise.reject.call(function () {}, 1); } catch (error) {
                rejectThrows = error instanceof TypeError;
            }
            resolveThrows && rejectThrows;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("Promise static receiver validation should run: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_promise_resolve_preserves_same_constructor_promise_identity() {
    let source = r#"
            let promise = new Promise(function () {});
            Promise.resolve(promise) === promise;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("Promise.resolve identity should run: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_promise_with_resolvers_uses_generic_constructor_capability() {
    let source = r#"
            let returnedPromise = {};
            let expectedResolve = function () {};
            let expectedReject = function () {};
            let constructorCalls = 0;
            function Capability(executor) {
                constructorCalls += 1;
                executor(expectedResolve, expectedReject);
                return returnedPromise;
            }
            let capability = Promise.withResolvers.call(Capability);
            constructorCalls === 1
                && capability.promise === returnedPromise
                && capability.resolve === expectedResolve
                && capability.reject === expectedReject;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| {
            panic!("Promise.withResolvers generic capability should run: {err:?}")
        });
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_promise_try_uses_generic_constructor_capability() {
    let source = r#"
            let returnedPromise = {};
            let sentinel = {};
            let constructorCalls = 0;
            let callbackPrecededConstructor = false;
            let resolvedValue;
            let resolveThis;
            let rejectCalls = 0;
            function Capability(executor) {
                constructorCalls += 1;
                executor(function (value) {
                    "use strict";
                    resolveThis = this;
                    resolvedValue = value;
                }, function () { rejectCalls += 1; });
                return returnedPromise;
            }
            let result = Promise.try.call(Capability, function () {
                callbackPrecededConstructor = constructorCalls === 0;
                return sentinel;
            });
            result === returnedPromise
                && callbackPrecededConstructor
                && constructorCalls === 1
                && resolvedValue === sentinel
                && resolveThis === undefined
                && rejectCalls === 0;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("Promise.try generic capability should run: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_promise_all_settles_the_custom_constructor_capability() {
    let source = r#"
            let resolved;
            let capabilityObject;
            function CustomPromise(executor) {
                capabilityObject = {};
                executor(function (value) { resolved = value; }, function () {});
                return capabilityObject;
            }
            CustomPromise.resolve = function (value) {
                return { then: function (onFulfilled) { onFulfilled(value); } };
            };
            let result = Promise.all.call(CustomPromise, [1, 2]);
            result === capabilityObject &&
                Array.isArray(resolved) &&
                resolved[0] === 1 &&
                resolved[1] === 2;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("custom Promise.all capability should run: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_resumes_generator_return_call_arguments() {
    let source = r#"
            let calls = 0;
            const generator = function* g() {
                calls += 1;
                return (function(value) {
                    const yield = value + 1;
                    return yield;
                }(yield));
            };
            const iterator = generator();
            const suspended = iterator.next();
            const completed = iterator.next(42);
            suspended.value === undefined && suspended.done === false
                && completed.value === 43 && completed.done === true
                && calls === 1;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("generator return argument should resume: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_initializes_generator_parameters_at_call_time() {
    let source = r#"
            let effects = "";
            function* initialized(value = (effects += "p", 3), ...rest) {
                effects += "b";
                yield value;
                yield rest[1];
            }
            const iterator = initialized(undefined, 4, 5);
            const initializedBeforeResume = effects === "p";
            const first = iterator.next();
            const second = iterator.next();

            function* destructured([left], { right }) {
                effects += "d";
                yield left + right;
            }
            const destructuredIterator = destructured([2], { right: 3 });
            const destructuredBeforeResume = effects === "pb";
            const destructuredYield = destructuredIterator.next();

            const intrinsicGeneratorPrototype = Object.getPrototypeOf(
                Object.getPrototypeOf((function* () {})())
            );
            function* topology(value = (topology.prototype = null)) { yield value; }
            const topologyIterator = topology();

            let thrownValue = 0;
            function* abrupt(value = (() => { throw 9; })()) {
                effects += "x";
                yield value;
            }
            try { abrupt(); } catch (error) { thrownValue = error; }

            initializedBeforeResume
                && effects === "pbd"
                && first.value === 3 && first.done === false
                && second.value === 5 && second.done === false
                && destructuredBeforeResume
                && destructuredYield.value === 5 && destructuredYield.done === false
                && Object.getPrototypeOf(topologyIterator) === intrinsicGeneratorPrototype
                && thrownValue === 9;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("generator parameters should initialize: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_supports_generator_function_has_instance() {
    let source = r#"
            const generator = function* () {};
            const arrow = () => {};
            let arrowThrows = false;
            try {
                ({} instanceof arrow);
            } catch (error) {
                arrowThrows = error instanceof TypeError;
            }

            generator() instanceof generator && arrowThrows;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("generator instanceof should compile: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}
