use super::*;

#[cfg(feature = "spec-exec-oracle")]
#[test]
fn observed_spec_exec_delegates_and_captures_without_revealing_symbol_description() {
    let delegated_lines = Arc::new(Mutex::new(Vec::new()));
    let engine = engine_with_captured_prints(Arc::clone(&delegated_lines));
    let outcome = engine
        .observe_script(
            "print('oracle line'); throw Symbol('t25-private-description');",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::SpecExec,
                ..RunOptions::default()
            },
        )
        .expect("spec-exec throw should be an observed completion");

    assert_eq!(
        outcome.completion,
        ObservedCompletion::Throw(ObservedJsValue::Symbol)
    );
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine("oracle line".to_string())]
    );
    assert_eq!(outcome.note, "uncaught ECMAScript throw (symbol)");
    assert!(!format!("{outcome:?}").contains("t25-private-description"));
    assert_eq!(
        *delegated_lines.lock().expect("capture mutex poisoned"),
        vec!["oracle line".to_string()]
    );
}

#[test]
fn wasm_backend_json_parse_duplicate_proto_literal_property() {
    let outcome = engine()
            .run_script(
                "var value = JSON.parse('{\"__proto__\": 1, \"__proto__\": 2}'); typeof value === 'object' && value.__proto__ === 2;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("JSON.parse duplicate __proto__ should run");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_json_parse_dynamic_reviver_accepts_callable_proxy() {
    let outcome = engine()
        .run_script(
            r#"
var calls = 0;
var sourceIsExact = false;
var reviver = new Proxy(function(key, value, context) {
  calls += 1;
  sourceIsExact = key === "" && context.source === "1.0";
  return value;
}, {});
function parse(text, callback) { return JSON.parse(text, callback); }
var result = parse("  1.0  ", reviver);
result === 1 && calls === 1 && sourceIsExact;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("callable Proxy JSON reviver should run");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_json_parse_dynamic_reviver_preserves_array_accessor_throw_identity() {
    let outcome = engine()
        .run_script(
            r#"
var accessorSentinel = {};
function parseWithAccessorThrow() {
  return JSON.parse("[0,0]", function() {
    Object.defineProperty(this, "1", {
      get: function() { throw accessorSentinel; }
    });
  });
}
var identityPreserved = false;
try { parseWithAccessorThrow(); } catch (error) {
  identityPreserved = error === accessorSentinel;
}
identityPreserved;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("dynamic JSON reviver array accessor throw should propagate");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_json_stringify_normalizes_property_list_entries_in_first_seen_order() {
    let outcome = engine()
        .run_script(
            r#"
var conversions = [];
var boxedString = new String("ignored");
boxedString.toString = function() {
  conversions.push("string");
  return "b";
};
var boxedNumber = new Number(7);
Object.setPrototypeOf(boxedNumber, {
  toString: function() {
    conversions.push("number");
    return "a";
  }
});
var replacer = [
  "c", boxedString, boxedNumber, "c", 1, -0, NaN, Infinity,
  undefined, null, true, {}, Symbol("ignored")
];
var value = { a: 1, b: 2, c: 3, 1: 4, 0: 5, NaN: 6, Infinity: 7 };
[
  JSON.stringify(value, replacer),
  conversions.join(","),
  JSON.stringify({ a: 1 }, function(key, current) {
    return key === "a" ? current + 1 : current;
  })
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("JSON stringify should normalize property list entries once");
    assert!(
            outcome.note.contains(
                "string({\"c\":3,\"b\":2,\"a\":1,\"1\":4,\"0\":5,\"NaN\":6,\"Infinity\":7}|string,number|{\"a\":2})"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_json_stringify_calls_proxy_replacer_with_exact_holders_and_arguments() {
    let outcome = engine()
        .run_script(
            r#"
var child = { value: 1 };
var input = { child: child };
var rootWrapper;
var calls = [];
var holders = [];
var applyHolders = [];
var target = function(key, value) {
  calls.push(key);
  if (key === "") {
    rootWrapper = this;
    holders.push(this !== input && this[""] === input);
  } else if (key === "child") {
    holders.push(this === input);
  } else if (key === "value") {
    holders.push(this === child);
    return value + 1;
  }
  return value;
};
var replacer = new Proxy(target, {
  apply: function(target, thisArg, args) {
    if (args[0] === "") {
      applyHolders.push(thisArg !== input && thisArg[""] === input);
    } else if (args[0] === "child") {
      applyHolders.push(thisArg === input);
    } else if (args[0] === "value") {
      applyHolders.push(thisArg === child);
    }
    return target.apply(thisArg, args);
  }
});
var result = JSON.stringify(input, replacer);

var sentinel = {};
var abruptIdentity = false;
try {
  JSON.stringify({}, new Proxy(function() {}, {
    apply: function() { throw sentinel; }
  }));
} catch (error) {
  abruptIdentity = error === sentinel;
}

var revoked = Proxy.revocable(function() {}, {});
revoked.revoke();
var revokedTypeError = false;
try {
  JSON.stringify({}, revoked.proxy);
} catch (error) {
  revokedTypeError = error instanceof TypeError;
}

[
  result,
  calls.join(","),
  holders.join(","),
  applyHolders.join(","),
  Object.getPrototypeOf(rootWrapper) === Object.prototype,
  abruptIdentity,
  revokedTypeError
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("JSON stringify should call Proxy replacers with exact holders");
    assert!(
            outcome.note.contains(
                "string({\"child\":{\"value\":2}}|,child,value|true,true,true|true,true,true|true|true|true)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_json_stringify_calls_proxy_to_json_and_omits_callable_proxies() {
    let outcome = engine()
        .run_script(
            r#"
var callable = new Proxy(function() {}, {});
var order = [];
var value = {};
value.toJSON = new Proxy(function(key) {
  order.push("toJSON:" + key);
  return { kept: 1, dropped: callable };
}, {
  apply: function(target, thisArg, args) {
    order.push("toJSONApply:" + args[0] + ":" + (thisArg === value));
    return target.apply(thisArg, args);
  }
});
var result = JSON.stringify(value, function(key, current) {
  order.push("replacer:" + key);
  return key === "kept" ? current + 1 : current;
});

var getterSentinel = {};
var getterIdentity = false;
var replacerCalled = false;
var abruptToJSON = {};
Object.defineProperty(abruptToJSON, "toJSON", {
  get: function() { throw getterSentinel; }
});
try {
  JSON.stringify(abruptToJSON, function(key, current) {
    replacerCalled = true;
    return current;
  });
} catch (error) {
  getterIdentity = error === getterSentinel;
}

var hole = [];
hole.length = 1;
[
  result,
  order.join(","),
  JSON.stringify(callable) === undefined,
  JSON.stringify([callable]),
  JSON.stringify({ callable: callable }),
  JSON.stringify(0, function() { return callable; }) === undefined,
  JSON.stringify([0], function(key, current) {
    return key === "0" ? callable : current;
  }),
  JSON.stringify({ value: 0 }, function(key, current) {
    return key === "value" ? callable : current;
  }),
  JSON.stringify(hole, function(key, current) {
    return key === "0" ? "filled" : current;
  }),
  getterIdentity,
  replacerCalled
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("JSON stringify should call Proxy toJSON and omit callable Proxies");
    assert!(
            outcome.note.contains(
                "string({\"kept\":2}|toJSONApply::true,toJSON:,replacer:,replacer:kept,replacer:dropped|true|[null]|{}|true|[null]|{}|[\"filled\"]|true|false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_json_stringify_escapes_ascii_property_names_and_values() {
    let outcome = engine()
        .run_script(
            r#"
var input = "Az\"\\\x00\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x1f";
var expected = "\"Az\\\"\\\\\\u0000\\u0007\\b\\t\\n\\u000b\\f\\r\\u000e\\u001f\"";
var object = {};
object[input] = input;
JSON.stringify(input) === expected &&
  JSON.stringify(object) === "{" + expected + ":" + expected + "}";
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("JSON stringify should escape ASCII property names and values");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_object_has_own_checks_to_object_before_key() {
    let outcome = engine()
            .run_script(
                "var calls = 0; var key = { toString: function() { calls = calls + 1; throw 'key'; } }; var ok = false; try { Object.hasOwn(undefined, key); } catch (e) { ok = e.name === 'TypeError'; } ok && calls === 0 && Object.hasOwn({x: 1}, 'x');",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Object.hasOwn ToObject ordering case should run");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_object_has_own_observes_property_key_and_descriptor_semantics() {
    let outcome = engine()
        .run_script(
            r#"
var symbol = Symbol("owned");
var otherSymbol = Symbol("owned");
var target = {};
Object.defineProperty(target, "value", {
  get: function() { throw "property getter must not run"; }
});
target[symbol] = 1;
var calls = [];
var proxy = new Proxy(target, {
  getOwnPropertyDescriptor: function(actualTarget, key) {
    calls.push("descriptor:" + (key === symbol ? "symbol" : key));
    return Reflect.getOwnPropertyDescriptor(actualTarget, key);
  }
});
var symbolKey = {};
symbolKey[Symbol.toPrimitive] = function(hint) {
  calls.push("key:" + hint);
  return symbol;
};
var marker = {};
var abruptIdentity = false;
try {
  Object.hasOwn(new Proxy({}, {
    getOwnPropertyDescriptor: function() { throw marker; }
  }), "x");
} catch (error) {
  abruptIdentity = error === marker;
}
var other = __scriptwCreateRealm().global;
var otherRealmError = false;
var otherHasOwn = other.Object.hasOwn;
try {
  otherHasOwn(null, "x");
} catch (error) {
  otherRealmError =
    Object.getPrototypeOf(error) === other.TypeError.prototype &&
    !(error instanceof TypeError);
}
function checkedHasOwn(label, value, key) {
  try {
    return label + ":" + Object.hasOwn(value, key);
  } catch (error) {
    return label + ":error:" + error.name;
  }
}
[
  checkedHasOwn("string-zero", "abcdef", "0"),
  checkedHasOwn("string-five", "abcdef", "5"),
  checkedHasOwn("string-length", "abcdef", "length"),
  checkedHasOwn("array-five", [0, 1, 2, 3, 4, 5], "5"),
  checkedHasOwn("proxy-value", proxy, "value"),
  checkedHasOwn("proxy-symbol", proxy, symbolKey),
  checkedHasOwn("other-symbol", target, otherSymbol),
  calls.join(","),
  abruptIdentity,
  otherRealmError
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.hasOwn should use ToObject, ToPropertyKey, and [[GetOwnProperty]]");
    assert!(
            outcome.note.contains(
                "string(string-zero:true|string-five:true|string-length:true|array-five:true|proxy-value:true|proxy-symbol:true|other-symbol:false|descriptor:value,key:string,descriptor:symbol|true|true)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_proxy_descriptor_invariants_preserve_symbol_identity() {
    let outcome = engine()
        .run_script(
            r#"
var ownedSymbol = Symbol("same description");
var distinctSymbol = Symbol("same description");
var target = {};
Object.defineProperty(target, ownedSymbol, {
  value: 1,
  configurable: false
});
var proxy = new Proxy(target, {
  getOwnPropertyDescriptor: function(actualTarget, key) {
    return Reflect.getOwnPropertyDescriptor(actualTarget, key);
  }
});
var iteratorTarget = {
  "Symbol.asyncIterator": "string"
};
iteratorTarget[Symbol.asyncIterator] = "symbol";
var iteratorKeys = Reflect.ownKeys(iteratorTarget);
[
  Reflect.getOwnPropertyDescriptor(proxy, distinctSymbol) === undefined,
  Object.hasOwn(proxy, distinctSymbol),
  Object.hasOwn(proxy, ownedSymbol),
  Object.hasOwn(iteratorTarget, "Symbol.asyncIterator"),
  Object.hasOwn(iteratorTarget, Symbol.asyncIterator),
  iteratorTarget["Symbol.asyncIterator"] === "string",
  iteratorTarget[Symbol.asyncIterator] === "symbol",
  iteratorKeys[0] === "Symbol.asyncIterator",
  iteratorKeys[1] === Symbol.asyncIterator
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Proxy descriptor invariants should compare Symbols by identity");
    assert!(
        outcome
            .note
            .contains("string(true|false|true|true|true|true|true|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_computed_symbol_keys_stay_distinct_from_similar_strings() {
    let outcome = engine()
        .run_script(
            r#"
var object = {
  [Symbol.iterator]: 1,
  [Symbol.toPrimitive]() {
    return 2;
  },
  "Symbol.iterator": 3
};
object[Symbol.iterator]++;
var argumentIterator = (function() {
  return arguments[Symbol.iterator];
})(1);
[
  object[Symbol.iterator],
  object["Symbol.iterator"],
  object[Symbol.toPrimitive](),
  Object.getOwnPropertyNames(object).length,
  Object.getOwnPropertySymbols(object).length,
  [][Symbol.iterator] === Array.prototype.values,
  ""[Symbol.iterator] === String.prototype[Symbol.iterator],
  argumentIterator === Array.prototype.values
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("computed Symbols should stay distinct from similarly spelled strings");
    assert!(
        outcome.note.contains("string(2|3|2|1|2|true|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_object_create_validates_and_preserves_the_prototype_kind() {
    let outcome = engine()
        .run_script(
            r#"
function FunctionPrototype() {}
var functionChild = Object.create(FunctionPrototype);
var arrayPrototype = [];
var arrayChild = Object.create(arrayPrototype);
var missingError = false;
try {
  Object.create();
} catch (error) {
  missingError = error instanceof TypeError;
}
var primitiveError = false;
try {
  Object.create(1);
} catch (error) {
  primitiveError = error instanceof TypeError;
}
var other = __scriptwCreateRealm().global;
var otherRealmError = false;
try {
  other.Object.create("invalid");
} catch (error) {
  otherRealmError =
    Object.getPrototypeOf(error) === other.TypeError.prototype &&
    !(error instanceof TypeError);
}
var descriptorObject = Object.create(null, {
  value: {
    value: 7,
    enumerable: true
  }
});
function choosePrototype(useObject) {
  if (useObject) {
    return { inherited: 9 };
  }
  return null;
}
var mixedPrototype = choosePrototype(globalThis === globalThis);
var mixedChild = Object.create(mixedPrototype);
[
  Object.getPrototypeOf(functionChild) === FunctionPrototype,
  Object.getPrototypeOf(arrayChild) === arrayPrototype,
  missingError,
  primitiveError,
  otherRealmError,
  Object.getPrototypeOf(descriptorObject) === null,
  descriptorObject.value,
  Object.keys(descriptorObject)[0],
  Object.getPrototypeOf(mixedChild) === mixedPrototype,
  mixedChild.inherited
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.create should validate and preserve its prototype");
    assert!(
        outcome
            .note
            .contains("string(true|true|true|true|true|true|7|value|true|9)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_object_descriptor_maps_convert_before_defining_properties() {
    let outcome = engine()
        .run_script(
            r#"
var target = {};
var order = [];
var firstDescriptor = {};
Object.defineProperty(firstDescriptor, "enumerable", {
  get: function() {
    order.push("first:enumerable");
    return true;
  }
});
Object.defineProperty(firstDescriptor, "value", {
  get: function() {
    order.push("first:value");
    return 1;
  }
});
var secondDescriptor = {};
Object.defineProperty(secondDescriptor, "enumerable", {
  get: function() {
    order.push("second:enumerable");
    return true;
  }
});
Object.defineProperty(secondDescriptor, "value", {
  get: function() {
    order.push("second:value:" + Object.hasOwn(target, "first"));
    return 2;
  }
});
var descriptorMapTarget = {};
Object.defineProperty(descriptorMapTarget, "first", {
  enumerable: true,
  get: function() {
    order.push("get:first");
    return firstDescriptor;
  }
});
Object.defineProperty(descriptorMapTarget, "second", {
  enumerable: true,
  get: function() {
    order.push("get:second");
    return secondDescriptor;
  }
});
Object.defineProperty(descriptorMapTarget, "hidden", {
  enumerable: false,
  get: function() {
    throw "non-enumerable descriptor must not be read";
  }
});
var descriptorMap = new Proxy(descriptorMapTarget, {
  ownKeys: function(actualTarget) {
    order.push("ownKeys");
    return Reflect.ownKeys(actualTarget);
  },
  getOwnPropertyDescriptor: function(actualTarget, key) {
    order.push("descriptor:" + key);
    return Reflect.getOwnPropertyDescriptor(actualTarget, key);
  },
  get: function(actualTarget, key, receiver) {
    order.push("get-trap:" + key);
    return Reflect.get(actualTarget, key, receiver);
  }
});
Object.defineProperties(target, descriptorMap);
var targetValidationPrecedesProperties = false;
var propertiesObserved = false;
try {
  Object.defineProperties(1, new Proxy({}, {
    ownKeys: function() {
      propertiesObserved = true;
      return [];
    }
  }));
} catch (error) {
  targetValidationPrecedesProperties =
    error instanceof TypeError && !propertiesObserved;
}
var nullPropertiesError = false;
try {
  Object.create(null, null);
} catch (error) {
  nullPropertiesError = error instanceof TypeError;
}
var primitivePropertiesTarget = {};
var primitiveProperties =
  Object.defineProperties(primitivePropertiesTarget, true) ===
  primitivePropertiesTarget;
var firstSymbol = Symbol("same");
var secondSymbol = Symbol("same");
var symbolTarget = {};
var symbolDescriptors = {};
symbolDescriptors[firstSymbol] = { value: 3, enumerable: true };
symbolDescriptors[secondSymbol] = { value: 4, enumerable: true };
Object.defineProperties(symbolTarget, symbolDescriptors);
var symbolKeys = Reflect.ownKeys(symbolTarget);
var other = __scriptwCreateRealm().global;
var fixedDescriptorMap = {};
Object.defineProperty(fixedDescriptorMap, "fixed", {
  value: { value: 1 },
  configurable: false
});
var otherRealmProxyInvariantError = false;
try {
  other.Object.defineProperties({}, new Proxy(fixedDescriptorMap, {
    getOwnPropertyDescriptor: function() {
      return undefined;
    }
  }));
} catch (error) {
  otherRealmProxyInvariantError =
    Object.getPrototypeOf(error) === other.TypeError.prototype &&
    !(error instanceof TypeError);
}
[
  target.first,
  target.second,
  Object.hasOwn(target, "hidden"),
  Object.keys(target).join(","),
  order.join(","),
  targetValidationPrecedesProperties,
  nullPropertiesError,
  primitiveProperties,
  symbolTarget[firstSymbol],
  symbolTarget[secondSymbol],
  symbolKeys[0] === firstSymbol,
  symbolKeys[1] === secondSymbol,
  otherRealmProxyInvariantError
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object descriptor maps should preserve observable two-phase semantics");
    assert!(
            outcome.note.contains(
                "string(1|2|false|first,second|ownKeys,descriptor:first,get-trap:first,get:first,first:enumerable,first:value,descriptor:second,get-trap:second,get:second,second:enumerable,second:value:false,descriptor:hidden|true|true|true|3|4|true|true|true)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_object_is_uses_same_value_for_allocated_strings_and_bigints() {
    let outcome = engine()
        .run_script(
            r#"
[
  Object.is(["ab", "cd"].join(""), "abcd"),
  Object.is(
    BigInt("123456789012345678901"),
    BigInt("123456789012345678901")
  ),
  Object.is(NaN, NaN),
  Object.is(0, -0)
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.is should implement SameValue");
    assert!(
        outcome.note.contains("string(true|true|true|false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_object_prototype_statics_coerce_and_preserve_prototype_invariants() {
    let outcome = engine()
        .run_script(
            r#"
var proxySupport = new Proxy({}, {});
var other = __scriptwCreateRealm().global;
var otherGetPrototypeOf = other.Object.getPrototypeOf;
var primitivePrototype =
  Object.getPrototypeOf(1) === Number.prototype &&
  Object.getPrototypeOf("x") === String.prototype &&
  Object.getPrototypeOf(true) === Boolean.prototype &&
  Object.getPrototypeOf(Symbol("x")) === Symbol.prototype &&
  Object.getPrototypeOf(1n) === BigInt.prototype;
var otherPrimitivePrototype =
  otherGetPrototypeOf(1) === other.Number.prototype &&
  otherGetPrototypeOf("x") === other.String.prototype;
var missingError = false;
try {
  Object.getPrototypeOf();
} catch (error) {
  missingError = error instanceof TypeError;
}
var otherRealmError = false;
try {
  otherGetPrototypeOf(null);
} catch (error) {
  otherRealmError =
    Object.getPrototypeOf(error) === other.TypeError.prototype &&
    !(error instanceof TypeError);
}
var immutableError = false;
try {
  Object.setPrototypeOf(Object.prototype, {});
} catch (error) {
  immutableError = error instanceof TypeError;
}
var otherImmutableError = false;
try {
  other.Object.setPrototypeOf(other.Object.prototype, {});
} catch (error) {
  otherImmutableError =
    Object.getPrototypeOf(error) === other.TypeError.prototype;
}
var trapMarker = {};
var trapAbruptIdentity = false;
try {
  Object.setPrototypeOf(new Proxy({}, {
    setPrototypeOf: function() { throw trapMarker; }
  }), null);
} catch (error) {
  trapAbruptIdentity = error === trapMarker;
}
var fixed = {};
Object.preventExtensions(fixed);
var fixedSame = Object.setPrototypeOf(
  fixed,
  Object.getPrototypeOf(fixed)
) === fixed;
var primitive = 1;
var primitiveResult = Object.setPrototypeOf(primitive, null) === primitive;
var invalidPrototypeError = false;
try {
  Object.setPrototypeOf(primitive, 1);
} catch (error) {
  invalidPrototypeError = error instanceof TypeError;
}
[
  primitivePrototype,
  otherPrimitivePrototype,
  missingError,
  otherRealmError,
  immutableError,
  Reflect.setPrototypeOf(Object.prototype, {}) === false,
  Object.setPrototypeOf(Object.prototype, null) === Object.prototype,
  otherImmutableError,
  Reflect.setPrototypeOf(other.Object.prototype, {}) === false,
  trapAbruptIdentity,
  fixedSame,
  primitiveResult,
  invalidPrototypeError,
  Object.getPrototypeOf(this).isPrototypeOf(this)
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object prototype statics should preserve coercion and immutable prototypes");
    assert!(
        outcome.note.contains(
            "string(true|true|true|true|true|true|true|true|true|true|true|true|true|true)"
        ),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_object_seal_sets_integrity_level_and_observes_proxy_operations() {
    let outcome = engine()
        .run_script(
            r#"
var target = { value: 1 };
var symbol = Symbol("sealed");
target[symbol] = 3;
Object.defineProperty(target, "accessor", {
  get: function() { return 2; },
  configurable: true
});
var sealed = Object.seal(target);
var valueDescriptor = Object.getOwnPropertyDescriptor(target, "value");
var accessorDescriptor = Object.getOwnPropertyDescriptor(target, "accessor");
var symbolDescriptor = Object.getOwnPropertyDescriptor(target, symbol);
target.value = 4;
target.extra = 5;
var deleted = delete target.value;

var array = [1];
Object.seal(array);
var elementDescriptor = Object.getOwnPropertyDescriptor(array, "0");
var lengthDescriptor = Object.getOwnPropertyDescriptor(array, "length");
array[0] = 2;
var pushError = "none";
try { array.push(3); } catch (error) { pushError = error.name; }

var calls = [];
var proxyTarget = { x: 1 };
var proxy = new Proxy(proxyTarget, {
  preventExtensions: function(target) {
    calls.push("preventExtensions");
    return Reflect.preventExtensions(target);
  },
  ownKeys: function(target) {
    calls.push("ownKeys");
    return Reflect.ownKeys(target);
  },
  defineProperty: function(target, key, descriptor) {
    calls.push("defineProperty " + key);
    return Reflect.defineProperty(target, key, descriptor);
  }
});
var sealedProxy = Object.seal(proxy);

var preventRejected = false;
try {
  Object.seal(new Proxy({}, {
    preventExtensions: function() { return false; }
  }));
} catch (error) {
  preventRejected = error instanceof TypeError;
}
var defineTarget = { x: 1 };
var defineRejected = false;
try {
  Object.seal(new Proxy(defineTarget, {
    preventExtensions: function(target) {
      return Reflect.preventExtensions(target);
    },
    defineProperty: function() { return false; }
  }));
} catch (error) {
  defineRejected = error instanceof TypeError;
}
var constructRejected = false;
try { new Object.seal({}); } catch (error) {
  constructRejected = error instanceof TypeError;
}

[
  typeof Object.seal,
  Object.seal.length,
  Object.seal.name,
  sealed === target,
  Object.isSealed(target),
  Object.isFrozen(target),
  valueDescriptor.configurable,
  valueDescriptor.writable,
  accessorDescriptor.configurable,
  symbolDescriptor.configurable,
  target.value,
  "extra" in target,
  deleted,
  Object.isSealed(array),
  elementDescriptor.configurable,
  elementDescriptor.writable,
  lengthDescriptor.writable,
  array[0],
  array.length,
  pushError,
  sealedProxy === proxy,
  Object.isSealed(proxyTarget),
  calls.join(","),
  preventRejected,
  defineRejected,
  constructRejected,
  Object.seal(1) === 1,
  Object.seal() === undefined
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.seal should implement SetIntegrityLevel semantics");
    assert!(
            outcome.note.contains(
                "string(function|1|seal|true|true|false|false|true|false|false|4|false|false|true|false|true|true|2|1|TypeError|true|true|preventExtensions,ownKeys,defineProperty x|true|true|true|true|true)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_object_seal_preserves_arguments_properties() {
    let outcome = engine()
        .run_script(
            r#"
function sealArguments() {
  return arguments;
}
function freezeMapped(value) {
  var values = arguments;
  Object.freeze(values);
  value = 2;
  return values[0] === 1 && Object.isFrozen(values);
}
var values = sealArguments();
values.foo = 10;
Object.seal(values);
var descriptor = Object.getOwnPropertyDescriptor(values, "foo");
var specialOnly = sealArguments();
Object.preventExtensions(specialOnly);
var configurableNamed = sealArguments();
Object.defineProperty(configurableNamed, "length", { configurable: false });
Object.defineProperty(configurableNamed, "callee", { configurable: false });
configurableNamed.foo = 1;
Object.preventExtensions(configurableNamed);
var frozen = sealArguments();
frozen.foo = 1;
Object.freeze(frozen);
var frozenNamed = [];
frozenNamed.foo = 1;
Object.freeze(frozenNamed);
var namedOnly = [];
namedOnly.foo = 1;
Object.defineProperty(namedOnly, "length", { writable: false });
Object.preventExtensions(namedOnly);
[
  !Object.isExtensible(values),
  Object.isSealed(values),
  !Object.isFrozen(values),
  descriptor.configurable === false,
  descriptor.value === 10,
  values.foo === 10,
  !Object.isSealed(specialOnly),
  !Object.isFrozen(specialOnly),
  !Object.isSealed(configurableNamed),
  !Object.isFrozen(configurableNamed),
  Object.isSealed(frozen),
  Object.isFrozen(frozen),
  Object.isSealed(frozenNamed),
  Object.isFrozen(frozenNamed),
  freezeMapped(1),
  !Object.isSealed(namedOnly),
  !Object.isFrozen(namedOnly)
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.seal should preserve existing Arguments properties");
    assert!(
            outcome.note.contains(
                "string(true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_object_freeze_preserves_sparse_array_elements() {
    let outcome = engine()
        .run_script(
            r#"
var values = [0, 1];
values[10000] = 10000;
Object.freeze(values);
var descriptor = Object.getOwnPropertyDescriptor(values, "10000");
[
  Object.isFrozen(values),
  values[10000],
  values.length,
  descriptor.writable,
  descriptor.configurable,
  Object.hasOwn(values, "9999")
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.freeze should preserve sparse array elements");
    assert!(
        outcome
            .note
            .contains("string(true|10000|10001|false|false|false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_object_entries_and_values_expose_standard_metadata_and_coerce_primitives() {
    let outcome = engine()
        .run_script(
            r#"
var entriesDescriptor = Object.getOwnPropertyDescriptor(Object, "entries");
var valuesDescriptor = Object.getOwnPropertyDescriptor(Object, "values");
var entriesNullError = "missing";
var valuesUndefinedError = "missing";
var entriesConstructionError = "missing";
var valuesConstructionError = "missing";
try {
  Object.entries(null);
} catch (error) {
  entriesNullError = error.name;
}
try {
  Object.values(undefined);
} catch (error) {
  valuesUndefinedError = error.name;
}
try {
  new Object.entries({});
} catch (error) {
  entriesConstructionError = error.name;
}
try {
  new Object.values({});
} catch (error) {
  valuesConstructionError = error.name;
}
var stringEntries = Object.entries("ab");
var emptyPrimitiveCount =
  Object.values(true).length +
  Object.values(1).length +
  Object.values(Symbol("s")).length +
  Object.values(1n).length;
var originalKeys = Object.keys;
Object.keys = function() {
  throw "Object.keys must not be called";
};
var tamperedValues = Object.values({ x: 7 });
var tamperedEntries = Object.entries({ x: 7 });
Object.keys = originalKeys;
[
  Object.entries.length,
  Object.entries.name,
  entriesDescriptor.writable,
  entriesDescriptor.enumerable,
  entriesDescriptor.configurable,
  Object.values.length,
  Object.values.name,
  valuesDescriptor.writable,
  valuesDescriptor.enumerable,
  valuesDescriptor.configurable,
  entriesNullError,
  valuesUndefinedError,
  entriesConstructionError,
  valuesConstructionError,
  Object.values("ab").join(","),
  stringEntries[0].join(":") + "," + stringEntries[1].join(":"),
  emptyPrimitiveCount,
  tamperedValues[0],
  tamperedEntries[0][0],
  tamperedEntries[0][1]
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.entries and Object.values should coerce primitives");
    assert!(
            outcome.note.contains(
                "string(1|entries|true|false|true|1|values|true|false|true|TypeError|TypeError|TypeError|TypeError|a,b|0:a,1:b|0|7|x|7)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_object_entries_and_values_snapshot_keys_and_recheck_descriptors() {
    let outcome = engine()
        .run_script(
            r#"
var symbol = Symbol("symbol");
var ordered = Object.create({ inherited: "inherited" });
ordered.b = "old-b";
ordered[2] = "two";
ordered[1] = "one";
ordered.a = "a";
Object.defineProperty(ordered, "hidden", {
  value: "hidden",
  enumerable: false
});
ordered[symbol] = "symbol";
Object.defineProperty(ordered, "a", {
  value: "updated-a"
});
delete ordered.b;
ordered.b = "b";
var orderedEntries = Object.entries(ordered);

function makeMutationObject() {
  var target = {};
  Object.defineProperty(target, "first", {
    enumerable: true,
    get: function() {
      delete target.deleted;
      Object.defineProperty(target, "later", {
        enumerable: false
      });
      target.added = "added";
      return "first";
    }
  });
  Object.defineProperty(target, "deleted", {
    value: "deleted",
    enumerable: true,
    configurable: true
  });
  Object.defineProperty(target, "later", {
    value: "later",
    enumerable: true,
    configurable: true
  });
  Object.defineProperty(target, "last", {
    enumerable: true,
    get: function() {
      return "last";
    }
  });
  return target;
}

var mutationValues = Object.values(makeMutationObject());
var mutationEntries = Object.entries(makeMutationObject());
[
  Object.values(ordered).join(","),
  orderedEntries[0].join(":") + "," +
    orderedEntries[1].join(":") + "," +
    orderedEntries[2].join(":") + "," +
    orderedEntries[3].join(":"),
  orderedEntries.length,
  orderedEntries[0] !== orderedEntries[1],
  Array.isArray(orderedEntries[0]),
  mutationValues.join(","),
  mutationEntries[0].join(":") + "," + mutationEntries[1].join(":"),
  mutationEntries.length
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.entries and Object.values should use live enumerable descriptors");
    assert!(
            outcome.note.contains(
                "string(one,two,updated-a,b|1:one,2:two,a:updated-a,b:b|4|true|true|first,last|first:first,last:last|2)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_object_entries_and_values_observe_proxy_operations_and_abrupt_completion() {
    let outcome = engine()
        .run_script(
            r#"
var calls = [];
var symbol = Symbol("symbol");
var receiverIsProxy = false;
var target = {};
Object.defineProperty(target, "a", {
  value: 1,
  enumerable: true
});
Object.defineProperty(target, "hidden", {
  value: 9,
  enumerable: false
});
Object.defineProperty(target, "b", {
  enumerable: true,
  get: function() {
    receiverIsProxy = this === proxy;
    return 2;
  }
});
var proxy = new Proxy(target, {
  ownKeys: function() {
    calls.push("ownKeys");
    return ["a", symbol, "hidden", "b"];
  },
  getOwnPropertyDescriptor: function(target, key) {
    calls.push("desc:" + key);
    return Reflect.getOwnPropertyDescriptor(target, key);
  },
  get: function(target, key, receiver) {
    calls.push("get:" + key);
    return Reflect.get(target, key, receiver);
  }
});
var values = Object.values(proxy);
var valuesCalls = calls.join(",");
var valuesReceiverIsProxy = receiverIsProxy;
calls.length = 0;
receiverIsProxy = false;
var entries = Object.entries(proxy);
var entriesCalls = calls.join(",");
var entriesReceiverIsProxy = receiverIsProxy;

var ownKeysMarker = {};
var ownKeysIdentity = false;
try {
  Object.values(new Proxy({}, {
    ownKeys: function() {
      throw ownKeysMarker;
    }
  }));
} catch (error) {
  ownKeysIdentity = error === ownKeysMarker;
}

calls.length = 0;
var descriptorMarker = {};
var descriptorIdentity = false;
try {
  Object.entries(new Proxy({ a: 1, b: 2 }, {
    ownKeys: function() {
      calls.push("ownKeys");
      return ["a", "b"];
    },
    getOwnPropertyDescriptor: function(target, key) {
      calls.push("desc:" + key);
      throw descriptorMarker;
    },
    get: function() {
      calls.push("unexpected get");
    }
  }));
} catch (error) {
  descriptorIdentity = error === descriptorMarker;
}
var descriptorCalls = calls.join(",");

calls.length = 0;
var getterMarker = {};
var getterIdentity = false;
var laterTouched = false;
var abruptTarget = {};
Object.defineProperty(abruptTarget, "a", {
  enumerable: true,
  get: function() {
    throw getterMarker;
  }
});
Object.defineProperty(abruptTarget, "b", {
  enumerable: true,
  get: function() {
    laterTouched = true;
    return 2;
  }
});
try {
  Object.values(new Proxy(abruptTarget, {
    ownKeys: function() {
      calls.push("ownKeys");
      return ["a", "b"];
    },
    getOwnPropertyDescriptor: function(target, key) {
      calls.push("desc:" + key);
      return Reflect.getOwnPropertyDescriptor(target, key);
    },
    get: function(target, key, receiver) {
      calls.push("get:" + key);
      return Reflect.get(target, key, receiver);
    }
  }));
} catch (error) {
  getterIdentity = error === getterMarker;
}
[
  values.join(","),
  valuesCalls,
  valuesReceiverIsProxy,
  entries[0].join(":") + "," + entries[1].join(":"),
  entriesCalls,
  entriesReceiverIsProxy,
  ownKeysIdentity,
  descriptorIdentity,
  descriptorCalls,
  getterIdentity,
  calls.join(","),
  laterTouched
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.entries and Object.values should observe Proxy internal methods");
    assert!(
            outcome.note.contains(
                "string(1,2|ownKeys,desc:a,get:a,desc:hidden,desc:b,get:b|true|a:1,b:2|ownKeys,desc:a,get:a,desc:hidden,desc:b,get:b|true|true|true|ownKeys,desc:a|true|ownKeys,desc:a,get:a|false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_object_assign_copies_live_enumerable_properties_with_strict_set() {
    let outcome = engine()
        .run_script(
            r#"
var calls = [];
var symbol = Symbol("symbol");
var sourceTarget = { a: 1 };
Object.defineProperty(sourceTarget, "hidden", {
  value: 2,
  enumerable: false
});
sourceTarget[symbol] = 3;
var source = new Proxy(sourceTarget, {
  ownKeys: function() {
    calls.push("ownKeys");
    return ["a", "hidden", symbol];
  },
  getOwnPropertyDescriptor: function(target, key) {
    calls.push("desc:" + (key === symbol ? "symbol" : key));
    return Reflect.getOwnPropertyDescriptor(target, key);
  },
  get: function(target, key, receiver) {
    calls.push("get:" + (key === symbol ? "symbol" : key));
    return Reflect.get(target, key, receiver);
  }
});
var targetObject = {};
var target = new Proxy(targetObject, {
  set: function(object, key, value) {
    calls.push("set:" + (key === symbol ? "symbol" : key));
    object[key] = value;
    return true;
  }
});
var result = Object.assign(target, null, undefined, "xy", source, { z: 4 });
var nonExtensibleError = "missing";
try {
  Object.assign(Object.preventExtensions({}), { q: 1 });
} catch (error) {
  nonExtensibleError = error.name;
}
var readOnlyStringError = "missing";
try {
  Object.assign("a", [1]);
} catch (error) {
  readOnlyStringError = error.name;
}
var nullTargetError = "missing";
try {
  Object.assign(null, {});
} catch (error) {
  nullTargetError = error.name;
}
var constructionError = "missing";
try {
  new Object.assign({}, {});
} catch (error) {
  constructionError = error.name;
}
var boxedTarget = Object.assign("a", { x: 1 });
var descriptor = Object.getOwnPropertyDescriptor(Object, "assign");
[
  result === target,
  targetObject[0],
  targetObject[1],
  targetObject.a,
  targetObject[symbol],
  targetObject.z,
  calls.join(","),
  nonExtensibleError,
  readOnlyStringError,
  nullTargetError,
  constructionError,
  typeof boxedTarget,
  boxedTarget.x,
  Object.assign.length,
  Object.assign.name,
  descriptor.writable,
  descriptor.enumerable,
  descriptor.configurable
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.assign should copy live enumerable own properties");
    assert!(
            outcome.note.contains(
                "string(true|x|y|1|3|4|set:0,set:1,ownKeys,desc:a,get:a,set:a,desc:hidden,desc:symbol,get:symbol,set:symbol,set:z|TypeError|TypeError|TypeError|TypeError|object|1|2|assign|true|false|true)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_object_get_own_property_descriptors_preserves_descriptors_and_proxy_order() {
    let outcome = engine()
        .run_script(
            r#"
var getter = function() {
  throw "descriptor getter must not run";
};
var setter = function(value) {};
var symbol = Symbol("symbol");
var source = Object.create({ inherited: 9 });
Object.defineProperty(source, "a", {
  value: 1,
  writable: false,
  enumerable: true,
  configurable: false
});
Object.defineProperty(source, "b", {
  get: getter,
  set: setter,
  enumerable: false,
  configurable: true
});
source[symbol] = 3;
var calls = [];
var proxy = new Proxy(source, {
  ownKeys: function() {
    calls.push("ownKeys");
    return ["a", "missing", "b", symbol];
  },
  getOwnPropertyDescriptor: function(target, key) {
    calls.push("desc:" + (key === symbol ? "symbol" : key));
    return Reflect.getOwnPropertyDescriptor(target, key);
  },
  get: function() {
    calls.push("unexpected get");
  }
});
var descriptors = Object.getOwnPropertyDescriptors(proxy);
var resultProperty = Object.getOwnPropertyDescriptor(descriptors, "a");
var stringDescriptors = Object.getOwnPropertyDescriptors("ab");
var originalGetOwnPropertyDescriptor = Object.getOwnPropertyDescriptor;
Object.getOwnPropertyDescriptor = function() {
  throw "the plural builtin must use the internal operation";
};
var untampered = Object.getOwnPropertyDescriptors({ x: 7 });
Object.getOwnPropertyDescriptor = originalGetOwnPropertyDescriptor;
var nullError = "missing";
try {
  Object.getOwnPropertyDescriptors(null);
} catch (error) {
  nullError = error.name;
}
var constructionError = "missing";
try {
  new Object.getOwnPropertyDescriptors({});
} catch (error) {
  constructionError = error.name;
}
var builtinDescriptor =
  Object.getOwnPropertyDescriptor(Object, "getOwnPropertyDescriptors");
[
  descriptors.a.value,
  descriptors.a.writable,
  descriptors.a.enumerable,
  descriptors.a.configurable,
  descriptors.b.get === getter,
  descriptors.b.set === setter,
  descriptors.b.enumerable,
  descriptors.b.configurable,
  descriptors[symbol].value,
  Object.hasOwn(descriptors, "missing"),
  Object.hasOwn(descriptors, "inherited"),
  calls.join(","),
  resultProperty.writable,
  resultProperty.enumerable,
  resultProperty.configurable,
  Object.getPrototypeOf(descriptors) === Object.prototype,
  Object.getPrototypeOf(descriptors.a) === Object.prototype,
  stringDescriptors[0].value,
  stringDescriptors[1].value,
  stringDescriptors.length.value,
  stringDescriptors.length.enumerable,
  untampered.x.value,
  nullError,
  constructionError,
  Object.getOwnPropertyDescriptors.length,
  Object.getOwnPropertyDescriptors.name,
  builtinDescriptor.writable,
  builtinDescriptor.enumerable,
  builtinDescriptor.configurable
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.getOwnPropertyDescriptors should preserve property descriptors");
    assert!(
            outcome.note.contains(
                "string(1|false|true|false|true|true|false|true|3|false|false|ownKeys,desc:a,desc:missing,desc:b,desc:symbol|true|true|true|true|true|a|b|2|false|7|TypeError|TypeError|1|getOwnPropertyDescriptors|true|false|true)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_object_integrity_queries_observe_proxy_operations() {
    let outcome = engine()
        .run_script(
            r#"
var sealedCalls = [];
var sealedSymbol = Symbol("sealed");
var sealedTarget = {};
Object.defineProperty(sealedTarget, "data", {
  value: 1,
  writable: true,
  configurable: false
});
Object.defineProperty(sealedTarget, "disappearing", {
  value: 2,
  configurable: true
});
Object.defineProperty(sealedTarget, "accessor", {
  get: function() { return 3; },
  configurable: false
});
Object.defineProperty(sealedTarget, sealedSymbol, {
  value: 4,
  writable: true,
  configurable: false
});
Object.preventExtensions(sealedTarget);
var sealedProxy = new Proxy(sealedTarget, {
  isExtensible: function(target) {
    sealedCalls.push("isExtensible");
    return Reflect.isExtensible(target);
  },
  ownKeys: function() {
    sealedCalls.push("ownKeys");
    return ["data", "disappearing", "accessor", sealedSymbol];
  },
  getOwnPropertyDescriptor: function(target, key) {
    var name = key === sealedSymbol ? "symbol" : key;
    sealedCalls.push("get " + name);
    if (key === "disappearing") {
      delete target.disappearing;
      return undefined;
    }
    return Reflect.getOwnPropertyDescriptor(target, key);
  }
});

var frozenCalls = [];
var frozenSymbol = Symbol("frozen");
var frozenTarget = {};
Object.defineProperty(frozenTarget, "data", {
  value: 1,
  writable: false,
  configurable: false
});
Object.defineProperty(frozenTarget, "disappearing", {
  value: 2,
  configurable: true
});
Object.defineProperty(frozenTarget, "accessor", {
  get: function() { return 3; },
  configurable: false
});
Object.defineProperty(frozenTarget, frozenSymbol, {
  value: 4,
  writable: false,
  configurable: false
});
Object.preventExtensions(frozenTarget);
var frozenProxy = new Proxy(frozenTarget, {
  isExtensible: function(target) {
    frozenCalls.push("isExtensible");
    return Reflect.isExtensible(target);
  },
  ownKeys: function() {
    frozenCalls.push("ownKeys");
    return ["data", "disappearing", "accessor", frozenSymbol];
  },
  getOwnPropertyDescriptor: function(target, key) {
    var name = key === frozenSymbol ? "symbol" : key;
    frozenCalls.push("get " + name);
    if (key === "disappearing") {
      delete target.disappearing;
      return undefined;
    }
    return Reflect.getOwnPropertyDescriptor(target, key);
  }
});

var extensibleCalls = [];
var extensibleProxy = new Proxy({}, {
  isExtensible: function(target) {
    extensibleCalls.push("isExtensible");
    return Reflect.isExtensible(target);
  },
  ownKeys: function() {
    throw "ownKeys must not run";
  }
});

var sealedRejectCalls = [];
var sealedRejectTarget = {};
Object.defineProperty(sealedRejectTarget, "first", {
  value: 1,
  configurable: true
});
Object.defineProperty(sealedRejectTarget, "later", {
  value: 2,
  configurable: true
});
Object.preventExtensions(sealedRejectTarget);
var sealedRejectProxy = new Proxy(sealedRejectTarget, {
  getOwnPropertyDescriptor: function(target, key) {
    sealedRejectCalls.push(key);
    if (key === "later") {
      throw "later sealed descriptor must not run";
    }
    return Reflect.getOwnPropertyDescriptor(target, key);
  }
});

var frozenRejectCalls = [];
var frozenRejectTarget = {};
Object.defineProperty(frozenRejectTarget, "first", {
  value: 1,
  writable: true,
  configurable: false
});
Object.defineProperty(frozenRejectTarget, "later", {
  value: 2,
  writable: false,
  configurable: false
});
Object.preventExtensions(frozenRejectTarget);
var frozenRejectProxy = new Proxy(frozenRejectTarget, {
  getOwnPropertyDescriptor: function(target, key) {
    frozenRejectCalls.push(key);
    if (key === "later") {
      throw "later frozen descriptor must not run";
    }
    return Reflect.getOwnPropertyDescriptor(target, key);
  }
});

var extensibleMarker = {};
var extensibleAbruptIdentity = false;
try {
  Object.isSealed(new Proxy({}, {
    isExtensible: function() { throw extensibleMarker; }
  }));
} catch (error) {
  extensibleAbruptIdentity = error === extensibleMarker;
}

var ownKeysMarker = {};
var ownKeysAbruptIdentity = false;
var ownKeysTarget = {};
Object.preventExtensions(ownKeysTarget);
try {
  Object.isFrozen(new Proxy(ownKeysTarget, {
    ownKeys: function() { throw ownKeysMarker; }
  }));
} catch (error) {
  ownKeysAbruptIdentity = error === ownKeysMarker;
}

var descriptorMarker = {};
var descriptorAbruptIdentity = false;
var descriptorTarget = {};
Object.defineProperty(descriptorTarget, "value", {
  value: 1,
  configurable: false
});
Object.preventExtensions(descriptorTarget);
try {
  Object.isSealed(new Proxy(descriptorTarget, {
    getOwnPropertyDescriptor: function() { throw descriptorMarker; }
  }));
} catch (error) {
  descriptorAbruptIdentity = error === descriptorMarker;
}

[
  Object.isSealed(sealedProxy),
  sealedCalls.join(","),
  Object.isFrozen(frozenProxy),
  frozenCalls.join(","),
  !Object.isSealed(extensibleProxy),
  !Object.isFrozen(extensibleProxy),
  extensibleCalls.join(","),
  !Object.isSealed(sealedRejectProxy),
  sealedRejectCalls.join(","),
  !Object.isFrozen(frozenRejectProxy),
  frozenRejectCalls.join(","),
  extensibleAbruptIdentity,
  ownKeysAbruptIdentity,
  descriptorAbruptIdentity,
  Object.isSealed(1),
  Object.isFrozen(null),
  Object.isSealed(),
  Object.isFrozen("text")
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object integrity queries should use Proxy internal methods");
    assert!(
            outcome.note.contains(
                "string(true|isExtensible,ownKeys,get data,get disappearing,get accessor,get symbol|true|isExtensible,ownKeys,get data,get disappearing,get accessor,get symbol|true|true|isExtensible,isExtensible|true|first|true|first|true|true|true|true|true|true|true)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_object_freeze_observes_proxy_integrity_operations() {
    let outcome = engine()
        .run_script(
            r#"
var calls = [];
var symbol = Symbol("frozen");
var setterValue = 0;
var target = { data: 1, disappearing: 2 };
Object.defineProperty(target, "accessor", {
  get: function() { return setterValue; },
  set: function(value) { setterValue = value; },
  enumerable: true,
  configurable: true
});
target[symbol] = 3;

var proxy = new Proxy(target, {
  preventExtensions: function(innerTarget) {
    calls.push("preventExtensions");
    return Reflect.preventExtensions(innerTarget);
  },
  ownKeys: function() {
    calls.push("ownKeys");
    return ["data", "disappearing", "accessor", symbol];
  },
  getOwnPropertyDescriptor: function(innerTarget, key) {
    calls.push("get " + (key === symbol ? "symbol" : key));
    if (key === "disappearing") {
      delete innerTarget.disappearing;
      return undefined;
    }
    return Reflect.getOwnPropertyDescriptor(innerTarget, key);
  },
  defineProperty: function(innerTarget, key, descriptor) {
    var name = key === symbol ? "symbol" : key;
    calls.push(
      "define " + name + ":" +
      Object.hasOwn(descriptor, "writable") + ":" +
      descriptor.writable + ":" +
      Object.hasOwn(descriptor, "value") + ":" +
      Object.hasOwn(descriptor, "get") + ":" +
      Object.hasOwn(descriptor, "set") + ":" +
      Object.hasOwn(descriptor, "enumerable") + ":" +
      descriptor.configurable
    );
    return Reflect.defineProperty(innerTarget, key, descriptor);
  }
});

var returned = Object.freeze(proxy);
target.accessor = 7;
var dataDescriptor = Object.getOwnPropertyDescriptor(target, "data");
var accessorDescriptor = Object.getOwnPropertyDescriptor(target, "accessor");
var symbolDescriptor = Object.getOwnPropertyDescriptor(target, symbol);

var preventFalseThrows = false;
try {
  Object.freeze(new Proxy({}, {
    preventExtensions: function() { return false; }
  }));
} catch (error) {
  preventFalseThrows = error instanceof TypeError;
}

var abruptMarker = {};
var abruptPropagates = false;
try {
  Object.freeze(new Proxy({}, {
    preventExtensions: function() { throw abruptMarker; }
  }));
} catch (error) {
  abruptPropagates = error === abruptMarker;
}

var defineFalseTarget = { value: 1 };
var defineFalseThrows = false;
try {
  Object.freeze(new Proxy(defineFalseTarget, {
    preventExtensions: function(innerTarget) {
      return Reflect.preventExtensions(innerTarget);
    },
    defineProperty: function() { return false; }
  }));
} catch (error) {
  defineFalseThrows = error instanceof TypeError;
}

[
  returned === proxy,
  !Object.isExtensible(target),
  dataDescriptor.writable === false && dataDescriptor.configurable === false,
  accessorDescriptor.configurable === false && setterValue === 7,
  symbolDescriptor.writable === false && symbolDescriptor.configurable === false,
  !Object.hasOwn(target, "disappearing"),
  preventFalseThrows,
  abruptPropagates,
  defineFalseThrows,
  calls.join(",")
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.freeze should use Proxy integrity-level internal methods");
    assert!(
            outcome.note.contains(
                "string(true|true|true|true|true|true|true|true|true|preventExtensions,ownKeys,get data,define data:true:false:false:false:false:false:false,get disappearing,get accessor,define accessor:false:undefined:false:false:false:false:false,get symbol,define symbol:true:false:false:false:false:false:false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_get_own_property_names_to_object_string() {
    let outcome = engine()
            .run_script(
                "var empty = Object.getOwnPropertyNames(''); var text = Object.getOwnPropertyNames('ab'); var boxed = new String('abc'); boxed[5] = 'de'; var boxedNames = Object.getOwnPropertyNames(boxed); var arrNames = Object.getOwnPropertyNames([0, 1, 2]); var arrWithAccessor = [0, 1, 2]; Object.defineProperty(arrWithAccessor, 'ownProperty', { get: function() { return 'ownArray'; }, configurable: true }); var arrAccessorNames = Object.getOwnPropertyNames(arrWithAccessor); var accessorFound = false; for (var p in arrAccessorNames) { if (arrAccessorNames[p] === 'ownProperty') { accessorFound = true; } } var arrWithDot = [0, 1, 2]; arrWithDot.ownProperty = 'ownArray'; var arrDotNames = Object.getOwnPropertyNames(arrWithDot); var arrOrder = []; arrOrder.a = 1; Object.defineProperty(arrOrder, 'length', { value: 2 }); var arrOrderNames = Object.getOwnPropertyNames(arrOrder); var threw = false; try { Object.getOwnPropertyNames(undefined); } catch (e) { threw = e.name === 'TypeError'; } empty.length === 1 && empty[0] === 'length' && text.length === 3 && text[0] === '0' && text[1] === '1' && text[2] === 'length' && boxedNames.length === 5 && boxedNames[0] === '0' && boxedNames[1] === '1' && boxedNames[2] === '2' && boxedNames[3] === '5' && boxedNames[4] === 'length' && arrNames.length === 4 && arrNames[0] === '0' && arrNames[1] === '1' && arrNames[2] === '2' && arrNames[3] === 'length' && arrAccessorNames.length === 5 && arrAccessorNames[0] === '0' && arrAccessorNames[1] === '1' && arrAccessorNames[2] === '2' && arrAccessorNames[3] === 'length' && arrAccessorNames[4] === 'ownProperty' && accessorFound && arrDotNames.length === 5 && arrDotNames[0] === '0' && arrDotNames[1] === '1' && arrDotNames[2] === '2' && arrDotNames[3] === 'length' && arrDotNames[4] === 'ownProperty' && arrWithDot.ownProperty === 'ownArray' && arrOrderNames.length === 2 && arrOrderNames[0] === 'length' && arrOrderNames[1] === 'a' && threw;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Object.getOwnPropertyNames ToObject string case should run");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_get_own_property_names_includes_boxed_string_custom_descriptors() {
    let outcome = engine()
        .run_script(
            r#"
var boxed = new String("abc");
Object.defineProperty(boxed, "dataProperty", {
  value: 1,
  configurable: true
});
Object.defineProperty(boxed, "accessorProperty", {
  get: function() { return 2; },
  configurable: true
});
Object.getOwnPropertyNames(boxed).join(",");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("boxed String own descriptor names should run");

    assert!(
        outcome
            .note
            .contains("string(0,1,2,length,dataProperty,accessorProperty)"),
        "unexpected boxed String own names: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_own_key_enumeration_separates_symbols() {
    let outcome = engine()
        .run_script(
            r#"
const arraySymbol = Symbol("array");
const array = [];
array.visible = 1;
array[arraySymbol] = 2;

Object.getOwnPropertyNames(array).join(",") === "length,visible"
  && Object.keys(array).join(",") === "visible"
  && Object.getOwnPropertySymbols(array).length === 1
  && Object.getOwnPropertySymbols(array)[0] === arraySymbol;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array own-key enumeration should preserve key kinds");

    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_arguments_own_key_enumeration_returns_symbols() {
    let outcome = engine()
        .run_script(
            r#"
(function(value) {
  const argumentsSymbol = Symbol("arguments");
  arguments.visible = 3;
  arguments[argumentsSymbol] = 4;
  const names = Object.getOwnPropertyNames(arguments);
  const symbols = Object.getOwnPropertySymbols(arguments);
  return names.length === 4
    && names.indexOf("visible") !== -1
    && symbols.length === 2
    && symbols[0] === Symbol.iterator
    && symbols[1] === argumentsSymbol
    && arguments[Symbol.iterator] === Array.prototype.values
    && Reflect.ownKeys(arguments).length === 6;
})(0);
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Arguments own-key enumeration should return symbol keys");

    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_typed_array_constructor_gets_new_target_prototype_before_source_iterator() {
    let source = r#"
const order = [];
const newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, "prototype", {
  get() {
    order.push("prototype");
    return Uint8Array.prototype;
  }
});
const values = {
  get [Symbol.iterator]() {
    order.push("iterator");
    return function() {
      return [7][Symbol.iterator]();
    };
  }
};
const result = Reflect.construct(Uint8Array, [values], newTarget);
order.join(",") + "|" + result.join(",");
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
            .expect(
                "TypedArray constructor should get newTarget.prototype before inspecting the source iterator",
            );
    assert!(
        outcome.note.contains("string(prototype,iterator|7)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_owned_buffer_inherits_the_complete_array_buffer_prototype() {
    // No Uint8Array or ArrayBuffer reference: the Float64Array alone must
    // publish %ArrayBuffer.prototype%, whose members the detach call leaves
    // reachable only through ordinary [[Get]] on the owned buffer.
    let outcome = engine()
            .run_script(
                "const view = new Float64Array(2); const buffer = view.buffer; __scriptwDetachArrayBuffer(buffer); [buffer.detached, buffer.resizable, buffer.maxByteLength, buffer.byteLength, typeof buffer.slice, typeof buffer.resize, typeof buffer.transfer, view.length].join();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("TypedArray owned buffer should expose ArrayBuffer.prototype");
    assert!(
        outcome
            .note
            .contains("string(true,false,0,0,function,function,function,0)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_supports_destructured_object_parameter() {
    // Destructured object parameters (the case `wasm_emit_reports_
    // unsupported_slice_precisely` used to lock as unsupported) now
    // compile and run correctly on this branch.
    let outcome = engine()
        .run_script(
            "function f({ x }) { return x; } f({ x: 42 });",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("destructured object parameter should compile and run");
    assert!(outcome.note.contains("number(42"), "note: {}", outcome.note);
}

#[test]
fn wasm_backend_supports_string_symbol_hook_dispatch() {
    let outcome = engine()
        .run_script(
            r#"
function IsHTMLDDA() { return null; }
Object.defineProperty(IsHTMLDDA, "$IsHTMLDDA", { value: true });
var $262 = { IsHTMLDDA: IsHTMLDDA };
var total = 0;
function invoke(name, target) {
  if (name === "match") return "".match(target);
  if (name === "matchAll") return "".matchAll(target);
  if (name === "replace") return "".replace(target, target);
  if (name === "replaceAll") return "".replaceAll(target, target);
  if (name === "search") return "".search(target);
  if (name === "split") return "".split(target);
  throw name;
}
function check(name, symbol, expectedArgs) {
  var target = $262.IsHTMLDDA;
  var gets = 0;
  Object.defineProperty(target, symbol, {
    get: function() {
      gets += 1;
      return function() {
        if (this !== target) throw "this";
        if (arguments.length !== expectedArgs) throw "argc";
        if (arguments[0] !== "") throw "arg0";
        if ((name === "replace" || name === "replaceAll") && arguments[1] !== target) {
          throw "arg1";
        }
        return null;
      };
    },
    configurable: true
  });
  if (invoke(name, target) !== null) throw name;
  if (gets !== 1) throw "gets";
  total += gets;
  delete target[symbol];
}
check("match", Symbol.match, 1);
check("matchAll", Symbol.matchAll, 1);
check("replace", Symbol.replace, 2);
check("replaceAll", Symbol.replace, 2);
check("search", Symbol.search, 1);
check("split", Symbol.split, 2);
total;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should dispatch String.prototype symbol hooks");
    assert!(outcome.note.contains("number(6"));
}

#[test]
fn wasm_backend_supports_object_property_read() {
    let outcome = engine()
        .run_script(
            "let o = { x: 1, y: 2 }; o.x;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should run object property read");
    assert!(outcome.note.contains("number(1"));
}

#[test]
fn wasm_backend_supports_object_bracket_write() {
    let outcome = engine()
        .run_script(
            "let o = { x: 1 }; o[\"x\"] = 3; o.x;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should support object bracket write");
    assert!(outcome.note.contains("number(3"));
}

#[test]
fn wasm_backend_supports_object_return_from_function() {
    let outcome = engine()
        .run_script(
            "function box(x) { let o = { x: x }; return o; } let o = box(2); o.x;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should run object-return function");
    assert!(outcome.note.contains("number(2"));
}

#[test]
fn wasm_backend_supports_chained_object_access() {
    let outcome = engine()
        .run_script(
            "let o = { inner: { x: 1 } }; o.inner.x = 4; o.inner.x;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should run chained object access");
    assert!(outcome.note.contains("number(4"));
}

#[test]
fn wasm_backend_supports_call_result_property_and_array_length() {
    let property_outcome = engine()
        .run_script(
            "function box(x) { let o = { x: x }; return o; } box(2).x;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should run call-result property read");
    assert!(property_outcome.note.contains("number(2"));

    let array_outcome = engine()
        .run_script(
            "let a = [1, 2, 3]; a.length;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should run array length read");
    assert!(array_outcome.note.contains("number(3"));
}

#[test]
fn wasm_backend_supports_nested_array_object_reads() {
    let outcome = engine()
            .run_script(
                "function make() { let o = { items: [{ x: 1 }, { x: 3 }] }; return o; } make().items[1].x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run nested heap reads");
    assert!(outcome.note.contains("number(3"));
}

#[test]
fn wasm_backend_loose_equality_compares_distinct_objects_without_coercion() {
    let source = r#"
            let coercions = 0;
            function makeObject() {
                return {
                    valueOf: function () {
                        coercions += 1;
                        return 1;
                    }
                };
            }
            let left = makeObject();
            let right = makeObject();
            let equal = left == right;
            equal === false && coercions === 0;
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
        .expect("wasm backend should compare distinct objects without coercion");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_loose_equality_compares_objects_to_null_without_coercion() {
    let source = r#"
            let coercions = 0;
            let object = {
                valueOf: function () {
                    coercions += 1;
                    return null;
                }
            };
            let equal = object == null;
            equal === false && coercions === 0;
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
        .expect("wasm backend should compare objects to null without coercion");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_lowers_property_access_on_dynamic_target() {
    let outcome = engine()
        .run_script(
            "let v; if (true) { v = 1; } else { v = { x: 1 }; } v.x;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("dynamic property access should run");
    assert!(outcome.note.contains("undefined(undefined)"));
}

#[test]
fn wasm_backend_proxy_get_missing_trap_falls_back_to_object_target() {
    for (source, expected) in [
        (
            "let p = new Proxy({ attr: 1 }, { get: undefined }); p.attr;",
            "number(1)",
        ),
        (
            "let p = new Proxy({ attr: 1 }, { get: undefined }); p.foo;",
            "undefined(undefined)",
        ),
        (
            "let target = { get attr() { return this; } }; let p = Object.create(new Proxy(target, {})); p.attr === p;",
            "boolean(true)",
        ),
        (
            "let a = [1, 2, 3]; let p = new Proxy(new Proxy(a, {}), { get: undefined }); p.length;",
            "number(3)",
        ),
        (
            "let a = [1, 2, 3]; let p = new Proxy(new Proxy(a, {}), { get: undefined }); p[1];",
            "number(2)",
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
                panic!("proxy get missing-trap case should run for `{source}`: {err:?}")
            });
        assert!(
            outcome.note.contains(expected),
            "source: {source}, note: {}",
            outcome.note
        );
    }
}

#[test]
fn wasm_backend_proxy_get_non_callable_trap_throws_type_error() {
    let err = engine()
        .run_script(
            "let p = new Proxy({}, { get: {} }); p.attr;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect_err("non-callable proxy get trap should throw");
    assert!(err.message().contains("uncaught throw: TypeError"));
}

#[test]
fn wasm_backend_proxy_get_trap_receives_well_known_symbol_key() {
    let outcome = engine()
        .run_script(
            r#"
let seen = "";
let target = { next: function() { return { done: true }; } };
let p = new Proxy(target, {
  get: function(target, key, receiver) {
    if (String(key) === "Symbol(Symbol.iterator)") {
      seen = (typeof key) + ":" + (key === Symbol.iterator);
    }
    return Reflect.get(target, key, receiver);
  }
});
Iterator.from(p);
seen;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("proxy get trap should receive Symbol.iterator as a symbol");
    assert!(
        outcome.note.contains("string(symbol:true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_symbol_iterator_next() {
    let outcome = engine()
        .run_script(
            r#"
let array = new Int8Array([3, 1, 2]);
let iterator = array[Symbol.iterator]();
let first = iterator.next();
let second = iterator.next();
let third = iterator.next();
let exhausted = iterator.next();
let repeated = iterator.next();
let clamped = new Uint8ClampedArray([255, 1, 2]);
let clampedIterator = clamped[Symbol.iterator]();
let clampedFirst = clampedIterator.next();
first.value + ":" + first.done + "|" +
second.value + ":" + second.done + "|" +
third.value + ":" + third.done + "|" +
typeof exhausted.value + ":" + exhausted.done + "|" +
typeof repeated.value + ":" + repeated.done + "|" +
clampedFirst.value + ":" + clampedFirst.done;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("typed array Symbol.iterator next should run");
    assert!(
        outcome
            .note
            .contains("string(3:false|1:false|2:false|undefined:true|undefined:true|255:false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_push_is_generic_for_objects_and_boxed_booleans() {
    let outcome = engine()
            .run_script(
                r#"
var obj = {};
obj.push = Array.prototype.push;
var firstLen = obj.push(-1);
var first = firstLen + ":" + obj.length + ":" + obj["0"];

obj.length = null;
var nullLen = obj.push(-7);
var afterNull = nullLen + ":" + obj.length + ":" + obj["0"];

obj.length = 4294967296;
var largeLen = obj.push("x", "y");
var afterLarge =
  largeLen + ":" + obj.length + ":" + obj["4294967296"] + ":" + obj["4294967297"] + ":" + (obj["0"] === -7);

var boolLen = Array.prototype.push.call(true);
var stringThrow = "no";
try {
  Array.prototype.push.call("");
} catch (e) {
  stringThrow = e.name;
}
first + "|" + afterNull + "|" + afterLarge + "|" + boolLen + "|" + stringThrow;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.push generic object cases should run");
    assert!(
        outcome
            .note
            .contains("string(1:1:-1|1:1:-7|4294967298:4294967298:x:y:true|0|TypeError)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_push_preserves_max_array_length_boundary_property() {
    let outcome = engine()
        .run_script(
            r#"
var array = [];
array.length = 4294967295;
var first = array.push();
var caught = "no";
try {
  array.push("x");
} catch (e) {
  caught = e.name + ":" + (e instanceof RangeError);
}
caught + "|" + first + "|" + array.length + "|" + array[4294967295] + "|" + array["4294967295"];
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.push should store the boundary property before RangeError");
    assert!(
        outcome
            .note
            .contains("string(RangeError:true|4294967295|4294967295|x|x)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_reflect_set_calls_inherited_array_numeric_setter() {
    let outcome = engine()
        .run_script(
            r#"
var array = [];
var callCount = 0;
var receiverMatches = false;
Object.defineProperty(Array.prototype, "0", {
  set: function(value) {
    callCount += value;
    receiverMatches = this === array;
  },
  configurable: true
});

var result = Reflect.set(array, "0", 5);
delete Array.prototype[0];
result + ":" + callCount + ":" + receiverMatches + ":" + array.length + ":" +
  (array[0] === undefined);
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Reflect.set should honor inherited array numeric setters");
    assert!(
        outcome.note.contains("string(true:5:true:0:true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_reflect_set_propagates_accessor_exceptions() {
    let outcome = engine()
        .run_script(
            r#"
var target = {};
var receiver = {};
var token = {};
var caught;
var callCount = 0;
Object.defineProperty(target, "slot", {
  set: function() {
    callCount += 1;
    throw token;
  }
});
try {
  Reflect.set(target, "slot", 1, receiver);
} catch (error) {
  caught = error;
}
callCount === 1 && caught === token;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Reflect.set should propagate accessor exceptions");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_shift_observes_proxy_operations_in_spec_order() {
    let outcome = engine()
            .run_script(
                r#"
var calls = [];
var target = { 0: "a", 1: "b", length: 2 };
var proxy = new Proxy(target, {
  get: function(target, key, receiver) {
    calls.push("get " + key);
    return Reflect.get(target, key, receiver);
  },
  has: function(target, key) {
    calls.push("has " + key);
    return Reflect.has(target, key);
  },
  set: function(target, key, value, receiver) {
    calls.push("set " + key + " " + value);
    return Reflect.set(target, key, value, receiver);
  },
  deleteProperty: function(target, key) {
    calls.push("delete " + key);
    return Reflect.deleteProperty(target, key);
  }
});
var first = Array.prototype.shift.call(proxy);
var proxyResult = first + ":" + target.length + ":" + target[0] + ":" + (1 in target) + "|" + calls.join(",");

var boundaryCalls = [];
var boundary = new Proxy({}, {
  get: function(target, key) {
    boundaryCalls.push("get " + key);
    if (key === "length") return 9007199254740991;
    if (key === "0") return "first";
  },
  has: function(target, key) {
    throw key;
  }
});
var boundaryResult = "missing";
try {
  Array.prototype.shift.call(boundary);
} catch (error) {
  boundaryResult = error + ":" + boundaryCalls.join(",");
}

proxyResult + "|" + boundaryResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Array.prototype.shift should use observable object operations in spec order");
    assert!(
            outcome.note.contains(
                "string(a:1:b:false|get length,get 0,has 1,get 1,set 0 b,delete 1,set length 1|1:get length,get 0)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_proxy_revocable_get_after_revoke_throws_type_error() {
    let err = engine()
        .run_script(
            "let p = Proxy.revocable({}, {}); p.revoke(); p.proxy.attr;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect_err("revoked proxy get should throw");
    assert!(err.message().contains("uncaught throw: TypeError"));
}

#[test]
fn wasm_backend_create_realm_exposes_proxy_constructor() {
    let outcome = engine()
        .run_script(
            r#"
let OProxy = __scriptwCreateRealm().global.Proxy;
let p = new OProxy({ attr: 7 }, {});
(typeof OProxy) + ":" + (OProxy === Proxy) + ":" + (p.attr === 7);
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should expose Proxy constructor");
    assert!(
        outcome.note.contains("string(function:false:true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_create_realm_uses_realm_local_object_and_array_prototypes() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let array = new other.Array();
let object = new other.Object();
object.own = 1;
[
  other.Object === Object,
  other.Array === Array,
  other.Function.prototype === Function.prototype,
  Object.getPrototypeOf(other.Object) === other.Function.prototype,
  Object.getPrototypeOf(other.Array) === other.Function.prototype,
  other.Object.prototype === Object.prototype,
  other.Array.prototype === Array.prototype,
  Object.getPrototypeOf(other.Array.prototype) === other.Object.prototype,
  Object.getPrototypeOf(array) === other.Array.prototype,
  typeof other.Object.prototype.hasOwnProperty,
  typeof other.Object.prototype.propertyIsEnumerable,
  typeof other.Object.prototype.isPrototypeOf,
  typeof other.Object.prototype.toString,
  typeof other.Object.prototype.toLocaleString,
  typeof other.Object.prototype.valueOf,
  other.Object.prototype.hasOwnProperty === Object.prototype.hasOwnProperty,
  other.Object.prototype.propertyIsEnumerable === Object.prototype.propertyIsEnumerable,
  other.Object.prototype.isPrototypeOf === Object.prototype.isPrototypeOf,
  other.Object.prototype.toString === Object.prototype.toString,
  other.Object.prototype.toLocaleString === Object.prototype.toLocaleString,
  other.Object.prototype.valueOf === Object.prototype.valueOf,
  other.Object.prototype.hasOwnProperty.call(object, "own"),
  other.Object.prototype.propertyIsEnumerable.call(object, "own"),
  other.Object.prototype.isPrototypeOf.call(other.Object.prototype, object),
  other.Object.prototype.toString.call(object),
  other.Object.prototype.toLocaleString.call(object),
  other.Object.prototype.valueOf.call(object) === object
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should use realm-local Object/Array prototypes");
    assert!(
            outcome
                .note
                .contains("string(false|false|false|true|true|false|false|true|true|function|function|function|function|function|function|false|false|false|false|false|false|true|true|true|[object Object]|[object Object]|true)"),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_create_realm_uses_realm_local_function_constructor_and_prototype() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
function add(a, b) { return this.base + a + b; }
let receiver = { base: 10 };
let bound = other.Function.prototype.bind.call(add, receiver, 2);
[
  other.Function === Function,
  other.Function.prototype === Function.prototype,
  Object.getPrototypeOf(other.Function) === other.Function.prototype,
  Object.getPrototypeOf(other.Function.prototype) === other.Object.prototype,
  other.Function.prototype.constructor === other.Function,
  typeof other.Function.prototype.call,
  typeof other.Function.prototype.apply,
  typeof other.Function.prototype.bind,
  typeof other.Function.prototype.toString,
  other.Function.prototype.call === Function.prototype.call,
  other.Function.prototype.apply === Function.prototype.apply,
  other.Function.prototype.bind === Function.prototype.bind,
  other.Function.prototype.toString === Function.prototype.toString,
  other.Function.prototype.call.call(add, receiver, 1, 2),
  other.Function.prototype.apply.call(add, receiver, [3, 4]),
  bound(3)
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should use realm-local Function constructor and prototype");
    assert!(
            outcome
                .note
                .contains("string(false|false|true|true|true|function|function|function|function|false|false|false|false|13|17|15)"),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_create_realm_uses_realm_local_array_iterator_prototype() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let otherIter = other.Array.prototype.values.call(new other.Array(1));
let thisIter = Array.prototype.values.call([]);
let otherArrayIteratorPrototype = Object.getPrototypeOf(otherIter);
let thisArrayIteratorPrototype = Object.getPrototypeOf(thisIter);
let otherIteratorPrototype = Object.getPrototypeOf(otherArrayIteratorPrototype);
let thisIteratorPrototype = Object.getPrototypeOf(thisArrayIteratorPrototype);
[
  otherArrayIteratorPrototype === thisArrayIteratorPrototype,
  otherIteratorPrototype === thisIteratorPrototype,
  Object.getPrototypeOf(otherIteratorPrototype) === other.Object.prototype,
  typeof otherIter.next,
  otherArrayIteratorPrototype.next === thisArrayIteratorPrototype.next,
  otherArrayIteratorPrototype[Symbol.iterator] === thisArrayIteratorPrototype[Symbol.iterator],
  otherIter[Symbol.iterator]() === otherIter
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should use a realm-local Array Iterator prototype graph");
    assert!(
        outcome
            .note
            .contains("string(false|false|true|function|false|false|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_create_realm_uses_realm_local_typed_array_prototypes() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let otherTypedArrayPrototype = Object.getPrototypeOf(other.Uint8Array.prototype);
let thisTypedArrayPrototype = Object.getPrototypeOf(Uint8Array.prototype);
let view = new other.Uint8Array(2);
[
  other.Uint8Array === Uint8Array,
  other.Uint8Array.prototype === Uint8Array.prototype,
  otherTypedArrayPrototype === thisTypedArrayPrototype,
  Object.getPrototypeOf(otherTypedArrayPrototype) === other.Object.prototype,
  Object.getPrototypeOf(view) === other.Uint8Array.prototype,
  Object.getPrototypeOf(other.Uint8Array) === Object.getPrototypeOf(Uint8Array)
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should use realm-local typed array prototypes");
    assert!(
        outcome
            .note
            .contains("string(false|false|false|true|true|false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_create_realm_uses_realm_local_array_buffer_and_data_view_prototypes() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let buffer = new other.ArrayBuffer(8);
let view = new other.DataView(buffer);
[
  other.ArrayBuffer === ArrayBuffer,
  other.ArrayBuffer.prototype === ArrayBuffer.prototype,
  Object.getPrototypeOf(other.ArrayBuffer) === other.Function.prototype,
  Object.getPrototypeOf(other.ArrayBuffer.prototype) === other.Object.prototype,
  other.ArrayBuffer.prototype.constructor === other.ArrayBuffer,
  Object.getPrototypeOf(buffer) === other.ArrayBuffer.prototype,
  buffer instanceof other.ArrayBuffer,
  buffer instanceof ArrayBuffer,
  other.DataView === DataView,
  other.DataView.prototype === DataView.prototype,
  Object.getPrototypeOf(other.DataView) === other.Function.prototype,
  Object.getPrototypeOf(other.DataView.prototype) === other.Object.prototype,
  other.DataView.prototype.constructor === other.DataView,
  Object.getPrototypeOf(view) === other.DataView.prototype,
  view instanceof other.DataView,
  view instanceof DataView,
  other.ArrayBuffer.isView === ArrayBuffer.isView,
  typeof other.ArrayBuffer.isView,
  other.ArrayBuffer.isView(view),
  other.ArrayBuffer.isView(new other.Uint8Array(buffer)),
  other.ArrayBuffer.isView(buffer),
  other.ArrayBuffer.isView({})
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should use realm-local ArrayBuffer and DataView prototypes");
    assert!(
            outcome.note.contains(
                "string(false|false|true|true|true|true|true|false|false|false|true|true|true|true|true|false|false|function|true|true|false|false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_create_realm_uses_realm_local_bigint_prototype() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
[
  other.BigInt.prototype === BigInt.prototype,
  Object.getPrototypeOf(other.BigInt.prototype) === other.Object.prototype,
  other.BigInt.prototype.constructor === other.BigInt
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should use realm-local BigInt prototype");
    assert!(
        outcome.note.contains("string(false|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_create_realm_uses_realm_local_date_constructor_and_prototype() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let date = new other.Date(0);
let thisDate = new Date(0);
let setResult = date.setUTCFullYear(2001, 1, 3);
[
  other.Date === Date,
  other.Date.prototype === Date.prototype,
  Object.getPrototypeOf(other.Date.prototype) === other.Object.prototype,
  Object.getPrototypeOf(date) === other.Date.prototype,
  typeof other.Date.now,
  typeof other.Date.UTC,
  typeof other.Date.prototype.setUTCFullYear,
  typeof other.Date.prototype.toUTCString,
  other.Date.prototype.toGMTString === other.Date.prototype.toUTCString,
  other.Date.now() >= 946684800000,
  other.Date.UTC(1970, 0, 1),
  date.getTime(),
  date.getUTCFullYear(),
  setResult === date.getTime(),
  Date.prototype.getTime.call(date),
  other.Date.prototype.getTime.call(thisDate),
  other.Date.prototype.getUTCFullYear.call(thisDate)
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should use realm-local Date constructor and prototype");
    assert!(
            outcome.note.contains(
                "string(false|false|true|true|function|function|function|function|true|true|0|981158400000|2001|true|981158400000|0|1970)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_create_realm_uses_realm_local_error_constructors_and_prototypes() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let names = [
  "Error", "EvalError", "RangeError", "ReferenceError", "SyntaxError",
  "TypeError", "URIError", "AggregateError", "SuppressedError"
];
let results = [];

for (let i = 0; i < names.length; i++) {
  let name = names[i];
  let C = other[name];
  let error =
    name === "AggregateError" ? new C([], "m") :
    name === "SuppressedError" ? new C("e", "s", "m") :
    new C("m");
  let expectedParent =
    name === "Error" ? other.Object.prototype : other.Error.prototype;
  let expectedConstructorParent =
    name === "Error" ? other.Function.prototype : other.Error;

  results.push(
    name + "=" +
    (C === globalThis[name]) + ":" +
    (C.prototype === globalThis[name].prototype) + ":" +
    (Object.getPrototypeOf(C) === expectedConstructorParent) + ":" +
    (C.prototype.constructor === C) + ":" +
    (Object.getPrototypeOf(C.prototype) === expectedParent) + ":" +
    (Object.getPrototypeOf(error) === C.prototype) + ":" +
    (error instanceof C) + ":" +
    (error instanceof other.Error) + ":" +
    (error instanceof Error)
  );
}

results.join(",");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should use realm-local error constructors and prototypes");
    assert!(
            outcome.note.contains(
                "string(Error=false:false:true:true:true:true:true:true:false,EvalError=false:false:true:true:true:true:true:true:false,RangeError=false:false:true:true:true:true:true:true:false,ReferenceError=false:false:true:true:true:true:true:true:false,SyntaxError=false:false:true:true:true:true:true:true:false,TypeError=false:false:true:true:true:true:true:true:false,URIError=false:false:true:true:true:true:true:true:false,AggregateError=false:false:true:true:true:true:true:true:false,SuppressedError=false:false:true:true:true:true:true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_create_realm_uses_realm_local_error_prototype_to_string() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let otherError = new other.Error("m");
let thisError = new Error("m");
let object = new other.Object();
object.name = "OtherName";
object.message = "OtherMessage";
[
  other.Error.prototype.toString === Error.prototype.toString,
  typeof other.Error.prototype.toString,
  other.Error.prototype.toString.call(otherError),
  other.Error.prototype.toString.call(thisError),
  Error.prototype.toString.call(otherError),
  other.Error.prototype.toString.call(object)
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should use realm-local Error.prototype.toString");
    assert!(
        outcome
            .note
            .contains("string(false|function|Error: m|Error: m|Error: m|OtherName: OtherMessage)"),
        "note: {}",
        outcome.note
    );
}

// Proxy [[Call]] and [[Construct]] (ECMA-262 10.5.12 and 10.5.13, step 1)
// perform ValidateNonRevokedProxy (10.5.14), whose TypeError comes from
// the current Realm. A Proxy exotic object has no [[Realm]] and pushes no
// execution context, so that Realm is the caller's, never the Realm of the
// Proxy constructor or of the target. Test262 pins the same rule in
// built-ins/Proxy/{apply,construct}/null-handler-realm.js.
#[test]
fn wasm_backend_cross_realm_revoked_proxy_throws_running_realm_type_error() {
    let outcome = engine()
            .run_script(
                r#"
let other = __scriptwCreateRealm();
let OProxy = other.global.Proxy;
let sameRealm = (new TypeError("same")) instanceof TypeError;
function describe(operation) {
  try {
    operation();
    return "missing";
  } catch (error) {
    return (Object.getPrototypeOf(error) === TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }
}
let localTarget = OProxy.revocable(function() {}, {});
localTarget.revoke();
let otherTarget = OProxy.revocable(other.global.Object, {});
otherTarget.revoke();
let mainRevocable = Proxy.revocable(function() {}, {});
mainRevocable.revoke();
other.global.mainRevokedProxy = mainRevocable.proxy;
let otherCallerError = other.evalScript("var caught = 'missing'; try { mainRevokedProxy(); } catch (error) { caught = error; } caught;");
[
  sameRealm,
  describe(() => localTarget.proxy()),
  describe(() => new localTarget.proxy()),
  describe(() => otherTarget.proxy()),
  describe(() => new otherTarget.proxy()),
  (Object.getPrototypeOf(otherCallerError) === other.global.TypeError.prototype) + ":" +
    (otherCallerError instanceof TypeError)
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("revoked cross-realm proxy call should throw in the running realm");
    assert!(
            outcome.note.contains(
                "string(true|true:false:true|true:false:true|true:false:true|true:false:true|true:false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_create_realm_exposes_object_static_methods() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let proto = new other.Object();
other.Object.defineProperty(proto, "inherited", { value: 1, enumerable: true });
let object = other.Object.create(proto);
other.Object.defineProperty(object, "own", { value: 2, enumerable: true });
let replacement = new other.Object();
let setResult = other.Object.setPrototypeOf(object, replacement);
let desc = other.Object.getOwnPropertyDescriptor(object, "own");
let keys = other.Object.keys(object);
[
  typeof other.Object.create,
  typeof other.Object.getPrototypeOf,
  typeof other.Object.setPrototypeOf,
  typeof other.Object.defineProperty,
  typeof other.Object.getOwnPropertyDescriptor,
  typeof other.Object.keys,
  other.Object.getPrototypeOf(proto) === other.Object.prototype,
  Object.getPrototypeOf(proto) === Object.prototype,
  setResult === object,
  other.Object.getPrototypeOf(object) === replacement,
  other.Object.getPrototypeOf(replacement) === other.Object.prototype,
  desc.value,
  desc.enumerable,
  desc.configurable,
  keys.length,
  keys[0]
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm Object statics should operate on the other realm graph");
    assert!(
            outcome.note.contains(
                "string(function|function|function|function|function|function|true|false|true|true|true|2|true|false|1|own)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_create_realm_exposes_object_integrity_static_methods() {
    let outcome = engine()
            .run_script(
                r#"
let other = __scriptwCreateRealm().global;
let object = new other.Object();
let defined = other.Object.defineProperties(object, {
  a: { value: 1, enumerable: true },
  hidden: { value: 2 }
});
let extensibleBefore = other.Object.isExtensible(object);
let prevented = other.Object.preventExtensions(object);
let extensibleAfter = other.Object.isExtensible(object);
let frozenTarget = new other.Object();
other.Object.defineProperty(frozenTarget, "value", { value: 3, writable: true, configurable: true });
let frozen = other.Object.freeze(frozenTarget);
[
  typeof other.Object.defineProperties,
  typeof other.Object.hasOwn,
  typeof other.Object.is,
  typeof other.Object.freeze,
  typeof other.Object.isFrozen,
  typeof other.Object.isSealed,
  typeof other.Object.isExtensible,
  typeof other.Object.preventExtensions,
  defined === object,
  other.Object.hasOwn(object, "a"),
  other.Object.hasOwn(object, "hidden"),
  other.Object.hasOwn(object, "missing"),
  other.Object.is(NaN, NaN),
  other.Object.is(0, -0),
  extensibleBefore,
  prevented === object,
  extensibleAfter,
  frozen === frozenTarget,
  other.Object.isFrozen(frozenTarget),
  other.Object.isSealed(frozenTarget),
  other.Object.isExtensible(frozenTarget)
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm Object integrity statics should operate on objects");
    assert!(
            outcome.note.contains(
                "string(function|function|function|function|function|function|function|function|true|true|true|false|true|false|true|true|false|true|true|true|false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_create_realm_exposes_object_property_list_static_methods() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let symbol = Symbol("s");
let object = new other.Object();
other.Object.defineProperty(object, "a", { value: 1, enumerable: true });
other.Object.defineProperty(object, "hidden", { value: 2 });
other.Object.defineProperty(object, symbol, { value: 3, enumerable: true });
let names = other.Object.getOwnPropertyNames(object);
let symbols = other.Object.getOwnPropertySymbols(object);
let values = other.Object.values(object);
let entries = other.Object.entries(object);
let nullErrorFromOtherRealm = false;
try {
  other.Object.entries(null);
} catch (error) {
  nullErrorFromOtherRealm =
    Object.getPrototypeOf(error) === other.TypeError.prototype;
}
[
  typeof other.Object.getOwnPropertyNames,
  typeof other.Object.getOwnPropertySymbols,
  typeof other.Object.values,
  typeof other.Object.entries,
  names.length,
  names[0],
  names[1],
  symbols.length,
  symbols[0] === symbol,
  values.length,
  values[0],
  entries.length,
  entries[0][0],
  entries[0][1],
  Object.getPrototypeOf(values) === other.Array.prototype,
  Object.getPrototypeOf(entries) === other.Array.prototype,
  Object.getPrototypeOf(entries[0]) === other.Array.prototype,
  nullErrorFromOtherRealm
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm Object property-list statics should operate on objects");
    assert!(
            outcome
                .note
                .contains("string(function|function|function|function|2|a|hidden|1|true|1|1|1|a|1|true|true|true|true)"),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_create_realm_object_property_copy_methods_use_their_defining_realm() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let target = {};
let assigned = other.Object.assign(target, { a: 1 });
let boxed = other.Object.assign("x", { a: 2 });
let descriptors = other.Object.getOwnPropertyDescriptors({ a: 3 });
let assignNullErrorFromOtherRealm = false;
try {
  other.Object.assign(null, {});
} catch (error) {
  assignNullErrorFromOtherRealm =
    Object.getPrototypeOf(error) === other.TypeError.prototype;
}
let descriptorsNullErrorFromOtherRealm = false;
try {
  other.Object.getOwnPropertyDescriptors(null);
} catch (error) {
  descriptorsNullErrorFromOtherRealm =
    Object.getPrototypeOf(error) === other.TypeError.prototype;
}
[
  typeof other.Object.assign,
  typeof other.Object.getOwnPropertyDescriptors,
  assigned === target,
  target.a,
  boxed.a,
  Object.getPrototypeOf(boxed) === other.String.prototype,
  descriptors.a.value,
  Object.getPrototypeOf(descriptors) === other.Object.prototype,
  Object.getPrototypeOf(descriptors.a) === other.Object.prototype,
  assignNullErrorFromOtherRealm,
  descriptorsNullErrorFromOtherRealm
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm Object property-copy methods should use their defining realm");
    assert!(
        outcome
            .note
            .contains("string(function|function|true|1|2|true|3|true|true|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_create_realm_exposes_reflect_object_methods() {
    let outcome = engine()
            .run_script(
                r#"
let other = __scriptwCreateRealm().global;
let object = new other.Object();
let proto = new other.Object();
let constructed = other.Reflect.construct(other.Object, []);
let defineResult = other.Reflect.defineProperty(object, "a", { value: 1, enumerable: true, configurable: true });
let setResult = other.Reflect.set(object, "b", 2);
let getResult = other.Reflect.get(object, "a");
let hasResult = other.Reflect.has(object, "b");
let desc = other.Reflect.getOwnPropertyDescriptor(object, "a");
let setProtoResult = other.Reflect.setPrototypeOf(object, proto);
let reflectedProto = other.Reflect.getPrototypeOf(object);
let keys = other.Reflect.ownKeys(object);
let deleteResult = other.Reflect.deleteProperty(object, "a");
let beforePreventExtensible = other.Reflect.isExtensible(object);
let preventResult = other.Reflect.preventExtensions(object);
[
  typeof other.Reflect,
  Object.getPrototypeOf(other.Reflect) === other.Object.prototype,
  typeof other.Reflect.construct,
  typeof other.Reflect.apply,
  typeof other.Reflect.get,
  typeof other.Reflect.getPrototypeOf,
  typeof other.Reflect.getOwnPropertyDescriptor,
  typeof other.Reflect.set,
  typeof other.Reflect.has,
  typeof other.Reflect.defineProperty,
  typeof other.Reflect.deleteProperty,
  typeof other.Reflect.isExtensible,
  typeof other.Reflect.preventExtensions,
  typeof other.Reflect.setPrototypeOf,
  typeof other.Reflect.ownKeys,
  Object.getPrototypeOf(constructed) === other.Object.prototype,
  defineResult,
  setResult,
  getResult,
  hasResult,
  desc.value,
  desc.enumerable,
  setProtoResult,
  reflectedProto === proto,
  Object.getPrototypeOf(object) === proto,
  keys.length,
  keys[0],
  keys[1],
  deleteResult,
  other.Reflect.has(object, "a"),
  beforePreventExtensible,
  preventResult,
  Object.isExtensible(object),
  other.Reflect.isExtensible(object)
].join("|");
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("synthetic realm Reflect object should expose object meta operations");
    assert!(
            outcome.note.contains(
                "string(object|true|function|function|function|function|function|function|function|function|function|function|function|function|function|true|true|true|1|true|1|true|true|true|true|2|a|b|true|false|true|true|false|false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_reflect_to_string_tag_describes_main_and_synthetic_realm_objects() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let localDescriptor = Object.getOwnPropertyDescriptor(Reflect, Symbol.toStringTag);
let otherDescriptor = other.Object.getOwnPropertyDescriptor(
  other.Reflect,
  other.Symbol.toStringTag
);
[
  localDescriptor.value,
  localDescriptor.writable,
  localDescriptor.enumerable,
  localDescriptor.configurable,
  Object.prototype.toString.call(Reflect),
  otherDescriptor.value,
  otherDescriptor.writable,
  otherDescriptor.enumerable,
  otherDescriptor.configurable,
  Object.prototype.toString.call(other.Reflect),
  other.Object.prototype.toString.call(Reflect),
  other.Object.prototype.toString.call(other.Reflect)
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Reflect toStringTag should describe main and synthetic realm objects");
    assert!(
            outcome.note.contains(
                "string(Reflect|false|false|true|[object Reflect]|Reflect|false|false|true|[object Reflect]|[object Reflect]|[object Reflect])"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_reflect_validates_targets_before_property_keys_in_its_defining_realm() {
    let outcome = engine()
        .run_script(
            r#"
var coercions = 0;
var key = {
  [Symbol.toPrimitive]: function() {
    coercions++;
    return "x";
  }
};
var invalidTargets = [undefined, null, true, 1, 1n, "x", Symbol("x")];
var localErrors = true;
for (var i = 0; i < invalidTargets.length; i++) {
  try {
    Reflect.get(invalidTargets[i], key);
    localErrors = false;
  } catch (error) {
    localErrors = localErrors && Object.getPrototypeOf(error) === TypeError.prototype;
  }
  try {
    Reflect.has(invalidTargets[i], key);
    localErrors = false;
  } catch (error) {
    localErrors = localErrors && Object.getPrototypeOf(error) === TypeError.prototype;
  }
}

var other = __scriptwCreateRealm().global;
var otherGetError = false;
var otherHasError = false;
var otherOwnKeysError = false;
try {
  other.Reflect.get(1, "x");
} catch (error) {
  otherGetError = Object.getPrototypeOf(error) === other.TypeError.prototype;
}
try {
  other.Reflect.has(Symbol("x"), "x");
} catch (error) {
  otherHasError = Object.getPrototypeOf(error) === other.TypeError.prototype;
}
try {
  other.Reflect.ownKeys(null);
} catch (error) {
  otherOwnKeysError = Object.getPrototypeOf(error) === other.TypeError.prototype;
}

[
  localErrors,
  coercions,
  otherGetError,
  otherHasError,
  otherOwnKeysError
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Reflect target validation should use the method defining realm");
    assert!(
        outcome.note.contains("string(true|0|true|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_proxy_internal_methods_accept_callable_proxy_traps() {
    let outcome = engine()
        .run_script(
            r#"
var applyTrap = new Proxy(function(target, thisArgument, argumentsList) {
  return target.apply(thisArgument, argumentsList);
}, {});
var callable = new Proxy(function(left, right) {
  return this.prefix + left + right;
}, { apply: applyTrap });

var constructTrap = new Proxy(function(target, argumentsList, newTarget) {
  return Reflect.construct(target, argumentsList, newTarget);
}, {});
function Pair(left, right) {
  this.value = left + right;
}
var constructable = new Proxy(Pair, { construct: constructTrap });

var prototype = {};
var getPrototypeOfTrap = new Proxy(function() {
  return prototype;
}, {});
var getPrototypeOfTarget = new Proxy({}, { getPrototypeOf: getPrototypeOfTrap });

var setPrototypeOfCalls = 0;
var setPrototypeOfTrap = new Proxy(function() {
  setPrototypeOfCalls++;
  return true;
}, {});
var setPrototypeOfTarget = new Proxy({}, { setPrototypeOf: setPrototypeOfTrap });

var isExtensibleTrap = new Proxy(function(target) {
  return Reflect.isExtensible(target);
}, {});
var isExtensibleTarget = new Proxy({}, { isExtensible: isExtensibleTrap });

var descriptorTrap = new Proxy(function(target, key) {
  return Reflect.getOwnPropertyDescriptor(target, key);
}, {});
var descriptorTarget = new Proxy({ present: 1 }, {
  getOwnPropertyDescriptor: descriptorTrap
});

[
  Reflect.apply(callable, { prefix: ">" }, ["a", "b"]),
  Reflect.construct(constructable, [2, 3]).value,
  Object.getPrototypeOf(getPrototypeOfTarget) === prototype,
  Reflect.setPrototypeOf(setPrototypeOfTarget, null),
  setPrototypeOfCalls,
  Reflect.isExtensible(isExtensibleTarget),
  Object.hasOwn(descriptorTarget, "present")
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("proxy internal methods should call callable proxy traps");
    assert!(
        outcome.note.contains("string(>ab|5|true|true|1|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_has_property_dispatches_every_live_exotic_branch() {
    let outcome = engine()
        .run_script(
            r#"
var ordinary = Object.create({ inherited: 1 });

var arrayCalls = [];
var array = [];
Object.setPrototypeOf(array, new Proxy({ inheritedIndex: true }, {
  has: function(target, key) {
    arrayCalls.push("array " + key);
    return Reflect.has(target, key);
  }
}));

var typed = new Uint8Array([7]);
var typedSymbol = Symbol("typed");
Object.defineProperty(typed, typedSymbol, { value: true });
var typedCalls = [];
Object.setPrototypeOf(typed, new Proxy({ typedProto: true }, {
  has: function(target, key) {
    typedCalls.push("typed " + key);
    return Reflect.has(target, key);
  }
}));

var argumentsCalls = [];
var argumentsResult = [];
var argumentsTarget;
function probeArguments() {
  var args = arguments;
  argumentsTarget = args;
  Object.setPrototypeOf(args, new Proxy({ argProto: true }, {
    has: function(target, key) {
      argumentsCalls.push("arguments " + key);
      return Reflect.has(target, key);
    }
  }));
  argumentsResult = [1 in args, "argProto" in args];
}
probeArguments(0, 1);
var absentOuter = new Proxy(
  new Proxy(argumentsTarget, { has: undefined }),
  { has: undefined }
);

var boxed = new String("xy");
boxed.extra = 1;

function PrototypeCarrier() {}
var functionChild = Object.create(PrototypeCarrier);

var nestedCalls = [];
var inner = new Proxy({ nested: 1 }, {
  has: function(target, key) {
    nestedCalls.push("inner " + key);
    return Reflect.has(target, key);
  }
});
var outer = new Proxy(inner, { has: undefined });

var callableCalls = [];
var callableTrap = new Proxy(function(_target, key) {
  callableCalls.push(key);
  return key === "callable";
}, {});
var callable = new Proxy({}, { has: callableTrap });

[
  "inherited" in ordinary,
  "inheritedIndex" in array,
  0 in typed,
  "-0" in typed,
  typedSymbol in typed,
  "typedProto" in typed,
  argumentsResult[0],
  argumentsResult[1],
  1 in absentOuter,
  "1" in boxed,
  "extra" in boxed,
  "prototype" in functionChild,
  "nested" in outer,
  "callable" in callable,
  arrayCalls.join(","),
  typedCalls.join(","),
  argumentsCalls.join(","),
  nestedCalls.join(","),
  callableCalls.join(",")
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("HasProperty should reclassify each target and prototype object");
    assert!(
            outcome.note.contains(
                "string(true|true|true|false|true|true|true|true|true|true|true|true|true|true|array inheritedIndex|typed typedProto|arguments argProto|inner nested|callable)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_proxy_has_preserves_handler_tag_and_full_getmethod() {
    let outcome = engine()
        .run_script(
            r#"
var calls = [];
function trap(label, expected) {
  return function(_target, key) {
    calls.push(label + ":" + (this === expected) + ":" + key);
    return true;
  };
}

function functionHandler() {}
Object.setPrototypeOf(functionHandler, {
  has: trap("function", functionHandler)
});

var arrayHandler = [];
Object.setPrototypeOf(arrayHandler, {
  has: trap("array", arrayHandler)
});

var argumentsHandler;
(function() {
  argumentsHandler = arguments;
  Object.setPrototypeOf(argumentsHandler, {
    has: trap("arguments", argumentsHandler)
  });
})();

var nestedHandler;
var nestedTrap = {
  get: function(_target, key) {
    calls.push("nested-get:" + key);
    return function(_subject, subjectKey) {
      calls.push("nested:" + (this === nestedHandler) + ":" + subjectKey);
      return true;
    };
  }
};
nestedHandler = new Proxy({}, nestedTrap);

var abruptHandler = {
  get has() {
    calls.push("getter");
    throw "sentinel";
  }
};
var caught;
try {
  "abrupt" in new Proxy({}, abruptHandler);
} catch (error) {
  caught = error;
}

[
  "function" in new Proxy({}, functionHandler),
  "array" in new Proxy({}, arrayHandler),
  "arguments" in new Proxy({}, argumentsHandler),
  "nested" in new Proxy({}, nestedHandler),
  caught === "sentinel",
  calls.join(",")
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Proxy has should preserve the complete tagged handler");
    assert!(
            outcome.note.contains(
                "string(true|true|true|true|true|getter,function:true:function,array:true:array,arguments:true:arguments,nested-get:has,nested:true:nested)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_proxy_descriptor_and_extensibility_preserve_handler_tags() {
    let outcome = engine()
        .run_script(
            r#"
var calls = [];

function descriptorTrap(label, handler) {
  return function(target, key) {
    calls.push(label + ":" + (this === handler) + ":" + key);
    return Reflect.getOwnPropertyDescriptor(target, key);
  };
}

function functionHandler() {}
functionHandler.getOwnPropertyDescriptor = descriptorTrap(
  "function",
  functionHandler
);

var arrayHandler = [];
arrayHandler.getOwnPropertyDescriptor = descriptorTrap("array", arrayHandler);

var argumentsHandler = (function() { return arguments; })();
argumentsHandler.isExtensible = function(target) {
  calls.push("arguments:" + (this === argumentsHandler));
  return Reflect.isExtensible(target);
};

var nestedHandler;
nestedHandler = new Proxy({}, {
  get: function(_target, key) {
    calls.push("nested-get:" + key);
    if (key === "isExtensible") {
      return function(target) {
        calls.push("nested:" + (this === nestedHandler));
        return Reflect.isExtensible(target);
      };
    }
  }
});

var marker = {};
var abruptHandler = new Proxy({}, {
  get: function(_target, key) {
    if (key === "getOwnPropertyDescriptor" || key === "isExtensible") {
      throw marker;
    }
  }
});
var descriptorAbrupt = false;
try {
  Object.getOwnPropertyDescriptor(new Proxy({}, abruptHandler), "missing");
} catch (error) {
  descriptorAbrupt = error === marker;
}
var extensibilityAbrupt = false;
try {
  Reflect.isExtensible(new Proxy({}, abruptHandler));
} catch (error) {
  extensibilityAbrupt = error === marker;
}

[
  Object.getOwnPropertyDescriptor(
    new Proxy({ first: 1 }, functionHandler),
    "first"
  ).value,
  Reflect.getOwnPropertyDescriptor(
    new Proxy({ second: 2 }, arrayHandler),
    "second"
  ).value,
  Object.isExtensible(new Proxy({}, argumentsHandler)),
  Reflect.isExtensible(new Proxy({}, nestedHandler)),
  descriptorAbrupt,
  extensibilityAbrupt,
  calls.join(",")
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Proxy descriptor and extensibility methods should retain handler tags");
    assert!(
            outcome.note.contains(
                "string(1|2|true|true|true|true|function:true:first,array:true:second,arguments:true,nested-get:isExtensible,nested:true)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_proxy_descriptor_facts_cover_direct_exotic_targets() {
    let outcome = engine()
        .run_script(
            r#"
function rejectsTypeError(operation) {
  try {
    operation();
    return false;
  } catch (error) {
    return error instanceof TypeError;
  }
}

function falseHas(target) {
  return new Proxy(target, {
    has: function() { return false; }
  });
}

function hidesDescriptor(target) {
  return new Proxy(target, {
    getOwnPropertyDescriptor: function() { return undefined; }
  });
}

var arrayIndex = [1];
Object.defineProperty(arrayIndex, "0", { configurable: false });

var arrayOrdinaryIndex = [1];
Object.preventExtensions(arrayOrdinaryIndex);

var arrayNamed = [];
arrayNamed.named = 1;
Object.preventExtensions(arrayNamed);

var allFalse = {};
Object.defineProperty(allFalse, "fixed", { value: 1 });

var typed = new Uint8Array(1);
Object.preventExtensions(typed);

var argumentsTarget = (function() { return arguments; })(1);
Object.defineProperty(argumentsTarget, "length", {
  writable: false,
  enumerable: false,
  configurable: false
});
var argumentsLength = Object.getOwnPropertyDescriptor(argumentsTarget, "length");

var argumentsRecreatedLength = (function() { return arguments; })(1);
delete argumentsRecreatedLength.length;
argumentsRecreatedLength.length = 1;
var recreatedArgumentsLength = Object.getOwnPropertyDescriptor(
  argumentsRecreatedLength,
  "length"
);
Object.preventExtensions(argumentsRecreatedLength);

var argumentsOrdinaryIndex = (function() { return arguments; })(1);
Object.preventExtensions(argumentsOrdinaryIndex);

var nonExtensibleArray = [];
Object.preventExtensions(nonExtensibleArray);
var fabricatedDescriptor = new Proxy(nonExtensibleArray, {
  getOwnPropertyDescriptor: function() {
    return {
      value: 1,
      writable: true,
      enumerable: true,
      configurable: true
    };
  }
});

var absent = [];
Object.preventExtensions(absent);

[
  rejectsTypeError(function() { return "length" in falseHas([1]); }),
  rejectsTypeError(function() {
    return Object.getOwnPropertyDescriptor(hidesDescriptor([1]), "length");
  }),
  rejectsTypeError(function() {
    return Object.getOwnPropertyDescriptor(new Proxy([1], {
      getOwnPropertyDescriptor: function() {
        return {
          value: 1,
          writable: true,
          enumerable: false,
          configurable: true
        };
      }
  }), "length");
  }),
  rejectsTypeError(function() { return 0 in falseHas(arrayIndex); }),
  rejectsTypeError(function() { return 0 in falseHas(arrayOrdinaryIndex); }),
  rejectsTypeError(function() { return "named" in falseHas(arrayNamed); }),
  rejectsTypeError(function() { return "fixed" in falseHas(allFalse); }),
  rejectsTypeError(function() { return "1" in falseHas(new String("xy")); }),
  rejectsTypeError(function() { return 0 in falseHas(typed); }),
  argumentsLength.writable === false &&
    argumentsLength.enumerable === false &&
    argumentsLength.configurable === false,
  rejectsTypeError(function() { return "length" in falseHas(argumentsTarget); }),
  rejectsTypeError(function() {
    return Object.getOwnPropertyDescriptor(hidesDescriptor(argumentsTarget), "length");
  }),
  recreatedArgumentsLength.writable === true &&
    recreatedArgumentsLength.enumerable === true &&
    recreatedArgumentsLength.configurable === true &&
    rejectsTypeError(function() {
      return "length" in falseHas(argumentsRecreatedLength);
    }),
  rejectsTypeError(function() { return 0 in falseHas(argumentsOrdinaryIndex); }),
  rejectsTypeError(function() {
    return Object.getOwnPropertyDescriptor(fabricatedDescriptor, "new");
  }),
  ("missing" in falseHas(absent)) === false
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Proxy descriptor facts should cover every direct target representation");
    assert!(
            outcome.note.contains(
                "string(true|true|true|true|true|true|true|true|true|true|true|true|true|true|true|true)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_prototype_and_extensibility_methods_cross_deep_proxy_chains() {
    let outcome = engine()
        .run_script(
            r#"
function wrapDeeply(target) {
  for (var depth = 0; depth < 12; depth++) {
    target = new Proxy(target, {});
  }
  return target;
}

var prototype = {};
var getPrototypeOfCalls = 0;
var getPrototypeOfTarget = wrapDeeply(new Proxy({}, {
  getPrototypeOf: function() {
    getPrototypeOfCalls++;
    return prototype;
  }
}));

var setPrototypeOfCalls = 0;
var setPrototypeOfTarget = wrapDeeply(new Proxy({}, {
  setPrototypeOf: function() {
    setPrototypeOfCalls++;
    return true;
  }
}));

var isExtensibleCalls = 0;
var isExtensibleTarget = wrapDeeply(new Proxy({}, {
  isExtensible: function(target) {
    isExtensibleCalls++;
    return Reflect.isExtensible(target);
  }
}));

[
  Object.getPrototypeOf(getPrototypeOfTarget) === prototype,
  getPrototypeOfCalls,
  Reflect.setPrototypeOf(setPrototypeOfTarget, null),
  setPrototypeOfCalls,
  Reflect.isExtensible(isExtensibleTarget),
  isExtensibleCalls
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("prototype and extensibility methods should cross deep proxy chains");
    assert!(
        outcome.note.contains("string(true|1|true|1|true|1)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_arguments_inherits_object_to_string() {
    let outcome = engine()
        .run_script(
            r#"
function describeArguments() {
  return arguments.toString();
}
describeArguments(1, 2);
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("arguments.toString should execute");
    assert!(
        outcome.note.contains("string([object Arguments])"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_create_realm_exposes_math_object_methods_and_constants() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
[
  other.Math === Math,
  Object.getPrototypeOf(other.Math) === other.Object.prototype,
  Object.prototype.toString.call(other.Math),
  typeof other.Math.max,
  typeof other.Math.pow,
  other.Math.max(1, 7, 3),
  other.Math.pow(2, 5),
  other.Math.trunc(3.9),
  other.Math.PI > 3 && other.Math.PI < 4
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm Math object should expose Math operations");
    assert!(
        outcome
            .note
            .contains("string(false|true|[object Math]|function|function|7|32|3|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_create_realm_exposes_json_object_methods() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let raw = other.JSON.rawJSON("1");
[
  other.JSON === JSON,
  Object.getPrototypeOf(other.JSON) === other.Object.prototype,
  Object.prototype.toString.call(other.JSON),
  typeof other.JSON.parse,
  typeof other.JSON.stringify,
  typeof other.JSON.rawJSON,
  typeof other.JSON.isRawJSON,
  other.JSON.parse("2"),
  other.JSON.stringify({ a: 1 }),
  other.JSON.isRawJSON(raw),
  other.JSON.isRawJSON({})
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm JSON object should expose JSON operations");
    assert!(
            outcome.note.contains(
                "string(false|true|[object JSON]|function|function|function|function|2|{\"a\":1}|true|false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_cross_realm_proxy_wrapped_builtin_throws_defining_realm_type_error() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm();
let method = other.global.String.prototype.toString;
let localProxy = new Proxy(method, {});
let otherProxy = new other.global.Proxy(method, {});
let localResult = "missing";
let otherResult = "missing";

try {
  localProxy.call(null);
} catch (error) {
  localResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

try {
  otherProxy.call(null);
} catch (error) {
  otherResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

localResult + "|" + otherResult;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("proxy-wrapped cross-realm builtin should throw in target defining realm");
    assert!(
        outcome
            .note
            .contains("string(true:true:false|true:true:false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_create_realm_uses_realm_local_regexp_constructor_and_prototype() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
[
  other.RegExp === RegExp,
  other.RegExp.prototype === RegExp.prototype,
  Object.getPrototypeOf(other.RegExp) === other.Function.prototype,
  Object.getPrototypeOf(other.RegExp.prototype) === other.Object.prototype,
  other.RegExp.prototype.constructor === other.RegExp,
  typeof other.RegExp.escape,
  other.RegExp.escape === RegExp.escape
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should use realm-local RegExp constructor and prototype");
    assert!(
        outcome
            .note
            .contains("string(false|false|true|true|true|function|false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_regexp_source_getter_rejects_cross_realm_prototypes() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let get = Object.getOwnPropertyDescriptor(RegExp.prototype, "source").get;
let otherGet = Object.getOwnPropertyDescriptor(other.RegExp.prototype, "source").get;
let primaryError = false;
let otherError = false;

try {
  get.call(other.RegExp.prototype);
} catch (error) {
  primaryError = Object.getPrototypeOf(error) === TypeError.prototype;
}
try {
  otherGet.call(RegExp.prototype);
} catch (error) {
  otherError = Object.getPrototypeOf(error) === other.TypeError.prototype;
}
primaryError + "|" + otherError;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("RegExp source getters should reject another realm's prototype");
    assert!(
        outcome.note.contains("string(true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_regexp_split_observes_symbol_match_getter_recompile() {
    let outcome = engine()
        .run_script(
            r#"
var regExp = /a/;
Object.defineProperty(regExp, Symbol.match, {
  get: function() {
    regExp.compile("b");
  }
});
regExp[Symbol.split]("abba").join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("RegExp split should observe Symbol.match getter side effects");
    assert!(
        outcome.note.contains("string(a||a)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_outlines_optional_computed_property_reads_after_nullish_check() {
    let outcome = engine()
        .run_script(
            r#"
let keyCalls = 0;
let key = { [Symbol.toPrimitive]() { keyCalls += 1; return "value"; } };
let object = { value: 42, undefined: 1, null: 2, true: 3, NaN: 4 };
let absent = null;
let skipped = absent?.[key];
let results = [
  object?.[key], object?.[undefined], object?.[null], object?.[true], object?.[NaN],
  object?.["value"], object?.[key], object?.[undefined], object?.[null],
  object?.[true], object?.[NaN], object?.["value"], object?.[key]
];
[skipped, keyCalls, results.join("|")].join(";");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("optional computed property reads should compile through the shared dispatcher");
    assert!(
        outcome
            .note
            .contains("string(;3;42|1|2|3|4|42|42|1|2|3|4|42|42)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_create_realm_uses_realm_local_boolean_constructor_and_prototype() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let value = new other.Boolean(true);
let thisValue = new Boolean(true);
[
  other.Boolean === Boolean,
  other.Boolean.prototype === Boolean.prototype,
  Object.getPrototypeOf(other.Boolean) === other.Function.prototype,
  Object.getPrototypeOf(other.Boolean.prototype) === other.Object.prototype,
  other.Boolean.prototype.constructor === other.Boolean,
  Object.getPrototypeOf(value) === other.Boolean.prototype,
  value instanceof other.Boolean,
  value instanceof Boolean,
  typeof other.Boolean.prototype.toString,
  typeof other.Boolean.prototype.valueOf,
  other.Boolean.prototype.toString.call(thisValue),
  other.Boolean.prototype.valueOf.call(thisValue)
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should use realm-local Boolean constructor and prototype");
    assert!(
        outcome.note.contains(
            "string(false|false|true|true|true|true|true|false|function|function|true|true)"
        ),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_cross_realm_object_value_of_throws_other_realm_type_error() {
    let outcome = engine()
            .run_script(
                r#"
let other = __scriptwCreateRealm();
let method = other.global.Object.prototype.valueOf;
let bound = method.bind(null);
let direct = "missing";
let boundResult = "missing";

try {
  method.call(null);
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
            .expect("borrowed and bound cross-realm Object.prototype.valueOf should throw in defining realm");
    assert!(
        outcome
            .note
            .contains("string(true:true:false|true:true:false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_create_realm_uses_realm_local_string_constructor_and_prototype() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let value = new other.String("abc");
let thisValue = new String("abc");
[
  other.String === String,
  other.String.prototype === String.prototype,
  Object.getPrototypeOf(other.String) === other.Function.prototype,
  Object.getPrototypeOf(other.String.prototype) === other.Object.prototype,
  other.String.prototype.constructor === other.String,
  Object.getPrototypeOf(value) === other.String.prototype,
  value instanceof other.String,
  value instanceof String,
  typeof other.String.prototype.toString,
  typeof other.String.prototype.valueOf,
  typeof other.String.prototype.charAt,
  other.String.prototype.toString.call(thisValue),
  other.String.prototype.valueOf.call(thisValue),
  other.String.prototype.charAt.call(thisValue, 1)
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should use realm-local String constructor and prototype");
    assert!(
            outcome.note.contains(
                "string(false|false|true|true|true|true|true|false|function|function|function|abc|abc|b)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_proxy_get_null_trap_forwards_to_proxy_target_get() {
    let outcome = engine()
            .run_script(
                r#"
let stringTarget = new Proxy(new String("str"), {});
let stringProxy = new Proxy(stringTarget, { get: null });
let sym = Symbol();
let target = new Proxy({}, {
  get: function(_target, key) {
    switch (key) {
      case sym: return 1;
      case "10": return 2;
      case "foo": return 3;
    }
  },
});
let proxy = new Proxy(target, { get: null });
(stringProxy[0] === "s") + ":" + (proxy[sym] === 1) + ":" + (proxy[10] === 2) + ":" + (Object.create(proxy).foo === 3);
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("null proxy get trap should forward to proxy target get");
    assert!(
        outcome.note.contains("string(true:true:true:true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_proxy_get_enforces_non_configurable_target_invariants() {
    for source in [
        "let target = {}; Object.defineProperty(target, 'attr', { configurable: false, writable: false, value: 1 }); let p = new Proxy(target, { get() { return 2; } }); p.attr;",
        "let target = {}; Object.defineProperty(target, 'attr', { configurable: false, get: undefined }); let p = new Proxy(target, { get() { return 2; } }); p.attr;",
    ] {
        let err = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_err();
        assert!(
            err.message().contains("uncaught throw: TypeError"),
            "source: {source}, error: {err:?}"
        );
    }
}

#[test]
fn wasm_backend_supports_object_closure_method() {
    let outcome = engine()
            .run_script(
                "function outer() { let x = 3; return { f: function () { return x; } }; } let o = outer(); o.f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("object closure method should run");
    assert!(outcome.note.contains("number(3"));
}

#[test]
fn wasm_backend_supports_object_literal_shorthand_methods_and_accessors() {
    let shorthand_outcome = engine()
        .run_script(
            "let x = 1; let o = { x }; o.x;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("object shorthand should run");
    assert!(shorthand_outcome.note.contains("number(1"));

    let method_outcome = engine()
        .run_script(
            "let o = { x: 3, f() { return this.x; } }; o.f();",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("concise method should run");
    assert!(method_outcome.note.contains("number(3"));

    let closure_method_outcome = engine()
        .run_script(
            "function make(x) { return { f() { return x; } }; } make(2).f();",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("closure method should run");
    assert!(closure_method_outcome.note.contains("number(2"));

    let getter_outcome = engine()
        .run_script(
            "let o = { get x() { return 1; } }; o.x;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("getter should run");
    assert!(getter_outcome.note.contains("number(1"));

    let setter_outcome = engine()
        .run_script(
            "let o = { _x: 0, set x(v) { this._x = v; } }; o.x = 3; o._x;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("setter should run");
    assert!(setter_outcome.note.contains("number(3"));

    let pair_outcome = engine()
            .run_script(
                "let o = { _x: 0, get x() { return this._x; }, set x(v) { this._x = v; } }; o.x = 4; o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("getter setter pair should run");
    assert!(pair_outcome.note.contains("number(4"));

    let arrow_method_outcome = engine()
        .run_script(
            "let o = { x: 3, f() { return (() => this.x)(); } }; o.f();",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("arrow in method should keep lexical this");
    assert!(arrow_method_outcome.note.contains("number(3"));

    let returned_accessor_outcome = engine()
        .run_script(
            "function make() { return { get x() { return 5; } }; } let o = make(); o.x;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("returned accessor object should run");
    assert!(returned_accessor_outcome.note.contains("number(5"));
}

#[test]
fn wasm_backend_sloppy_assignment_ignores_inherited_accessor_without_setter() {
    let outcome = engine()
        .run_script(
            r#"
let prototype = {};
Object.defineProperty(prototype, "value", {
  get: function() { return 7; }
});
let object = Object.create(prototype);
let before = object.value;
let prototypeMatches = Object.getPrototypeOf(object) === prototype;
object.value = 9;
before + "|" + prototypeMatches + "|" +
  Object.prototype.hasOwnProperty.call(object, "value") + "|" + object.value;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("sloppy assignment should ignore an inherited accessor without a setter");
    assert!(
        outcome.note.contains("string(7|true|false|7)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_strict_assignment_to_inherited_accessor_without_setter_throws() {
    let outcome = engine()
        .run_script(
            r#"
let prototype = {};
Object.defineProperty(prototype, "value", {
  get: function() { return 7; }
});
let object = Object.create(prototype);
let caught = false;
try {
  (function() { "use strict"; object.value = 9; })();
} catch (error) {
  caught = error instanceof TypeError;
}
caught + "|" + Object.prototype.hasOwnProperty.call(object, "value") + "|" + object.value;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("strict assignment to an inherited accessor without a setter should throw");
    assert!(
        outcome.note.contains("string(true|false|7)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_sloppy_assignment_ignores_inherited_read_only_property() {
    let outcome = engine()
        .run_script(
            r#"
let prototype = {};
Object.defineProperty(prototype, "value", { value: 7, writable: false });
let object = Object.create(prototype);
object.value = 9;
Object.prototype.hasOwnProperty.call(object, "value") + "|" + object.value;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("sloppy assignment should ignore an inherited read-only property");
    assert!(
        outcome.note.contains("string(false|7)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_strict_assignment_to_inherited_read_only_property_throws() {
    let outcome = engine()
        .run_script(
            r#"
let prototype = {};
Object.defineProperty(prototype, "value", { value: 7, writable: false });
let object = Object.create(prototype);
let caught = false;
try {
  (function() { "use strict"; object.value = 9; })();
} catch (error) {
  caught = error instanceof TypeError;
}
caught + "|" + Object.prototype.hasOwnProperty.call(object, "value") + "|" + object.value;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("strict assignment to an inherited read-only property should throw");
    assert!(
        outcome.note.contains("string(true|false|7)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_assignment_shadows_inherited_writable_property() {
    let outcome = engine()
        .run_script(
            r#"
let prototype = { value: 7 };
let object = Object.create(prototype);
object.value = 9;
Object.prototype.hasOwnProperty.call(object, "value") + "|" +
  object.value + "|" + prototype.value;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("assignment should shadow an inherited writable property");
    assert!(
        outcome.note.contains("string(true|9|7)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_supports_script_global_object_core() {
    let top_level_this = engine()
        .run_script(
            "this === globalThis;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("top-level this should run");
    assert!(top_level_this.note.contains("boolean(true"));

    let global_var = engine()
        .run_script(
            "{ var x = 1; } globalThis.x;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("global var alias should run");
    assert!(global_var.note.contains("number(1"));

    let lexical_not_global = engine()
        .run_script(
            "let x = 1; globalThis.x;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("top-level lexical should stay off global object");
    assert!(lexical_not_global.note.contains("undefined(undefined"));

    let lexical_not_own_property = engine()
        .run_script(
            "let x = 1; \"x\" in globalThis;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("top-level lexical should not define global object property");
    assert!(lexical_not_own_property.note.contains("boolean(false"));

    let default_this = engine()
        .run_script(
            "function f() { return this; } f() === globalThis;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("bare call default this should run");
    assert!(default_this.note.contains("boolean(true"));

    let lexical_this = engine()
        .run_script(
            "let f = () => this; f() === globalThis;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("top-level arrow lexical this should run");
    assert!(lexical_this.note.contains("boolean(true"));

    let global_function = engine()
        .run_script(
            "function f() {} globalThis.f;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("global function property should run");
    assert!(global_function
        .note
        .contains("wasm-aot completion: function"));
}

#[test]
fn wasm_backend_rejects_remaining_global_object_tails() {
    let err = engine()
        .run_script(
            "arguments",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect_err("unsupported global seam should stay unsupported");
    let message = err.message();
    assert!(
        message.contains("unsupported in scriptw wasm-aot first slice")
            || message.contains("ReferenceError"),
        "err: {message}"
    );
}

#[test]
fn wasm_backend_object_literal_method_forms_follow_the_specification() {
    // Accessor arity is an early error: a getter takes no parameters and a
    // setter exactly one.
    for source in ["({ get x(v) { return v; } })", "({ set x() {} })"] {
        let err = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("invalid accessor arity must be an early error");
        assert!(
            err.message().contains("parse error"),
            "source: {source}, err: {}",
            err.message()
        );
    }

    // `super` in an object-literal method used to be locked as
    // unsupported. It now compiles; the method's [[HomeObject]] is the
    // literal, so `super.x` reads from the literal's current prototype
    // with the caller's `this`, survives being detached, reads through a
    // null prototype as a TypeError, and `super.y = v` writes to `this`.
    let source = r#"
var obj = { f() { return super.x; } };
var results = [obj.f() === undefined];
Object.setPrototypeOf(obj, { x: 1 });
results.push(obj.f() === 1);
var withGetter = { tag: "receiver", g() { return super.x; } };
Object.setPrototypeOf(withGetter, { get x() { return this.tag; } });
results.push(withGetter.g() === "receiver");
results.push({ tag: "detached", g: withGetter.g }.g() === "detached");
Object.setPrototypeOf(withGetter, null);
try { withGetter.g(); results.push(false); }
catch (error) { results.push(error.constructor === TypeError); }
var parent = {};
var writer = { m() { super.y = 5; return this.y; } };
Object.setPrototypeOf(writer, parent);
results.push(writer.m() === 5 && Object.getOwnPropertyNames(parent).length === 0);
results.join();
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
        .unwrap_or_else(|err| panic!("object-literal super should run: {err:?}"));
    assert!(
        outcome
            .note
            .contains("string(true,true,true,true,true,true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_object_literal_spread_copies_own_enumerable_properties_in_order() {
    let source = r#"
var calls = [];
var copiedSymbol = Symbol("copied");
var source = {};
Object.defineProperty(source, "visible", {
  get: function() { calls.push("getter:visible"); return 2; },
  enumerable: true
});
Object.defineProperty(source, "hidden", { value: 4, enumerable: false });
source[copiedSymbol] = 3;
var proxy = new Proxy(source, {
  ownKeys: function(target) {
    calls.push("ownKeys");
    return ["visible", "hidden", copiedSymbol];
  },
  getOwnPropertyDescriptor: function(target, key) {
    calls.push(key === copiedSymbol ? "descriptor:symbol" : "descriptor:" + key);
    return Reflect.getOwnPropertyDescriptor(target, key);
  },
  get: function(target, key) {
    calls.push(key === copiedSymbol ? "get:symbol" : "get:" + key);
    return Reflect.get(target, key);
  }
});
function spreadSource(label, value) {
  calls.push("source:" + label);
  return value;
}
var result = {
  before: 1,
  ...spreadSource("null", null),
  ...spreadSource("proxy", proxy),
  visible: 5,
  ...spreadSource("string", "xy"),
  ...undefined
};
var ordinaryCalls = [];
var ordinarySymbol = Symbol("ordinary");
var ordinary = {
  get z() { ordinaryCalls.push("z"); },
  get a() { ordinaryCalls.push("a"); }
};
Object.defineProperty(ordinary, "1", {
  get: function() { ordinaryCalls.push("1"); },
  enumerable: true
});
Object.defineProperty(ordinary, ordinarySymbol, {
  get: function() { ordinaryCalls.push("symbol"); return 8; },
  enumerable: true
});
var ordinaryResult = { ...ordinary };
var descriptor = Object.getOwnPropertyDescriptor(result, "visible");
var setterOverwritten = Object.getOwnPropertyDescriptor(
  { set value(next) {}, ...{ value: 7 } },
  "value"
);
var marker = {};
var abrupt = false;
try {
  ({ ...new Proxy({}, { ownKeys: function() { throw marker; } }) });
} catch (error) {
  abrupt = error === marker;
}
[
  calls.join(","),
  result.before,
  result.visible,
  result[copiedSymbol],
  result.hidden === undefined,
  result[0] + result[1],
  ordinaryCalls.join(","),
  ordinaryResult[ordinarySymbol] === 8,
  descriptor.value === 5 && descriptor.writable && descriptor.enumerable && descriptor.configurable,
  setterOverwritten.value === 7 && setterOverwritten.writable && setterOverwritten.enumerable && setterOverwritten.configurable,
  abrupt
].join("|");
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
        .expect("object literal spreads should run through CopyDataProperties");

    assert!(
            outcome.note.contains(
                "string(source:null,source:proxy,ownKeys,descriptor:visible,get:visible,getter:visible,descriptor:hidden,descriptor:symbol,get:symbol,source:string|1|5|3|true|xy|1,z,a,symbol|true|true|true|true)"
            ),
            "unexpected object literal spread result: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_ordinary_own_keys_orders_array_indices_before_strings_and_symbols() {
    let source = r#"
var symbol = Symbol("last");
var object = {};
object.z = 1;
object["4294967295"] = 2;
object["4294967294"] = 3;
object["91"] = 4;
object["19"] = 5;
object["2"] = 6;
object["01"] = 7;
object.a = 8;
object[symbol] = 9;
var names = Object.getOwnPropertyNames(object);
var keys = Reflect.ownKeys(object);
var proxyKeys = Reflect.ownKeys(new Proxy(object, {
  ownKeys: function() {
    return [symbol, "a", "01", "4294967295", "z", "4294967294", "91", "19", "2"];
  }
}));
names.join(",") + "|" + keys.length + "|" + (keys[8] === symbol) + "|" +
  (proxyKeys[0] === symbol) + "," + proxyKeys.slice(1).join(",");
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
        .expect("ordinary own-key order should run");

    assert!(
            outcome.note.contains(
                "string(2,19,91,4294967294,z,4294967295,01,a|9|true|true,a,01,4294967295,z,4294967294,91,19,2)"
            ),
            "unexpected ordinary own-key order: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_proxy_own_keys_validates_symbols_before_public_filtering() {
    let source = r#"
var symbol = Symbol("invariant");
var failures = 0;
function expectsTypeError(callback) {
  try {
    callback();
  } catch (error) {
    if (error instanceof TypeError) failures += 1;
  }
}

var nonConfigurableTarget = {};
Object.defineProperty(nonConfigurableTarget, symbol, { configurable: false });
var omitsNonConfigurable = new Proxy(nonConfigurableTarget, {
  ownKeys: function() { return []; }
});
expectsTypeError(function() { Object.getOwnPropertyNames(omitsNonConfigurable); });

var duplicatesSymbol = new Proxy({}, {
  ownKeys: function() { return [symbol, symbol]; }
});
expectsTypeError(function() { Object.getOwnPropertyNames(duplicatesSymbol); });
expectsTypeError(function() { Reflect.ownKeys(duplicatesSymbol); });

var nonExtensibleTarget = {};
nonExtensibleTarget[symbol] = 1;
Object.preventExtensions(nonExtensibleTarget);
var omitsNonExtensible = new Proxy(nonExtensibleTarget, {
  ownKeys: function() { return []; }
});
expectsTypeError(function() { Object.getOwnPropertySymbols(omitsNonExtensible); });

var emptyNonExtensibleTarget = Object.preventExtensions({});
var addsToNonExtensible = new Proxy(emptyNonExtensibleTarget, {
  ownKeys: function() { return [symbol]; }
});
expectsTypeError(function() { Object.getOwnPropertyNames(addsToNonExtensible); });
expectsTypeError(function() { Reflect.ownKeys(addsToNonExtensible); });

var nonExtensibleArray = [1];
Object.preventExtensions(nonExtensibleArray);
var omitsArrayIndex = new Proxy(nonExtensibleArray, {
  ownKeys: function() { return ["length"]; }
});
expectsTypeError(function() { Reflect.ownKeys(omitsArrayIndex); });

var emptyNonExtensibleArray = Object.preventExtensions([]);
var addsToNonExtensibleArray = new Proxy(emptyNonExtensibleArray, {
  ownKeys: function() { return ["length", "extra"]; }
});
expectsTypeError(function() { Object.getOwnPropertyNames(addsToNonExtensibleArray); });

failures;
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
        .expect("proxy ownKeys symbol invariants should run");

    assert!(
        outcome.note.contains("number(8)"),
        "unexpected proxy ownKeys invariant result: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_proxy_own_keys_coerces_array_like_length_and_propagates_errors() {
    let source = r#"
var arrayLikeKeys = new Proxy({}, {
  ownKeys: function() { return { 0: "arrayLike", length: "1" }; }
});
var keys = Reflect.ownKeys(arrayLikeKeys);
var lengthError = {};
var observedLengthError = false;
var throwingLength = new Proxy({}, {
  ownKeys: function() {
    return Object.defineProperty({}, "length", {
      get: function() { throw lengthError; }
    });
  }
});
try {
  Reflect.ownKeys(throwingLength);
} catch (error) {
  observedLengthError = error === lengthError;
}
[keys.length, keys[0], observedLengthError].join(",");
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
        .expect("proxy ownKeys array-like length should run");

    assert!(
        outcome.note.contains("string(1,arrayLike,true)"),
        "unexpected proxy ownKeys array-like result: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_own_key_methods_accept_callable_proxy_traps() {
    let outcome = engine()
        .run_script(
            r#"
var ownKeysCalls = 0;
var ownKeysTrap = new Proxy(function() {
  ownKeysCalls++;
  return ["visible", "hidden"];
}, {});
var descriptorTrap = new Proxy(function(target, key) {
  return {
    value: target[key],
    writable: true,
    enumerable: key === "visible",
    configurable: true
  };
}, {});
var descriptorTrap = new Proxy(function(target, key) {
  return Reflect.getOwnPropertyDescriptor(target, key);
}, {});
var target = {};
target.visible = 1;
Object.defineProperty(target, "hidden", { value: 2, enumerable: false });
var proxy = new Proxy(target, {
  ownKeys: ownKeysTrap,
  getOwnPropertyDescriptor: descriptorTrap
});

[
  Reflect.ownKeys(proxy).join(","),
  Object.getOwnPropertyNames(proxy).join(","),
  Object.getOwnPropertySymbols(proxy).length,
  Object.keys(proxy).join(","),
  ownKeysCalls
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("own-key methods should call callable proxy traps");
    assert!(
        outcome
            .note
            .contains("string(visible,hidden|visible,hidden|0|visible|4)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_own_key_methods_cross_deep_trapless_proxy_chains() {
    let outcome = engine()
        .run_script(
            r#"
function wrapDeeply(target) {
  for (var depth = 0; depth < 12; depth++) {
    target = new Proxy(target, {});
  }
  return target;
}

var symbol = Symbol("owned");
var target = { visible: 1 };
target[symbol] = 2;
var proxy = wrapDeeply(target);
var reflected = Reflect.ownKeys(proxy);

[
  reflected.length,
  reflected[0],
  reflected[1] === symbol,
  Object.getOwnPropertyNames(proxy).join(","),
  Object.getOwnPropertySymbols(proxy)[0] === symbol,
  Object.keys(proxy).join(",")
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("own-key methods should cross deep trapless proxy chains");
    assert!(
        outcome
            .note
            .contains("string(2|visible|true|visible|true|visible)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_preserves_array_subclass_prototype_chain() {
    let outcome = engine()
        .run_script(
            r#"
class Sub extends Array {}
let array = new Sub(42, "foo");
let pushedLength = array.push(true);
let empty = new Sub();
let sized = new Sub(7);
[
  Object.getPrototypeOf(array) === Sub.prototype,
  Object.getPrototypeOf(Sub.prototype) === Array.prototype,
  array.constructor === Sub,
  array[0],
  array[1],
  pushedLength,
  array.length,
  array[2],
  empty.length,
  sized.length,
  array instanceof Sub,
  array instanceof Array,
  Sub[Symbol.species] === Sub
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array subclasses should preserve their exotic prototype chain");
    assert!(
        outcome
            .note
            .contains("string(true|true|true|42|foo|3|3|true|0|7|true|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_emits_arraybuffer_bytelength_getter_for_property_access() {
    let outcome = engine()
        .run_script(
            "new ArrayBuffer(42).byteLength;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("ArrayBuffer byteLength property access should run: {err:?}"));
    assert!(
        outcome.note.contains("number(42)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_validates_slice_named_function_property_reads() {
    let source = "let o = { slice() { return 1; } }; typeof o.slice === 'function' && ArrayBuffer.prototype.slice.length === 2;";
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
            panic!("slice-named function property reads should validate and run: {err:?}")
        });
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_to_string_requires_an_object_coercible_receiver() {
    let source = "var arrayMethod = Array.prototype.toString; var typedArrayMethod = Uint8Array.prototype.toString; var arrayThrows = false; var typedArrayThrows = false; try { arrayMethod(); } catch (error) { arrayThrows = error instanceof TypeError; } try { typedArrayMethod(); } catch (error) { typedArrayThrows = error instanceof TypeError; } arrayThrows + '|' + typedArrayThrows;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array toString methods should reject nullish receivers");
    assert!(
        outcome.note.contains("string(true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_to_string_calls_proxy_join_with_the_original_receiver() {
    let source = "const method = Uint8Array.prototype.toString; const events = []; let receiver; const join = new Proxy(function() {}, { apply(target, thisArgument) { events.push(thisArgument === receiver ? 'call' : 'wrong receiver'); return 'proxied'; } }); receiver = new Proxy({}, { get(target, key) { events.push('get:' + key); if (key === 'join') return join; } }); method.call(receiver) + '|' + events.join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.toString should call a Proxy join method");
    assert!(
        outcome.note.contains("string(proxied|get:join,call)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_from_snapshots_proxy_iterators_before_mapping() {
    let source = "let nextCalls = 0; const iterator = { next: new Proxy(function() { nextCalls++; return nextCalls <= 3 ? { value: nextCalls } : { done: true }; }, {}) }; const source = { [Symbol.iterator]: new Proxy(function() { return iterator; }, {}) }; const receiver = { factor: 2 }; const mapper = new Proxy(function(value, index) { return value * this.factor + index; }, {}); const Constructor = new Proxy(function(length) { return new Uint8Array(length); }, {}); const result = Uint8Array.from.call(Constructor, source, mapper, receiver); result.join(',') + '|' + result.length + '|' + nextCalls;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray.from should consume proxy iterators before mapping");
    assert!(
        outcome.note.contains("string(2,5,8|3|4)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_accessors_use_internal_resizable_view_state() {
    let source = "const prototype = Object.getPrototypeOf(Uint8Array.prototype); const names = ['buffer', 'byteLength', 'byteOffset', 'length']; const getters = names.map(name => Object.getOwnPropertyDescriptor(prototype, name).get); const rab = new ArrayBuffer(8, { maxByteLength: 16 }); const fixed = new Uint16Array(rab, 2, 2); const tracking = new Uint16Array(rab, 2); fixed.$TypedArrayByteLength = 100; fixed.$TypedArrayByteOffset = 100; function state(value) { return (getters[0].call(value) === rab) + ':' + getters[1].call(value) + ':' + getters[2].call(value) + ':' + getters[3].call(value); } const initial = state(fixed) + '|' + state(tracking); rab.resize(3); const shrunk = state(fixed) + '|' + state(tracking); rab.resize(8); const restored = state(fixed) + '|' + state(tracking); const forged = { $TypedArrayViewedArrayBuffer: rab, $TypedArrayByteLength: 4, $TypedArrayByteOffset: 0, $TypedArrayBytesPerElement: 1 }; let forgedErrors = 0; let proxyErrors = 0; for (const getter of getters) { try { getter.call(forged); } catch (error) { if (error.name === 'TypeError') forgedErrors++; } try { getter.call(new Proxy(fixed, {})); } catch (error) { if (error.name === 'TypeError') proxyErrors++; } } initial + '|' + shrunk + '|' + restored + '|' + forgedErrors + '|' + proxyErrors;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray accessors should use internal resizable view state");
    assert!(
        outcome.note.contains(
            "string(true:4:2:2|true:6:2:3|true:0:0:0|true:0:2:0|true:4:2:2|true:6:2:3|4|4)"
        ),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_accessor_redefinition_preserves_omitted_flags() {
    let source = "var array = [0, 1, 2]; Object.defineProperty(array, '1', { get: function() { return 1; } }); var descriptor = Object.getOwnPropertyDescriptor(array, '1'); array.length = 1; descriptor.enumerable + '|' + descriptor.configurable + '|' + array.length;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("array accessor redefinition should preserve existing index flags");
    assert!(
        outcome.note.contains("string(true|true|1)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_define_property_writes_typed_array_elements() {
    let source = r#"
const values = new Uint32Array([0]);
const objectResult = Object.defineProperty(values, "0", { value: 4294967295 });
const reflectResult = Reflect.defineProperty(values, "0", { value: 4294967296 });
let incompatibleThrew = false;
let incompatibleConverted = false;
try {
  Object.defineProperty(values, "0", {
    value: { valueOf() { incompatibleConverted = true; return 1; } },
    writable: false
  });
} catch (error) {
  incompatibleThrew = error instanceof TypeError;
}
let conversionError = "none";
try {
  Object.defineProperty(values, "0", {
    value: { valueOf() { throw new RangeError("conversion"); } }
  });
} catch (error) {
  conversionError = error.name;
}
(objectResult === values) + "|" + reflectResult + "|" + values[0] + "|" +
  incompatibleThrew + "|" + incompatibleConverted + "|" + conversionError;
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
        .expect("defineProperty should use TypedArray integer-indexed semantics");
    assert!(
        outcome
            .note
            .contains("string(true|true|0|true|false|RangeError)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_dataview_accessors_and_bigint_getters_require_private_brand() {
    let source = r#"
const accessors = ["buffer", "byteLength", "byteOffset"].map(
  name => Object.getOwnPropertyDescriptor(DataView.prototype, name).get
);
const forgedBuffer = new ArrayBuffer(8);
const forged = {
  buffer: forgedBuffer,
  $DataViewDataPtr: 1,
  $DataViewByteOffset: 0,
  $DataViewByteLength: 8,
  $DataViewLengthTracking: false
};
let accessorErrors = 0;
for (const accessor of accessors) {
  try {
    accessor.call(forged);
  } catch (error) {
    if (error.name === "TypeError") accessorErrors++;
  }
}
let offsetCoercions = 0;
let getterError = "missing";
try {
  DataView.prototype.getBigInt64.call(forged, {
    valueOf() {
      offsetCoercions++;
      return 0;
    }
  });
} catch (error) {
  getterError = error.name;
}
accessorErrors + "|" + getterError + "|" + offsetCoercions;
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
        .expect("DataView accessors and BigInt getters should require the private brand");
    assert!(
        outcome.note.contains("string(3|TypeError|0)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_dataview_accessors_and_bigint_getters_ignore_legacy_mirrors() {
    let source = r#"
const buffer = new ArrayBuffer(10, { maxByteLength: 16 });
const view = new DataView(buffer, 2);
const replacement = new ArrayBuffer(8);
Object.defineProperty(view, "buffer", { value: replacement, writable: true });
view.$DataViewDataPtr = 0;
view.$DataViewByteOffset = 0;
view.$DataViewByteLength = 1;
view.$DataViewLengthTracking = false;
const bufferGetter = Object.getOwnPropertyDescriptor(DataView.prototype, "buffer").get;
const byteOffsetGetter = Object.getOwnPropertyDescriptor(DataView.prototype, "byteOffset").get;
const byteLengthGetter = Object.getOwnPropertyDescriptor(DataView.prototype, "byteLength").get;
const initial = (bufferGetter.call(view) === buffer) + ":" +
  byteOffsetGetter.call(view) + ":" + byteLengthGetter.call(view);
buffer.resize(14);
const grown = byteOffsetGetter.call(view) + ":" +
  byteLengthGetter.call(view) + ":" + (view.getBigUint64(4) === 0n);
const fixedBuffer = new ArrayBuffer(8, { maxByteLength: 16 });
const fixed = new DataView(fixedBuffer, 0, 8);
fixed.$DataViewByteLength = 0;
fixed.$DataViewLengthTracking = true;
fixedBuffer.resize(4);
const offsetError = {};
let abruptOffset = false;
try {
  fixed.getBigInt64({
    valueOf() {
      throw offsetError;
    }
  });
} catch (error) {
  abruptOffset = error === offsetError;
}
let fixedError = "missing";
try {
  fixed.getBigInt64(0);
} catch (error) {
  fixedError = error.name;
}
initial + "|" + grown + "|" + fixedError + ":" + abruptOffset;
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
        .expect("DataView private state should ignore poisoned legacy mirrors");
    assert!(
        outcome
            .note
            .contains("string(true:2:8|2:12:true|TypeError:true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_preserves_global_object_method_return_after_loop_assignment() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
var assert = {
  sameValue: function(actual, expected) {
    return actual === expected;
  }
};
function waitUntil(expected) {
  var actual = 0;
  while ((actual = expected) !== expected) {
  }
  return assert.sameValue(actual, expected);
}
print("wait-until-method:" + waitUntil(3));
"#;
    let outcome = engine_with_captured_prints(Arc::clone(&lines))
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("global object method calls should preserve their result after loop assignment");
    assert!(outcome.note.contains("undefined"), "note: {}", outcome.note);
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["wait-until-method:true".to_string()]
    );
}

#[test]
fn wasm_backend_array_prototype_exposes_standard_unscopables() {
    let source = "const value = Array.prototype[Symbol.unscopables]; const descriptor = Object.getOwnPropertyDescriptor(Array.prototype, Symbol.unscopables); Object.getPrototypeOf(value) === null && descriptor.value === value && descriptor.writable === false && descriptor.enumerable === false && descriptor.configurable === true && value.copyWithin === true && value.findLast === true && value.toSpliced === true && !Object.prototype.hasOwnProperty.call(value, 'with');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype should expose the standard unscopables object");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_map_rechecks_prototype_holes_after_getter_side_effects() {
    let source = "function callbackfn(val, idx) { return idx === 1 && val === 6.99 ? false : true; } var arr = [0, , 2]; Object.defineProperty(arr, '0', { get: function() { Object.defineProperty(Array.prototype, '1', { get: function() { return 6.99; }, configurable: true }); return 0; }, configurable: true }); var r = arr.map(callbackfn); delete Array.prototype[1]; r[0] + '|' + r[1] + '|' + r[2] + '|' + Object.keys(r).join(',');";
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
            panic!("Array.prototype.map should re-check prototype holes: {err:?}")
        });
    assert!(
        outcome.note.contains("string(true|false|true|0,1,2)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_supports_phase_thirty_one_function_prototype_tostring() {
    for (source, expected) in [
        (
            "function f(x) { return x; } f.toString();",
            "string(function f(x) { return x; })",
        ),
        ("let f = x => x + 1; f.toString();", "string(x => x + 1)"),
        (
            "let o = { m(x) { return x; } }; o.m.toString();",
            "string(m(x) { return x; })",
        ),
        (
            "class C { constructor(x) { this.x = x; } } C.toString();",
            "string(class C { constructor(x) { this.x = x; } })",
        ),
        (
            "class C { m() { return 1; } } new C().m.toString();",
            "string(m() { return 1; })",
        ),
        (
            "let g = function f(x) { return x; }.bind(null, 1); g.toString();",
            "string(function () { [native code] })",
        ),
        (
            "Function.prototype.toString.call(Function.prototype.call);",
            "string(function call() { [native code] })",
        ),
        (
            "print.toString();",
            "string(function print() { [native code] })",
        ),
        (
            "try { Function.prototype.toString.call({}); } catch (e) { e instanceof TypeError; }",
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
                panic!("phase 31 function toString case should run for `{source}`: {err:?}")
            });
        assert!(
            outcome.note.contains(expected),
            "source: {source}, note: {}",
            outcome.note
        );
    }
}

#[test]
fn wasm_backend_collection_species_getters_have_standard_descriptors_and_return_receivers() {
    let source = r#"
let mapDescriptor = Object.getOwnPropertyDescriptor(Map, Symbol.species);
let setDescriptor = Object.getOwnPropertyDescriptor(Set, Symbol.species);
let mapReceiver = {};
let setReceiver = function () {};
[
  Map[Symbol.species] === Map,
  Set[Symbol.species] === Set,
  mapDescriptor.get !== setDescriptor.get,
  mapDescriptor.get.call(mapReceiver) === mapReceiver,
  setDescriptor.get.call(setReceiver) === setReceiver,
  mapDescriptor.get.call(undefined) === undefined,
  setDescriptor.get.call(42) === 42,
  mapDescriptor.set === undefined,
  setDescriptor.set === undefined,
  mapDescriptor.enumerable,
  setDescriptor.enumerable,
  mapDescriptor.configurable,
  setDescriptor.configurable,
  mapDescriptor.get.name,
  setDescriptor.get.name,
  mapDescriptor.get.length,
  setDescriptor.get.length
].join("|");
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
        .expect("Map and Set species getters should expose their standard observable surface");
    assert!(
            outcome.note.contains(
                "string(true|true|true|true|true|true|true|true|true|false|false|true|true|get [Symbol.species]|get [Symbol.species]|0|0)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_object_group_by_preserves_property_keys_and_callback_contract() {
    let outcome = engine()
        .run_script(
            r#"
var iteratorGets = 0;
var invalidCallbackThrows = false;
var invalidCallbackIterable = {};
Object.defineProperty(invalidCallbackIterable, Symbol.iterator, {
  get: function() {
    iteratorGets += 1;
    throw new Error("iterator must not be read");
  }
});
try {
  Object.groupBy(invalidCallbackIterable, null);
} catch (error) {
  invalidCallbackThrows = error instanceof TypeError;
}
var symbolKey = Symbol("group");
var sharedKey = { toString: function() { return "1"; } };
var values = ["symbol", "proto", "number", "object"];
var nextGets = 0;
var index = 0;
var iterable = {
  [Symbol.iterator]: new Proxy(function() {
    return {
      get next() {
        nextGets += 1;
        return new Proxy(function() {
          if (index === values.length) return { done: true };
          return { done: false, value: values[index++] };
        }, {});
      }
    };
  }, {})
};
var callbackThis = 1;
var callback = new Proxy(function(value, callbackIndex) {
  "use strict";
  callbackThis = this;
  if (callbackIndex === 0) return symbolKey;
  if (callbackIndex === 1) return "__proto__";
  if (callbackIndex === 2) return 1;
  return sharedKey;
}, {});
var grouped = Object.groupBy.call(function NotObject() {}, iterable, callback);
var keys = Object.keys(grouped);
[
  typeof Object.groupBy,
  Object.groupBy.name,
  Object.groupBy.length,
  invalidCallbackThrows,
  iteratorGets,
  Object.getPrototypeOf(grouped) === null,
  callbackThis === undefined,
  nextGets,
  grouped[symbolKey][0],
  Object.getOwnPropertySymbols(grouped)[0] === symbolKey,
  grouped.hasOwnProperty === undefined,
  Object.prototype.hasOwnProperty.call(grouped, "__proto__"),
  grouped.__proto__[0],
  grouped[1].join(","),
  keys.join(",")
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.groupBy should preserve property keys and iterator callback contracts");
    assert!(
            outcome.note.contains(
                "string(function|groupBy|2|true|0|true|true|1|symbol|true|true|true|proto|number,object|1,__proto__)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_object_group_by_closes_for_callback_and_property_key_abrupt_completion() {
    let outcome = engine()
        .run_script(
            r#"
function CallbackError() {}
function KeyError() {}
function NextError() {}
function DoneError() {}
function ValueError() {}
var closeCount = 0;
function iterableWithNext(next) {
  return {
    [Symbol.iterator]: function() {
      return {
        next: next,
        return: function() {
          closeCount += 1;
          throw new TypeError("suppressed close failure");
        }
      };
    }
  };
}
var callbackError = new CallbackError();
var callbackIdentity = false;
try {
  Object.groupBy(iterableWithNext(function() {
    return { done: false, value: 1 };
  }), function() { throw callbackError; });
} catch (error) {
  callbackIdentity = error === callbackError;
}
var keyError = new KeyError();
var keyIdentity = false;
try {
  Object.groupBy(iterableWithNext(function() {
    return { done: false, value: 1 };
  }), function() {
    return { toString: function() { throw keyError; } };
  });
} catch (error) {
  keyIdentity = error === keyError;
}
var nextError = new NextError();
var nextIdentity = false;
try {
  Object.groupBy(iterableWithNext(function() { throw nextError; }), function() {});
} catch (error) {
  nextIdentity = error === nextError;
}
var doneError = new DoneError();
var doneIdentity = false;
try {
  Object.groupBy(iterableWithNext(function() {
    return { get done() { throw doneError; } };
  }), function() {});
} catch (error) {
  doneIdentity = error === doneError;
}
var valueError = new ValueError();
var valueIdentity = false;
try {
  Object.groupBy(iterableWithNext(function() {
    return { done: false, get value() { throw valueError; } };
  }), function() {});
} catch (error) {
  valueIdentity = error === valueError;
}
[callbackIdentity, keyIdentity, nextIdentity, doneIdentity, valueIdentity, closeCount].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect(
            "Object.groupBy should close only after callback or property-key abrupt completion",
        );
    assert!(
        outcome.note.contains("string(true|true|true|true|true|2)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_object_group_by_uses_the_method_defining_realm() {
    let outcome = engine()
        .run_script(
            r#"
var other = __scriptwCreateRealm().global;
var grouped = other.Object.groupBy([1, 2, 3], function(value) { return value % 2; });
var group = grouped[1];
var realmError = false;
try {
  other.Object.groupBy([], null);
} catch (error) {
  realmError = error instanceof other.TypeError && !(error instanceof TypeError);
}
[
  other.Object.groupBy !== Object.groupBy,
  Object.getPrototypeOf(grouped) === null,
  group instanceof other.Array,
  group instanceof Array,
  group[0] === 1 && group[1] === 3,
  realmError
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Object.groupBy should allocate groups and errors from its defining realm");
    assert!(
        outcome
            .note
            .contains("string(true|true|true|false|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_map_accepts_callable_proxy_iterator_components() {
    let outcome = engine()
        .run_script(
            r#"
var intrinsicSet = Map.prototype.set;
var setCalls = 0;
var proxiedSet = new Proxy(function(key, value) {
  setCalls += 1;
  return intrinsicSet.call(this, key, value);
}, {});
class DerivedMap extends Map {}
DerivedMap.prototype.set = proxiedSet;
var index = 0;
var proxiedNext = new Proxy(function() {
  if (index === 2) return { done: true };
  index += 1;
  return { done: false, value: new Proxy([index, index * 10], {}) };
}, {});
var proxiedIterator = new Proxy(function() {
  return { next: proxiedNext };
}, {});
var iterable = { [Symbol.iterator]: proxiedIterator };
var map = new DerivedMap(iterable);
[map.size, map.get(1), map.get(2), setCalls].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Map should call Proxy-wrapped iterator components and setter");
    assert!(
        outcome.note.contains("string(2|10|20|2)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_map_uses_cross_realm_new_target_prototype_fallback() {
    let outcome = engine()
        .run_script(
            r#"
let otherRealm = __scriptwCreateRealm();
let other = otherRealm.global;
// An ordinary constructor admits a non-object prototype without mutating an intrinsic.
let otherConstructor = otherRealm.evalScript("function Other() {} Other;");
otherConstructor.prototype = null;
if (otherConstructor.prototype !== null) throw new Error("fallback precondition");
let direct = Reflect.construct(Map, [], otherConstructor);
let proxied = Reflect.construct(Map, [], new Proxy(otherConstructor, {}));
let boundConstructed = Reflect.construct(Map, [], otherConstructor.bind(null));
let revocable;
revocable = Proxy.revocable(otherConstructor, {
  get(target, key, receiver) {
    if (key === "prototype") {
      revocable.revoke();
      return null;
    }
    return Reflect.get(target, key, receiver);
  }
});
let revokedThrows = false;
try {
  Reflect.construct(Map, [], revocable.proxy);
} catch (error) {
  revokedThrows = error instanceof TypeError;
}
[
  other.Map.prototype !== Map.prototype,
  Object.getPrototypeOf(direct) === other.Map.prototype,
  Object.getPrototypeOf(proxied) === other.Map.prototype,
  Object.getPrototypeOf(boundConstructed) === other.Map.prototype,
  Object.getPrototypeOf(direct) !== Map.prototype,
  revokedThrows
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Map construction should use the original newTarget function realm");
    assert!(
        outcome
            .note
            .contains("string(true|true|true|true|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_set_accepts_callable_proxy_iterator_components() {
    let outcome = engine()
        .run_script(
            r#"
var intrinsicAdd = Set.prototype.add;
var addCalls = 0;
var proxiedAdd = new Proxy(function(value) {
  addCalls += 1;
  return intrinsicAdd.call(this, value);
}, {});
class DerivedSet extends Set {}
DerivedSet.prototype.add = proxiedAdd;
var index = 0;
var proxiedNext = new Proxy(function() {
  if (index === 2) return { done: true };
  index += 1;
  return { done: false, value: index };
}, {});
var proxiedIterator = new Proxy(function() {
  return { next: proxiedNext };
}, {});
var iterable = { [Symbol.iterator]: proxiedIterator };
var set = new DerivedSet(iterable);
[set.size, set.has(1), set.has(2), addCalls].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Set should call Proxy-wrapped iterator components");
    assert!(
        outcome.note.contains("string(2|true|true|2)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_set_uses_cross_realm_new_target_prototype_fallback() {
    let outcome = engine()
        .run_script(
            r#"
let otherRealm = __scriptwCreateRealm();
let other = otherRealm.global;
// An ordinary constructor admits a non-object prototype without mutating an intrinsic.
let otherConstructor = otherRealm.evalScript("function Other() {} Other;");
otherConstructor.prototype = null;
if (otherConstructor.prototype !== null) throw new Error("fallback precondition");
let direct = Reflect.construct(Set, [], otherConstructor);
let proxied = Reflect.construct(Set, [], new Proxy(otherConstructor, {}));
let boundConstructed = Reflect.construct(Set, [], otherConstructor.bind(null));
let revocable;
revocable = Proxy.revocable(otherConstructor, {
  get(target, key, receiver) {
    if (key === "prototype") {
      revocable.revoke();
      return null;
    }
    return Reflect.get(target, key, receiver);
  }
});
let revokedThrows = false;
try {
  Reflect.construct(Set, [], revocable.proxy);
} catch (error) {
  revokedThrows = error instanceof TypeError;
}
[
  other.Set.prototype !== Set.prototype,
  Object.getPrototypeOf(direct) === other.Set.prototype,
  Object.getPrototypeOf(proxied) === other.Set.prototype,
  Object.getPrototypeOf(boundConstructed) === other.Set.prototype,
  Object.getPrototypeOf(direct) !== Set.prototype,
  revokedThrows
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Set construction should use the original newTarget function realm");
    assert!(
        outcome
            .note
            .contains("string(true|true|true|true|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_async_object_method_preserves_receiver_and_is_not_constructable() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            let holder = {
                marker: 40,
                async method(delta) {
                    await Promise.resolve();
                    return this.marker + delta;
                }
            };
            print(holder.method.name);
            let returned = holder.method(2);
            print(returned instanceof Promise);
            try {
                new holder.method(1);
            } catch (error) {
                print(error instanceof TypeError);
            }
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
        .unwrap_or_else(|err| panic!("async object method should execute: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &[
            "method".to_string(),
            "true".to_string(),
            "true".to_string(),
            "sync".to_string(),
            "fulfilled:42".to_string(),
        ]
    );
}

#[test]
fn wasm_backend_async_object_method_rejects_later_parameter_tdz_read() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            let holder = { async method(value = later, later) {} };
            holder.method().then(
                undefined,
                function (error) { print(error instanceof ReferenceError); }
            );
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
        .unwrap_or_else(|err| panic!("async parameter TDZ should reject: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["sync".to_string(), "true".to_string()]
    );
}

#[test]
fn wasm_backend_async_generator_object_and_class_methods_use_real_activations() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            function delegate(prefix) {
                return {
                    [Symbol.asyncIterator]: function () { return this; },
                    next: function () {
                        return { value: prefix + ":open", done: false };
                    },
                    return: function (value) {
                        return { value: prefix + ":" + value, done: true };
                    }
                };
            }
            const object = {
                marker: "object",
                async *method() { yield* delegate(this.marker); }
            };
            class Example {
                constructor() { this.marker = "instance"; }
                async *method() { yield* delegate(this.marker); }
                static async *staticMethod() { yield* delegate(this.marker); }
            }
            Example.marker = "static";

            function observe(label, iterator) {
                return iterator.next().then(function (result) {
                    print(label + ":" + result.value + ":" + result.done);
                    return iterator.return("closed");
                }).then(function (result) {
                    print(label + ":" + result.value + ":" + result.done);
                });
            }

            const objectIterator = object.method();
            const instance = new Example();
            const instanceIterator = instance.method();
            const staticIterator = Example.staticMethod();
            print(Object.getPrototypeOf(objectIterator) === object.method.prototype);
            print(Object.getPrototypeOf(instanceIterator) === instance.method.prototype);
            print(Object.getPrototypeOf(staticIterator) === Example.staticMethod.prototype);
            observe("object", objectIterator)
                .then(function () { return observe("instance", instanceIterator); })
                .then(function () { return observe("static", staticIterator); });
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
        .unwrap_or_else(|err| panic!("async-generator methods should execute: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &[
            "true".to_string(),
            "true".to_string(),
            "true".to_string(),
            "object:object:open:false".to_string(),
            "object:object:closed:true".to_string(),
            "instance:instance:open:false".to_string(),
            "instance:instance:closed:true".to_string(),
            "static:static:open:false".to_string(),
            "static:static:closed:true".to_string(),
        ]
    );
}

#[test]
fn wasm_backend_for_await_sync_iterator_rejects_abrupt_and_non_object_results() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            let nextError = {};
            let doneError = {};
            let valueError = {};
            let closeCalls = 0;
            function iterableWithNext(next) {
                let iterable = {};
                iterable[Symbol.iterator] = function () {
                    return {
                        next: next,
                        return: function () {
                            closeCalls += 1;
                            return {};
                        }
                    };
                };
                return iterable;
            }
            async function catches(iterable, expected, expectsTypeError) {
                try {
                    for await (const value of iterable) {}
                } catch (error) {
                    return expectsTypeError ? error instanceof TypeError : error === expected;
                }
                return false;
            }
            let settled = 0;
            let passed = 0;
            function record(result) {
                if (result) passed += 1;
                settled += 1;
                if (settled === 4) print(passed === 4 && closeCalls === 0);
            }
            catches(iterableWithNext(function () { throw nextError; }), nextError, false).then(record);
            catches(iterableWithNext(function () { return 1; }), undefined, true).then(record);
            catches(iterableWithNext(function () {
                return {
                    get done() { throw doneError; }
                };
            }), doneError, false).then(record);
            catches(iterableWithNext(function () {
                return {
                    done: false,
                    get value() { throw valueError; }
                };
            }), valueError, false).then(record);
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
        .unwrap_or_else(|err| panic!("sync iterator abrupt results should reject: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["true".to_string()]
    );
}

#[test]
fn wasm_backend_awaited_optional_property_fulfills_with_property_value() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            async function read(source) {
                var value = await source?.value;
                return value;
            }
            read({ value: Promise.resolve(7) }).then(function (value) {
                print("fulfilled:" + value);
            });
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
        .unwrap_or_else(|err| panic!("awaited optional property should fulfill: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["sync".to_string(), "fulfilled:7".to_string()]
    );
}

#[test]
fn wasm_backend_awaited_optional_property_propagates_rejection() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            async function read(source) {
                var value = await source?.value;
                return value;
            }
            read({ value: Promise.reject("bad") }).then(
                function () { print("unreachable"); },
                function (reason) { print("rejected:" + reason); }
            );
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
        .unwrap_or_else(|err| panic!("awaited optional property should reject: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["sync".to_string(), "rejected:bad".to_string()]
    );
}

#[test]
fn wasm_backend_async_generator_initializes_parameters_before_selecting_instance_prototype() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            let bodyStarted = false;
            let stream = async function* (value = (stream.prototype = null)) {
                bodyStarted = true;
            };
            let oldPrototype = stream.prototype;
            let iterator = stream();
            print((Object.getPrototypeOf(iterator) !== oldPrototype) + ":" + bodyStarted);
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
        .unwrap_or_else(|err| {
            panic!("async-generator parameter initialization should run: {err:?}")
        });
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["true:false".to_string()]
    );
}

#[test]
fn wasm_backend_async_generator_object_spreads_resume_in_source_order() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            async function* stream() {
                yield { ...yield, y: 1, ...yield yield };
            }
            let iterator = stream();
            iterator.next();
            iterator.next({ x: 42 });
            iterator.next({ x: "ignored" });
            iterator.next({ y: 39 }).then(function (result) {
                print(result.value.x + ":" + result.value.y + ":" + result.done);
            });
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
        .unwrap_or_else(|err| panic!("async-generator object spreads should run: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["42:39:false".to_string()]
    );
}

#[test]
fn wasm_backend_promise_catch_boxes_object_coercible_receivers() {
    let source = r#"
            let catchMethod = Promise.prototype.catch;
            let booleanCount = 0;
            let numberCount = 0;
            let stringCount = 0;
            let symbolCount = 0;
            Boolean.prototype.then = function () { booleanCount += 1; };
            Number.prototype.then = function () { numberCount += 1; };
            String.prototype.then = function () { stringCount += 1; };
            Symbol.prototype.then = function () { symbolCount += 1; };
            catchMethod.call(true);
            catchMethod.call(34);
            catchMethod.call("");
            catchMethod.call(Symbol());
            booleanCount === 1
                && numberCount === 1
                && stringCount === 1
                && symbolCount === 1;
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
        .unwrap_or_else(|err| panic!("Promise catch should box primitive receivers: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_promise_species_getter_has_standard_descriptor_and_returns_receiver() {
    let source = r#"
            let descriptor = Object.getOwnPropertyDescriptor(Promise, Symbol.species);
            let receiver = {};
            Promise[Symbol.species] = {};
            descriptor.get.call(receiver) === receiver
                && Promise[Symbol.species] === Promise
                && descriptor.set === undefined
                && descriptor.enumerable === false
                && descriptor.configurable === true
                && descriptor.get.name === "get [Symbol.species]"
                && descriptor.get.length === 0;
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
        .unwrap_or_else(|err| panic!("Promise species descriptor should run: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_promise_prototype_has_standard_to_string_tag_descriptor() {
    let source = r#"
            let descriptor = Object.getOwnPropertyDescriptor(
                Promise.prototype,
                Symbol.toStringTag
            );
            Promise.prototype[Symbol.toStringTag] = "changed";
            descriptor.value === "Promise"
                && descriptor.writable === false
                && descriptor.enumerable === false
                && descriptor.configurable === true
                && Promise.prototype[Symbol.toStringTag] === "Promise";
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
        .unwrap_or_else(|err| panic!("Promise prototype toStringTag should run: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_directly_called_promise_prototype_methods_throw_receiver_type_errors() {
    // Both calls resolve statically, so each builtin runs with the caller
    // Realm's %Function.prototype% as its environment rather than its own
    // function object. The receiver TypeError must still come from that
    // Realm (27.2.5.4 step 2, 27.2.5.3 step 2) instead of trapping.
    let source = r#"
            let thenError = "none";
            try { Promise.prototype.then(); } catch (error) {
                thenError = error instanceof TypeError && error.constructor === TypeError;
            }
            const finallyMethod = Promise.prototype.finally;
            let finallyError = "none";
            try { finallyMethod(); } catch (error) {
                finallyError = error instanceof TypeError && error.constructor === TypeError;
            }
            thenError + "|" + finallyError;
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
            panic!("direct Promise prototype receiver errors should run: {err:?}")
        });
    assert!(
        outcome.note.contains("string(true|true)"),
        "{}",
        outcome.note
    );
}

#[test]
fn wasm_backend_promise_all_settled_preserves_input_order_and_record_descriptors() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            Promise.allSettled([
                Promise.reject("first"),
                Promise.resolve("second"),
                3
            ]).then(function (results) {
                let statusDescriptor = Object.getOwnPropertyDescriptor(results[0], "status");
                let reasonDescriptor = Object.getOwnPropertyDescriptor(results[0], "reason");
                let valueDescriptor = Object.getOwnPropertyDescriptor(results[1], "value");
                print(
                    results[0].status + ":" + results[0].reason + "," +
                    results[1].status + ":" + results[1].value + "," +
                    results[2].status + ":" + results[2].value + "," +
                    statusDescriptor.writable + ":" + statusDescriptor.enumerable + ":" + statusDescriptor.configurable + "," +
                    reasonDescriptor.writable + ":" + valueDescriptor.writable
                );
            });
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
        .unwrap_or_else(|err| panic!("Promise.allSettled records should run: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &[
            "sync".to_string(),
            "rejected:first,fulfilled:second,fulfilled:3,true:true:true,true:true".to_string(),
        ]
    );
}

#[test]
fn wasm_backend_promise_all_keyed_preserves_own_keys_in_a_null_prototype_object() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            let symbol = Symbol("symbol key");
            let prototype = { inherited: Promise.resolve("ignored") };
            let promises = {
                first: Promise.resolve("one"),
                second: 2
            };
            Object.setPrototypeOf(promises, prototype);
            Object.defineProperty(promises, "hidden", {
                value: Promise.resolve("ignored"),
                enumerable: false
            });
            Object.defineProperty(promises, symbol, {
                value: Promise.resolve("three"),
                enumerable: true,
                configurable: true,
                writable: true
            });
            Promise.allKeyed(promises).then(function (result) {
                let firstDescriptor = Object.getOwnPropertyDescriptor(result, "first");
                print(
                    (Object.getPrototypeOf(result) === null) + ":" +
                    Reflect.ownKeys(result).length + ":" +
                    Object.keys(result).join(",") + ":" +
                    result.first + ":" + result.second + ":" + result[symbol] + ":" +
                    firstDescriptor.writable + ":" + firstDescriptor.enumerable + ":" +
                    firstDescriptor.configurable
                );
            });
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
        .unwrap_or_else(|err| panic!("Promise.allKeyed should run: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &[
            "sync".to_string(),
            "true:3:first,second:one:2:three:true:true:true".to_string(),
        ]
    );
}

#[test]
fn wasm_backend_promise_resolution_calls_proxy_then_functions() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            let then = new Proxy(function (resolve) { resolve(11); }, {});
            let promise = Promise.resolve({ then: then });
            promise.then(function (value) { print("value:" + value); });
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
        .unwrap_or_else(|err| panic!("proxy then function should run: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["value:11".to_string()]
    );
}

#[test]
fn wasm_backend_reads_error_constructor_stack_as_an_ordinary_property() {
    // `stack` is not an ECMA-262 property of any Error constructor, of
    // %Function.prototype% or of %Object.prototype%. These reads used to be
    // rejected as an unsupported compiler slice; they are ordinary [[Get]]s
    // that find nothing unless the program defines the property itself.
    for (source, expected) in [
        ("Error.stack", "undefined(undefined)"),
        (
            "TypeError.stack === undefined && !('stack' in Error) \
                 && !Object.getOwnPropertyNames(Error).includes('stack');",
            "boolean(true)",
        ),
        ("Error.stack = 5; Error.stack;", "number(5)"),
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
            .unwrap_or_else(|err| panic!("`{source}` should run: {err:?}"));
        assert!(
            outcome.note.contains(expected),
            "source: {source}, note: {}",
            outcome.note
        );
    }

    // Function.prototype.toString throws a TypeError for a non-callable
    // receiver, before `Error.stack` is reached.
    let err = engine()
        .run_script(
            "Function.prototype.toString.call({}); Error.stack;",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect_err("a non-callable toString receiver should throw");
    assert_eq!(
        err.wasm_javascript_exception_constructor_name(),
        Some("TypeError"),
        "err: {}",
        err.message()
    );
    assert!(
        err.message()
            .contains("Function.prototype.toString receiver is not callable"),
        "err: {}",
        err.message()
    );
}

#[test]
fn wasm_backend_resumes_generator_object_spreads_in_source_order() {
    let source = r#"
            const symbol = Symbol("marker");
            let calls = 0;
            const generator = function* g() {
                calls += 1;
                yield {
                    ...yield yield,
                    ...(function(value) {
                        const yield = value;
                        return {...yield};
                    }(yield)),
                    ...yield,
                };
            };
            const iterator = generator();
            iterator.next();
            iterator.next();
            iterator.next({x: 10, a: 0, b: 0, [symbol]: 1});
            iterator.next({y: 20, a: 1, b: 1, [symbol]: 42});
            const yielded = iterator.next({z: 30, b: 2});
            const value = yielded.value;
            yielded.done === false && calls === 1
                && value.x === 10 && value.y === 20 && value.z === 30
                && value.a === 1 && value.b === 2 && value[symbol] === 42
                && Object.getOwnPropertySymbols(value)[0] === symbol
                && Object.keys(value).length === 5;
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
        .unwrap_or_else(|err| panic!("generator object spreads should resume: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_resumes_generator_array_and_mixed_object_spreads() {
    let source = r#"
            function* singleArray() { yield [...yield]; }
            function* nestedArray() { yield [...yield yield]; }
            function* mixedObject() { yield {...yield, y: 1, ...yield yield}; }

            const single = singleArray();
            single.next();
            const singleValue = single.next(["a", "b", "c"]).value;

            const nested = nestedArray();
            nested.next();
            const nestedMiddle = nested.next(["a", "b", "c"]);
            const nestedValue = nested.next(nestedMiddle.value).value;

            const mixed = mixedObject();
            mixed.next();
            mixed.next({x: 42});
            mixed.next({x: "ignored"});
            const mixedValue = mixed.next({y: 39}).value;

            singleValue !== nestedValue
                && singleValue.join("") === "abc"
                && nestedValue.join("") === "abc"
                && mixedValue.x === 42 && mixedValue.y === 39
                && Object.keys(mixedValue).length === 2;
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
        .unwrap_or_else(|err| panic!("generator literal spreads should resume: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_preserves_with_object_bindings_across_generator_resume() {
    let source = r#"
            let evaluations = 0;
            function createScope() {
                evaluations += 1;
                return { x: 2 };
            }
            function* sequence() {
                let x = 1;
                yield x;
                with (createScope()) {
                    yield x;
                    x = 3;
                    yield x;
                }
                yield x;
            }

            const iterator = sequence();
            const first = iterator.next();
            const second = iterator.next();
            const third = iterator.next();
            const fourth = iterator.next();
            const completed = iterator.next();

            first.value === 1 && first.done === false
                && second.value === 2 && second.done === false
                && third.value === 3 && third.done === false
                && fourth.value === 1 && fourth.done === false
                && completed.value === undefined && completed.done === true
                && evaluations === 1;
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
        .unwrap_or_else(|err| panic!("generator with binding should resume: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_preserves_property_assignment_references_across_generator_yield() {
    let source = r#"
            let order = "";
            const original = {};
            const replacement = {};
            let selectedTarget = original;
            let selectedKey = "before";
            function assignmentTarget() {
                order += "target";
                return selectedTarget;
            }
            function assignmentKey() {
                order += ",key";
                return selectedKey;
            }
            function yieldedValue() {
                order += ",yield";
                return 1;
            }
            function* assigned() {
                assignmentTarget()[assignmentKey()] = yield yieldedValue();
            }
            const assignedIterator = assigned();
            const assignedYield = assignedIterator.next();
            selectedTarget = replacement;
            selectedKey = "after";
            const assignedResult = assignedIterator.next(7);

            function* interrupted() {
                try {
                    original.interrupted = yield 2;
                } finally {
                    return 9;
                }
            }
            const interruptedIterator = interrupted();
            const interruptedYield = interruptedIterator.next();
            const interruptedResult = interruptedIterator.return(45);

            order === "target,key,yield"
                && assignedYield.value === 1 && assignedYield.done === false
                && assignedResult.value === undefined && assignedResult.done === true
                && original.before === 7
                && !("after" in replacement)
                && interruptedYield.value === 2 && interruptedYield.done === false
                && interruptedResult.value === 9 && interruptedResult.done === true
                && !("interrupted" in original);
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
        .unwrap_or_else(|err| panic!("generator property assignment should run: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_exposes_generator_function_prototype_topology() {
    let source = r#"
            const generator = function* sample() {};
            const functionPrototype = Object.getPrototypeOf(generator);
            const generatorPrototype = Object.getPrototypeOf(generator.prototype);
            const constructor = functionPrototype.constructor;
            const ownPrototypeDescriptor = Object.getOwnPropertyDescriptor(generator, "prototype");
            const intrinsicPrototypeDescriptor = Object.getOwnPropertyDescriptor(
                functionPrototype,
                "prototype"
            );
            const generatorConstructorDescriptor = Object.getOwnPropertyDescriptor(
                generatorPrototype,
                "constructor"
            );
            let prototypeCallThrows = false;
            try { functionPrototype(); } catch (error) { prototypeCallThrows = error instanceof TypeError; }

            Object.getPrototypeOf(functionPrototype) === Function.prototype
                && Object.getPrototypeOf(constructor) === Function
                && constructor.prototype === functionPrototype
                && functionPrototype.prototype === generatorPrototype
                && generatorPrototype.constructor === functionPrototype
                && Object.getOwnPropertyNames(generator.prototype).length === 0
                && ownPrototypeDescriptor.writable === true
                && ownPrototypeDescriptor.enumerable === false
                && ownPrototypeDescriptor.configurable === false
                && intrinsicPrototypeDescriptor.writable === false
                && intrinsicPrototypeDescriptor.enumerable === false
                && intrinsicPrototypeDescriptor.configurable === true
                && generatorConstructorDescriptor.writable === false
                && generatorConstructorDescriptor.enumerable === false
                && generatorConstructorDescriptor.configurable === true
                && constructor.name === "GeneratorFunction"
                && constructor.length === 1
                && typeof functionPrototype === "object"
                && prototypeCallThrows;
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
        .unwrap_or_else(|err| panic!("generator topology should compile: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_with_typed_array_reads_outer_object_binding() {
    let source = r#"
            const fallback = { value: 17 };
            function observe(view) {
                with (view) {
                    return fallback.value === 17;
                }
            }

            const numeric = new Uint8Array([1]);
            __scriptwDetachArrayBuffer(numeric.buffer);
            observe(numeric);
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
        .unwrap_or_else(|err| panic!("typed-array with object fallback should compile: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_copies_generator_object_rest_parameters() {
    let outcome = engine()
        .run_script(
            r#"
                    var explicitRest;
                    var defaultRest;
                    var explicit = function* ({ a, b, ...rest }) {
                        explicitRest = rest;
                    };
                    var defaulted = function* (
                        { a, ...rest } = { a: 5, x: 6, y: 7 }
                    ) {
                        defaultRest = rest;
                    };
                    explicit({ a: 1, b: 2, x: 3, y: 4 }).next();
                    defaulted().next();
                    explicitRest.a === undefined
                        && explicitRest.b === undefined
                        && explicitRest.x === 3
                        && explicitRest.y === 4
                        && defaultRest.a === undefined
                        && defaultRest.x === 6
                        && defaultRest.y === 7
                        && Object.getPrototypeOf(explicitRest) === Object.prototype
                        && Object.getPrototypeOf(defaultRest) === Object.prototype;
                "#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("generator object rest should compile: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_evaluates_computed_generator_object_parameter_keys() {
    let outcome = engine()
            .run_script(
                r#"
                    var keyCalls = 0;
                    var defaultCalls = 0;
                    var firstValue;
                    var secondValue;
                    var first = function* ({ [keyCalls = keyCalls + 1]: value = 9 }) {
                        firstValue = value;
                    };
                    var second = function* ({ [keyCalls = keyCalls + 1]: value = (defaultCalls = defaultCalls + 1, 9) }) {
                        secondValue = value;
                    };
                    first({ 1: 7 }).next();
                    second({ 2: undefined }).next();

                    var marker = {};
                    var bodyRan = false;
                    var thrown;
                    function thrower() { throw marker; }
                    var abrupt = function* ({ [thrower()]: value }) {
                        bodyRan = true;
                    };
                    try { abrupt({}); } catch (error) { thrown = error; }

                    keyCalls === 2
                        && defaultCalls === 1
                        && firstValue === 7
                        && secondValue === 9
                        && thrown === marker
                        && bodyRan === false;
                "#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("computed generator object parameter keys should compile: {err:?}")
            });
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_object_and_class_generator_methods_preserve_this_and_prototypes() {
    let source = r#"
            const object = {
                value: 3,
                *method() { return this.value; }
            };
            class Example {
                constructor() { this.value = 4; }
                *method() { return this.value; }
            }
            const objectIterator = object.method();
            const instance = new Example();
            const classIterator = instance.method();
            objectIterator.next().value === 3
                && classIterator.next().value === 4
                && Object.getPrototypeOf(objectIterator) === object.method.prototype
                && Object.getPrototypeOf(classIterator) === instance.method.prototype;
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
        .unwrap_or_else(|err| panic!("generator methods should run: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_iterator_zip_keyed_preserves_keys_and_result_descriptors() {
    let source = r#"
            const symbolKey = Symbol("symbol");
            const iterables = Object.create({ inherited: [99] });
            iterables.second = ["second"];
            Object.defineProperty(iterables, "hidden", {
                value: ["hidden"],
                enumerable: false
            });
            iterables[symbolKey] = ["symbol"];
            iterables.omitted = undefined;
            iterables[1] = ["one"];
            iterables[0] = ["zero"];

            const iterator = Iterator.zipKeyed(iterables);
            const result = iterator.next();
            const value = result.value;
            const keys = Reflect.ownKeys(value);
            const descriptor = Object.getOwnPropertyDescriptor(value, symbolKey);

            !result.done
                && Object.getPrototypeOf(value) === null
                && keys.length === 4
                && keys[0] === "0"
                && keys[1] === "1"
                && keys[2] === "second"
                && keys[3] === symbolKey
                && value[0] === "zero"
                && value[1] === "one"
                && value.second === "second"
                && value[symbolKey] === "symbol"
                && !Object.prototype.hasOwnProperty.call(value, "omitted")
                && !Object.prototype.hasOwnProperty.call(value, "hidden")
                && !Object.prototype.hasOwnProperty.call(value, "inherited")
                && descriptor.writable
                && descriptor.enumerable
                && descriptor.configurable
                && iterator.next().done;
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
        .unwrap_or_else(|err| panic!("Iterator.zipKeyed should preserve keys: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_object_from_entries_consumes_entries_and_closes_on_abrupt_completion() {
    let source = r#"
            const empty = Object.fromEntries([]);
            const symbol = Symbol("entry");
            const result = Object.fromEntries([["key", 1], [symbol, 2], ["key", 3]]);
            const descriptor = Object.getOwnPropertyDescriptor(result, "key");
            let closed = false;
            const original = new Error("original");
            const iterable = {
                [Symbol.iterator]() {
                    return {
                        next() {
                            return {
                                done: false,
                                value: {
                                    get 0() { throw original; },
                                    1: 1
                                }
                            };
                        },
                        return() {
                            closed = true;
                            return {};
                        }
                    };
                }
            };
            let caught;
            try {
                Object.fromEntries(iterable);
            } catch (error) {
                caught = error;
            }
            let stepClosed = false;
            const stepError = new Error("step");
            const stepIterable = {
                [Symbol.iterator]() {
                    return {
                        next() {
                            throw stepError;
                        },
                        return() {
                            stepClosed = true;
                            return {};
                        }
                    };
                }
            };
            let stepCaught;
            try {
                Object.fromEntries(stepIterable);
            } catch (error) {
                stepCaught = error;
            }

            Object.getPrototypeOf(empty) === Object.prototype
                && Reflect.ownKeys(empty).length === 0
                && result.key === 3
                && result[symbol] === 2
                && descriptor.writable
                && descriptor.enumerable
                && descriptor.configurable
                && caught === original
                && closed
                && stepCaught === stepError
                && !stepClosed;
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
        .unwrap_or_else(|err| panic!("Object.fromEntries should run: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_iterator_zip_keyed_closes_iterators_when_a_descriptor_lookup_throws() {
    let source = r#"
            const events = [];
            const original = new Error("descriptor");
            function iterator(name) {
                return {
                    next() {
                        return { done: true };
                    },
                    return() {
                        events.push(name);
                        throw new Error("ignored " + name);
                    }
                };
            }
            const iterables = new Proxy({
                first: iterator("first"),
                second: iterator("second"),
                third: null
            }, {
                getOwnPropertyDescriptor(target, key) {
                    if (key === "third") {
                        throw original;
                    }
                    return Reflect.getOwnPropertyDescriptor(target, key);
                }
            });
            let caught;
            try {
                Iterator.zipKeyed(iterables);
            } catch (error) {
                caught = error;
            }

            caught === original && events.join(",") === "second,first";
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
        .unwrap_or_else(|err| panic!("Iterator.zipKeyed descriptor closing should run: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}
