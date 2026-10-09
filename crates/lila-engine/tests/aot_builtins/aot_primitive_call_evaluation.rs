use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions,
};

fn assert_modes(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "'use strict';\n"] {
        let outcome = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &format!("{directive}{source}"),
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("primitive calls compile and execute through Wasm AOT");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{outcome:?}"
        );
    }
}

#[test]
fn primitive_calls_preserve_ignored_arguments_and_undefined_symbol_effects() {
    assert_modes(
        r#"
        let trace = '';
        if (Number('2', trace += 'number;') !== 2) throw 'Number';
        if (Boolean(false, trace += 'boolean;') !== false) throw 'Boolean';
        if (!Boolean({ [(trace += 'key;', 'x')]: (trace += 'value;', 1) })) throw 'object';
        if (!Boolean([(trace += 'element;', 1)])) throw 'array';
        if (parseFloat('4tail', trace += 'float;') !== 4) throw 'parseFloat';
        const bare = Symbol(void (trace += 'description;'), trace += 'extra;');
        if (bare.description !== undefined) throw 'undefined description';
        const description = { toString() { trace += 'coerce;'; return 'ready'; } };
        const described = Symbol(description, trace += 'before;');
        if (described.description !== 'ready') throw 'description';
        if (trace !== 'number;boolean;key;value;element;float;description;extra;before;coerce;') {
            throw trace;
        }
        const marker = {};
        let caught;
        try { Symbol({ toString() { throw marker; } }, trace += 'before-throw;'); }
        catch (error) { caught = error; }
        if (caught !== marker || trace.slice(-13) !== 'before-throw;') throw 'abrupt ordering';
        let conversions = 0;
        function fail() { throw marker; }
        caught = undefined;
        try { Symbol({ toString() { conversions++; return 'unused'; } }, fail()); }
        catch (error) { caught = error; }
        if (caught !== marker || conversions !== 0) throw 'argument abrupt cutoff';
        if (Boolean(...[]) !== false || Boolean(...[true]) !== true) throw 'Boolean spread';
        if (Number(...[2n]) !== 2 || String(...[Symbol('s')]) !== 'Symbol(s)') throw 'primitive spread';
        true;
    "#,
    );
}

#[test]
fn primitive_methods_keep_constructor_effects_and_use_runtime_number_formatting() {
    assert_modes(
        r#"
        let trace = '';
        if (Number(7, trace += 'number;').toFixed(2) !== '7.00') throw 'fixed';
        if (new Number(6, trace += 'wrapper;').toString() !== '6') throw 'wrapper';
        if (Boolean(false, trace += 'boolean;').toString(trace += 'argument;') !== 'false') throw 'boolean';
        if (new Boolean(true, trace += 'boxed;').valueOf(trace += 'ignored;') !== true) throw 'boxed';
        if (trace !== 'number;wrapper;boolean;argument;boxed;ignored;') throw trace;
        if ((6).toString(2.9) !== '110') throw 'fractional radix';
        if ((1e20).toFixed(0) !== '100000000000000000000') throw 'wide fixed';
        if ((1e-20).toExponential() !== '1e-20') throw 'small exponential';
        if ((12345).toExponential(3) !== '1.235e+4') throw 'rounded exponential';
        if ((1.5).toPrecision(2) !== '1.5') throw 'precision';
        if (Infinity.toExponential(101) !== 'Infinity') throw 'non-finite exponential';
        let range = false;
        try { Infinity.toFixed(101); } catch (error) { range = error instanceof RangeError; }
        if (!range) throw 'non-finite fixed range';
        if (+'0x10000000000000000' !== 18446744073709551616) throw 'wide radix conversion';
        if (!Number.isNaN(+'0x+1') || !Number.isNaN(+'0b+1') || !Number.isNaN(+'0o+1')) throw 'radix grammar';
        true;
    "#,
    );
}

#[test]
fn shadowed_primitive_names_and_receivers_keep_their_live_values() {
    assert_modes(
        r#"
        function convert(undefined, NaN, Infinity) {
            if (Number(undefined) !== 9) throw 'shadowed undefined';
            if (String(NaN) !== 'Symbol(shadow)') throw 'shadowed NaN';
            if (Boolean(Infinity) !== false) throw 'shadowed Infinity';
        }
        convert(9n, Symbol('shadow'), 0);
        function receivers(Number, Boolean) {
            if (Number(7).toFixed(1) !== 'custom-number') throw 'Number receiver';
            if (Boolean(false).toString() !== 'custom-boolean') throw 'Boolean receiver';
        }
        receivers(
            function () { return { toFixed() { return 'custom-number'; } }; },
            function () { return { toString() { return 'custom-boolean'; } }; }
        );
        let gets = 0;
        function constants(Number) { return +Number.NaN + +Number.POSITIVE_INFINITY; }
        if (constants({ get NaN() { gets++; return 7; }, get POSITIVE_INFINITY() { gets++; return 8; } }) !== 15 || gets !== 2) throw 'Number property';
        function enumerable(globalThis, Number, Boolean, RangeError) {
            return globalThis.propertyIsEnumerable('NaN') && Number.propertyIsEnumerable('MAX_VALUE') &&
                Boolean.propertyIsEnumerable('prototype') && RangeError.propertyIsEnumerable('prototype');
        }
        if (!enumerable({ NaN: 1 }, { MAX_VALUE: 1 }, { prototype: 1 }, { prototype: 1 })) throw 'enumerability';
        true;
    "#,
    );
}

#[test]
fn mutable_static_methods_are_acquired_before_argument_evaluation() {
    assert_modes(
        r#"
        let trace = '';
        const receiver = Object;
        Object.defineProperty(Object, 'is', {
            configurable: true,
            get() {
                trace += 'get;';
                return function (left, right) {
                    if (this !== receiver || left !== 1 || right !== 2) throw 'call receiver';
                    trace += 'call;';
                    return 'replacement';
                };
            }
        });
        if (Object.is((trace += 'left;', 1), (trace += 'right;', 2)) !== 'replacement') throw 'Object.is';
        if (trace !== 'get;left;right;call;') throw trace;
        const marker = {};
        Object.defineProperty(Object, 'is', { configurable: true, get() { throw marker; } });
        let caught;
        try { Object.is((trace += 'unreachable;', 1), 2); } catch (error) { caught = error; }
        if (caught !== marker || trace !== 'get;left;right;call;') throw 'getter abrupt cutoff';
        Math.pow = function () { return 'pow'; };
        Math.clz32 = function () { return 'clz32'; };
        Math.round = function () { return 'round'; };
        String.fromCharCode = function () { return 'char'; };
        String.fromCodePoint = function () { return 'point'; };
        String.raw = function () { return 'raw'; };
        Object.keys = function () { return 'keys'; };
        Object.values = function () { return 'values'; };
        Object.entries = function () { return 'entries'; };
        if (Math.pow(2, 3) !== 'pow' || Math.clz32(1) !== 'clz32' || Math.round(1.5) !== 'round') throw 'Math';
        if (String.fromCharCode(65) !== 'char' || String.fromCodePoint(65) !== 'point' || String.raw({ raw: ['x'] }) !== 'raw') throw 'String';
        if (Object.keys({}) !== 'keys' || Object.values({}) !== 'values' || Object.entries({}) !== 'entries') throw 'Object';
        Object.prototype.propertyIsEnumerable = function () { return 'enumerable'; };
        if (globalThis.propertyIsEnumerable('NaN') !== 'enumerable' || Number.propertyIsEnumerable('MAX_VALUE') !== 'enumerable') throw 'live enumerability';
        true;
    "#,
    );
}
