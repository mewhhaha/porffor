use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_wasm_aot_boolean(name: &str, source: &str) {
    assert_wasm_aot_boolean_with_policy(name, source, HostSurfacePolicy::Product);
}

fn assert_wasm_aot_boolean_with_policy(
    name: &str,
    source: &str,
    host_surface_policy: HostSurfacePolicy,
) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            source,
            CompileOptions {
                host_surface_policy,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("{name}: {error:?}"));
    assert!(
        outcome.note.contains("boolean(true)"),
        "{name}: {}",
        outcome.note
    );
}

#[test]
fn borrowed_array_method_observes_typed_array_own_length() {
    assert_wasm_aot_boolean(
        "borrowed_array_method_observes_typed_array_own_length",
        r#"
        var source = new Uint8Array([7, 9]);
        Object.defineProperty(source, "length", { value: 1 });
        Array.prototype.toLocaleString.call(source) === "7";
        "#,
    );
}

#[test]
fn borrowed_array_method_observes_inherited_typed_array_length() {
    assert_wasm_aot_boolean(
        "borrowed_array_method_observes_inherited_typed_array_length",
        r#"
        var reads = 0;
        var source = new Uint8Array([7, 9]);
        Object.defineProperty(Uint8Array.prototype, "length", {
            configurable: true,
            get: function () { reads++; return 1; }
        });
        var result = Array.prototype.toLocaleString.call(source);
        result === "7" && reads === 1;
        "#,
    );
}

#[test]
fn borrowed_array_method_coerces_typed_array_length_once() {
    assert_wasm_aot_boolean(
        "borrowed_array_method_coerces_typed_array_length_once",
        r#"
        var reads = 0;
        var conversions = 0;
        var source = new Uint8Array([7, 9]);
        Object.defineProperty(source, "length", {
            get: function () {
                reads++;
                return { valueOf: function () { conversions++; return 1.9; } };
            }
        });
        var result = Array.prototype.toLocaleString.call(source);
        result === "7" && reads === 1 && conversions === 1;
        "#,
    );
}

#[test]
fn arguments_public_length_limits_iteration() {
    assert_wasm_aot_boolean(
        "arguments_public_length_limits_iteration",
        r#"
        function check() {
            arguments.length = 1;
            return Array.prototype.toLocaleString.call(arguments) === "first";
        }
        check("first", "second");
        "#,
    );
}

#[test]
fn arguments_length_accessor_is_observed_once() {
    assert_wasm_aot_boolean(
        "arguments_length_accessor_is_observed_once",
        r#"
        var reads = 0;
        function check() {
            Object.defineProperty(arguments, "length", {
                get: function () { reads++; return "1.9"; }
            });
            return Array.prototype.toLocaleString.call(arguments) === "first";
        }
        check("first", "second") && reads === 1;
        "#,
    );
}

#[test]
fn throwing_length_getter_stops_before_index_get() {
    assert_wasm_aot_boolean(
        "throwing_length_getter_stops_before_index_get",
        r#"
        var sentinel = {};
        var reads = 0;
        var source = {};
        Object.defineProperty(source, "length", { get: function () { throw sentinel; } });
        Object.defineProperty(source, "0", { get: function () { reads++; return "bad"; } });
        var caught = false;
        try { Array.prototype.toLocaleString.call(source); }
        catch (error) { caught = error === sentinel; }
        caught && reads === 0;
        "#,
    );
}

#[test]
fn throwing_length_coercion_stops_before_index_get() {
    assert_wasm_aot_boolean(
        "throwing_length_coercion_stops_before_index_get",
        r#"
        var sentinel = {};
        var reads = 0;
        var source = { length: { valueOf: function () { throw sentinel; } } };
        Object.defineProperty(source, "0", { get: function () { reads++; return "bad"; } });
        var caught = false;
        try { Array.prototype.toLocaleString.call(source); }
        catch (error) { caught = error === sentinel; }
        caught && reads === 0;
        "#,
    );
}

#[test]
fn non_positive_and_nan_lengths_do_not_read_elements() {
    assert_wasm_aot_boolean(
        "non_positive_and_nan_lengths_do_not_read_elements",
        r#"
        var reads = 0;
        var source = { length: -1 };
        Object.defineProperty(source, "0", { get: function () { reads++; return "bad"; } });
        var negative = Array.prototype.toLocaleString.call(source);
        source.length = NaN;
        var nan = Array.prototype.toLocaleString.call(source);
        source.length = undefined;
        var absent = Array.prototype.toLocaleString.call(source);
        negative === "" && nan === "" && absent === "" && reads === 0;
        "#,
    );
}

#[test]
fn length_is_captured_but_indexed_gets_remain_live() {
    assert_wasm_aot_boolean(
        "length_is_captured_but_indexed_gets_remain_live",
        r#"
        var source = { length: 2, 1: "old" };
        source[0] = { toLocaleString: function () {
            source.length = 0;
            source[1] = "new";
            return "first";
        } };
        Array.prototype.toLocaleString.call(source) === "first,new";
        "#,
    );
}

