use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_wasm_true(source: &str) {
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("toLocaleString regression must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn nested_array_own_method_preserves_receiver_without_object_bootstrap() {
    assert_wasm_true(
        r#"
var inner = [8], calls = 0, correctThis = false;
inner.toLocaleString = function() {
  "use strict";
  calls++;
  correctThis = this === inner;
  return "custom";
};
var result = [inner].toLocaleString();
result === "custom" && calls === 1 && correctThis;
"#,
    );
}

#[test]
fn nested_array_inherited_getter_and_method_keep_original_receiver() {
    assert_wasm_true(
        r#"
var inner = [8], gets = 0, calls = 0, ok = true;
var prototype = Object.create(Array.prototype);
Object.defineProperty(prototype, "toLocaleString", { get() {
  gets++;
  ok = ok && this === inner;
  return function() { "use strict"; calls++; ok = ok && this === inner; return "inherited"; };
}});
Object.setPrototypeOf(inner, prototype);
var result = Array.prototype.toLocaleString.call([inner]);
result === "inherited" && gets === 1 && calls === 1 && ok;
"#,
    );
}

#[test]
fn nested_array_method_getter_is_observed_once_and_later_elements_stay_live() {
    assert_wasm_true(
        r#"
var trace = "", first = [1], second = [2], source = [first, null];
Object.defineProperty(first, "toLocaleString", { get() {
  trace += "get;";
  source[1] = second;
  return function() { trace += "first;"; return "A"; };
}});
second.toLocaleString = function() { trace += "second;"; return "B"; };
var result = Array.prototype.toLocaleString.call(source);
result === "A,B" && trace === "get;first;second;";
"#,
    );
}

#[test]
fn nested_array_throwing_getter_stops_before_later_indexed_get() {
    assert_wasm_true(
        r#"
var marker = {}, inner = [1], source = [inner, null], trace = "", caught = false;
Object.defineProperty(inner, "toLocaleString", { get() { trace += "method;"; throw marker; }});
Object.defineProperty(source, "1", { get() { trace += "next;"; return null; }});
try { Array.prototype.toLocaleString.call(source); }
catch (error) { caught = error === marker; }
caught && trace === "method;";
"#,
    );
}

#[test]
fn nested_array_noncallable_methods_throw_instead_of_using_to_string() {
    assert_wasm_true(
        r#"
var methods = [undefined, null, 3, {}], caught = 0, stringCalls = 0, ok = true;
for (var i = 0; i < methods.length; i++) {
  var inner = [9];
  inner.toLocaleString = methods[i];
  inner.toString = function() { stringCalls++; return "wrong fallback"; };
  try { Array.prototype.toLocaleString.call([inner]); ok = false; }
  catch (error) { if (error instanceof TypeError) caught++; else ok = false; }
}
ok && caught === methods.length && stringCalls === 0;
"#,
    );
}

#[test]
fn nested_array_callable_proxy_method_uses_proxy_aware_call() {
    assert_wasm_true(
        r#"
var inner = [9], calls = 0, targetCalls = 0, correctThis = false;
inner.toLocaleString = new Proxy(function() { targetCalls++; return "wrong target"; }, {
  apply(target, receiver, args) {
    calls++;
    correctThis = receiver === inner;
    return "proxy";
  }
});
var result = Array.prototype.toLocaleString.call([inner]);
result === "proxy" && calls === 1 && targetCalls === 0 && correctThis;
"#,
    );
}

#[test]
fn nested_array_method_result_is_converted_after_call() {
    assert_wasm_true(
        r#"
var inner = [9], trace = "", ok = true;
inner.toLocaleString = function() {
  trace += "call;";
  ok = ok && this === inner;
  return { [Symbol.toPrimitive](hint) { trace += hint + ";"; return "converted"; }};
};
var result = Array.prototype.toLocaleString.call([inner]);
result === "converted" && trace === "call;string;" && ok;
"#,
    );
}

#[test]
fn nested_array_method_throw_preserves_identity_and_stops_iteration() {
    assert_wasm_true(
        r#"
var marker = {}, inner = [9], source = [inner, null], trace = "", caught = false;
inner.toLocaleString = function() { trace += "call;"; throw marker; };
Object.defineProperty(source, "1", { get() { trace += "next;"; return null; }});
try { Array.prototype.toLocaleString.call(source); }
catch (error) { caught = error === marker; }
caught && trace === "call;";
"#,
    );
}

#[test]
fn nested_array_result_conversion_throw_stops_iteration() {
    assert_wasm_true(
        r#"
var marker = {}, inner = [9], trace = "", caught = false;
inner.toLocaleString = function() {
  trace += "call;";
  return { toString() { trace += "convert;"; throw marker; }};
};
var later = { toLocaleString() { trace += "later;"; return "wrong"; }};
try { Array.prototype.toLocaleString.call([inner, later]); }
catch (error) { caught = error === marker; }
caught && trace === "call;convert;";
"#,
    );
}

#[test]
fn nested_default_array_method_recurses_through_element_locale_methods() {
    assert_wasm_true(
        r#"
var calls = 0, ordinaryStringCalls = 0;
var element = {
  toLocaleString() { calls++; return "localized"; },
  toString() { ordinaryStringCalls++; return "ordinary"; }
};
var result = Array.prototype.toLocaleString.call([[element]]);
result === "localized" && calls === 1 && ordinaryStringCalls === 0;
"#,
    );
}

#[test]
fn arguments_element_custom_method_preserves_original_receiver() {
    assert_wasm_true(
        r#"
var inner = (function() { return arguments; })(4, 5), calls = 0, correctThis = false;
inner.toLocaleString = function() {
  "use strict";
  calls++;
  correctThis = this === inner;
  return "arguments custom";
};
var result = Array.prototype.toLocaleString.call([inner]);
result === "arguments custom" && calls === 1 && correctThis;
"#,
    );
}

#[test]
fn arguments_element_getter_is_read_once() {
    assert_wasm_true(
        r#"
var inner = (function() { return arguments; })(4), gets = 0, calls = 0, ok = true;
Object.defineProperty(inner, "toLocaleString", { get() {
  gets++;
  ok = ok && this === inner;
  return function() { "use strict"; calls++; ok = ok && this === inner; return "arguments getter"; };
}});
var result = Array.prototype.toLocaleString.call([inner]);
result === "arguments getter" && gets === 1 && calls === 1 && ok;
"#,
    );
}

#[test]
fn arguments_element_noncallable_method_throws_type_error() {
    assert_wasm_true(
        r#"
var inner = (function() { return arguments; })(4), caught = false, stringCalls = 0;
inner.toLocaleString = null;
inner.toString = function() { stringCalls++; return "wrong fallback"; };
try { Array.prototype.toLocaleString.call([inner]); }
catch (error) { caught = error instanceof TypeError; }
caught && stringCalls === 0;
"#,
    );
}

#[test]
fn generic_outer_receiver_invokes_array_and_arguments_elements() {
    assert_wasm_true(
        r#"
var array = [1], args = (function() { return arguments; })(2);
array.toLocaleString = function() { return "A"; };
args.toLocaleString = function() { return "G"; };
var result = Array.prototype.toLocaleString.call({ 0: array, 1: args, length: 2 });
result === "A,G";
"#,
    );
}

#[test]
fn existing_object_function_and_typed_array_element_dispatch_remains_live() {
    assert_wasm_true(
        r#"
var object = {}, fn = function() {}, typed = new Uint8Array([7]), trace = "", ok = true;
object.toLocaleString = function() { ok = ok && this === object; trace += "O"; return "object"; };
fn.toLocaleString = function() { ok = ok && this === fn; trace += "F"; return "function"; };
typed.toLocaleString = function() { ok = ok && this === typed; trace += "T"; return "typed"; };
var result = Array.prototype.toLocaleString.call([object, fn, typed]);
result === "object,function,typed" && trace === "OFT" && ok;
"#,
    );
}

#[test]
fn nullish_holes_and_primitive_elements_keep_existing_behavior() {
    assert_wasm_true(
        r#"
var result = Array.prototype.toLocaleString.call([1, true, 1n, "s", null, undefined, ,]);
result === "1,true,1,s,,,";
"#,
    );
}

#[test]
fn unmodified_nested_arrays_keep_recursive_separator_behavior() {
    assert_wasm_true(
        r#"
var result = Array.prototype.toLocaleString.call([[], [1, [2]], null]);
result === ",1,2,";
"#,
    );
}

#[test]
fn direct_typed_array_locale_method_retains_numeric_receiver_identity() {
    assert_wasm_true(
        r#"
var old = Number.prototype.toLocaleString, calls = 0, sum = 0;
Number.prototype.toLocaleString = function() {
  "use strict";
  calls++;
  sum += this;
  return "n" + this;
};
var result = Uint8Array.prototype.toLocaleString.call(new Uint8Array([3, 5]));
Number.prototype.toLocaleString = old;
result === "n3,n5" && calls === 2 && sum === 8;
"#,
    );
}