#[test]
fn nested_array_own_locale_method_is_invoked() {
    assert_wasm_aot_boolean(
        "nested_array_own_locale_method_is_invoked",
        r#"
        var inner = [1, 2];
        var calls = 0;
        inner.toLocaleString = function () {
            calls++;
            return this === inner ? "custom" : "wrong-this";
        };
        [inner].toLocaleString() === "custom" && calls === 1;
        "#,
    );
}

#[test]
fn nested_arguments_own_locale_method_is_invoked() {
    assert_wasm_aot_boolean(
        "nested_arguments_own_locale_method_is_invoked",
        r#"
        function make() { return arguments; }
        var inner = make(1, 2);
        inner.toLocaleString = function () { return this === inner ? "args" : "wrong-this"; };
        [inner].toLocaleString() === "args";
        "#,
    );
}

#[test]
fn nested_array_method_getter_runs_once_with_correct_receiver() {
    assert_wasm_aot_boolean(
        "nested_array_method_getter_runs_once_with_correct_receiver",
        r#"
        var inner = [1];
        var gets = 0;
        var calls = 0;
        var correctReceiver = false;
        Object.defineProperty(inner, "toLocaleString", {
            get: function () {
                gets++;
                correctReceiver = this === inner;
                return function () { calls++; return this === inner ? "ok" : "bad"; };
            }
        });
        [inner].toLocaleString() === "ok" && gets === 1 && calls === 1 && correctReceiver;
        "#,
    );
}

#[test]
fn nested_array_non_callable_locale_method_throws() {
    assert_wasm_aot_boolean(
        "nested_array_non_callable_locale_method_throws",
        r#"
        var inner = [1];
        var fallbackCalls = 0;
        inner.toLocaleString = 17;
        inner.toString = function () { fallbackCalls++; return "fallback"; };
        var caught = false;
        try { [inner].toLocaleString(); }
        catch (error) { caught = error instanceof TypeError; }
        caught && fallbackCalls === 0;
        "#,
    );
}

#[test]
fn throwing_nested_locale_getter_preserves_exception() {
    assert_wasm_aot_boolean(
        "throwing_nested_locale_getter_preserves_exception",
        r#"
        var sentinel = {};
        var inner = [1];
        var fallbackCalls = 0;
        Object.defineProperty(inner, "toLocaleString", { get: function () { throw sentinel; } });
        inner.toString = function () { fallbackCalls++; return "fallback"; };
        var caught = false;
        try { [inner].toLocaleString(); }
        catch (error) { caught = error === sentinel; }
        caught && fallbackCalls === 0;
        "#,
    );
}

#[test]
fn throwing_locale_call_stops_before_later_elements() {
    assert_wasm_aot_boolean(
        "throwing_locale_call_stops_before_later_elements",
        r#"
        var sentinel = {};
        var reads = 0;
        var source = [{ toLocaleString: function () { throw sentinel; } }];
        Object.defineProperty(source, "1", { get: function () { reads++; return "bad"; } });
        var caught = false;
        try { source.toLocaleString(); }
        catch (error) { caught = error === sentinel; }
        caught && reads === 0;
        "#,
    );
}

#[test]
fn locale_result_is_stringified_before_next_index_get() {
    assert_wasm_aot_boolean(
        "locale_result_is_stringified_before_next_index_get",
        r#"
        var log = "";
        var source = { length: 2 };
        Object.defineProperty(source, "0", { get: function () {
            log += "0";
            return { toLocaleString: function () {
                log += "c";
                return { toString: function () { log += "s"; return "first"; } };
            } };
        } });
        Object.defineProperty(source, "1", { get: function () { log += "1"; return "second"; } });
        var result = Array.prototype.toLocaleString.call(source);
        result === "first,second" && log === "0cs1";
        "#,
    );
}

#[test]
fn throwing_result_stringification_preserves_exception() {
    assert_wasm_aot_boolean(
        "throwing_result_stringification_preserves_exception",
        r#"
        var sentinel = {};
        var reads = 0;
        var source = [{ toLocaleString: function () {
            return { toString: function () { throw sentinel; } };
        } }];
        Object.defineProperty(source, "1", { get: function () { reads++; return "bad"; } });
        var caught = false;
        try { source.toLocaleString(); }
        catch (error) { caught = error === sentinel; }
        caught && reads === 0;
        "#,
    );
}

#[test]
fn nullish_and_missing_elements_keep_separators() {
    assert_wasm_aot_boolean(
        "nullish_and_missing_elements_keep_separators",
        r#"
        [null, undefined, , "last"].toLocaleString() === ",,,last";
        "#,
    );
}

#[test]
fn inherited_index_getter_is_observed() {
    assert_wasm_aot_boolean(
        "inherited_index_getter_is_observed",
        r#"
        var gets = 0;
        var prototype = {};
        var source = Object.create(prototype);
        source.length = 1;
        Object.defineProperty(prototype, "0", { get: function () {
            gets++;
            return this === source ? "inherited" : "wrong-this";
        } });
        Array.prototype.toLocaleString.call(source) === "inherited" && gets === 1;
        "#,
    );
}

#[test]
fn arguments_index_accessor_is_observed() {
    assert_wasm_aot_boolean(
        "arguments_index_accessor_is_observed",
        r#"
        var gets = 0;
        function check() {
            Object.defineProperty(arguments, "0", {
                get: function () { gets++; return "accessor"; }
            });
            return Array.prototype.toLocaleString.call(arguments) === "accessor";
        }
        check("old") && gets === 1;
        "#,
    );
}

#[test]
fn primitive_method_getter_and_call_keep_unboxed_this() {
    assert_wasm_aot_boolean(
        "primitive_method_getter_and_call_keep_unboxed_this",
        r#"
        var gets = 0;
        var correctReceiver = false;
        Object.defineProperty(Number.prototype, "toLocaleString", {
            configurable: true,
            get: function () {
                "use strict";
                gets++;
                correctReceiver = typeof this === "number" && this === 7;
                return function () {
                    "use strict";
                    return typeof this === "number" && this === 7 ? "seven" : "wrong-this";
                };
            }
        });
        [7].toLocaleString() === "seven" && gets === 1 && correctReceiver;
        "#,
    );
}

#[test]
fn direct_typed_array_method_ignores_public_length() {
    assert_wasm_aot_boolean(
        "direct_typed_array_method_ignores_public_length",
        r#"
        var source = new Uint8Array([7, 9]);
        var reads = 0;
        Object.defineProperty(source, "length", { get: function () { reads++; throw "unexpected"; } });
        source.toLocaleString() === "7,9" && reads === 0;
        "#,
    );
}

#[test]
fn direct_typed_array_method_rejects_non_typed_receiver() {
    assert_wasm_aot_boolean(
        "direct_typed_array_method_rejects_non_typed_receiver",
        r#"
        var reads = 0;
        var source = {};
        Object.defineProperty(source, "length", { get: function () { reads++; return 0; } });
        var caught = false;
        try { Uint8Array.prototype.toLocaleString.call(source); }
        catch (error) { caught = error instanceof TypeError; }
        caught && reads === 0;
        "#,
    );
}

#[test]
fn array_method_rejects_nullish_receivers() {
    assert_wasm_aot_boolean(
        "array_method_rejects_nullish_receivers",
        r#"
        var nullCaught = false;
        var undefinedCaught = false;
        try { Array.prototype.toLocaleString.call(null); }
        catch (error) { nullCaught = error instanceof TypeError; }
        try { Array.prototype.toLocaleString.call(undefined); }
        catch (error) { undefinedCaught = error instanceof TypeError; }
        nullCaught && undefinedCaught;
        "#,
    );
}

#[test]
fn indexed_proxy_revocation_keeps_the_locale_method_realm() {
    assert_wasm_aot_boolean_with_policy(
        "indexed_proxy_revocation_keeps_the_locale_method_realm",
        r#"
var other = __lilaCreateRealm().global;
var pair, lengthGets = 0;
pair = Proxy.revocable({ length: 1, 0: 7 }, {
  get(target, key, receiver) {
    if (key === 'length') { lengthGets++; pair.revoke(); return 1; }
    throw 'revoked indexed Get must not call this trap';
  }
});
var caught = false;
try { other.Array.prototype.toLocaleString.call(pair.proxy); }
catch (error) {
  caught = Object.getPrototypeOf(error) === other.TypeError.prototype &&
    Object.getPrototypeOf(error) !== TypeError.prototype;
}
caught && lengthGets === 1;
"#,
        HostSurfacePolicy::Test262,
    );
}

#[test]
fn indexed_callable_proxy_getters_keep_the_locale_method_realm() {
    assert_wasm_aot_boolean_with_policy(
        "indexed_callable_proxy_getters_keep_the_locale_method_realm",
        r#"
var other = __lilaCreateRealm().global;
var getter = Proxy.revocable(function() { throw 'revoked getter body'; }, {});
getter.revoke();
function make() { return arguments; }
var array = [7], args = make(7), ordinary = { length: 1, 0: 7 };
Object.defineProperty(array, '0', { get: getter.proxy });
Object.defineProperty(args, '0', { get: getter.proxy });
Object.defineProperty(ordinary, '0', { get: getter.proxy });
var receivers = [array, args, ordinary], correct = 0;
for (var i = 0; i < receivers.length; i++) {
  try { other.Array.prototype.toLocaleString.call(receivers[i]); }
  catch (error) {
    if (Object.getPrototypeOf(error) === other.TypeError.prototype &&
        Object.getPrototypeOf(error) !== TypeError.prototype) correct++;
  }
}
correct === 3;
"#,
        HostSurfacePolicy::Test262,
    );
}
