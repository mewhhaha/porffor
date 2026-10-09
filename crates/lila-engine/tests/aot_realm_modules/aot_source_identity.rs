use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, ObservedNumber,
    RealmBuilder, RunOptions,
};

fn assert_modes(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "'use strict';\n"] {
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &format!("{directive}{source}"),
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("source identity controls compile and execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{observed:?}"
        );
    }
}

#[test]
fn unresolved_bpe_and_declared_bpe_follow_environment_records() {
    assert_modes(
        r#"
        let caught = 0;
        try { BPE; } catch (error) {
            if (error instanceof ReferenceError) caught++;
        }
        if (caught !== 1 || typeof BPE !== 'undefined') throw 'unbound BPE';
        function bound(BPE) { return BPE; }
        if (bound(17) !== 17) throw 'parameter BPE';
        { const BPE = 23; if (BPE !== 23) throw 'lexical BPE'; }
        let reads = 0;
        Object.defineProperty(globalThis, 'BPE', {
            configurable: true, get() { reads++; return 29; }
        });
        if (BPE !== 29 || reads !== 1) throw 'global accessor BPE';
        delete globalThis.BPE;
        try { BPE; } catch (error) {
            if (error instanceof ReferenceError) caught++;
        }
        if (caught !== 2) throw 'deleted BPE';
        262;
        "#,
    );
}

#[test]
fn ordinary_generator_marker_properties_do_not_override_methods_or_arguments() {
    assert_modes(
        r#"
        let trace = '';
        const marker = {};
        const receiver = {
            $LilaYieldStarGenerator: true,
            $LilaYieldStarReturnNonObject: true,
            $LilaYieldStarThrowNonObject: true,
            get next() {
                trace += 'get;';
                return function (value) {
                    if (this !== receiver) throw 'next receiver';
                    trace += 'next;';
                    return value;
                };
            },
            return(value) {
                if (this !== receiver) throw 'return receiver';
                trace += 'return;';
                return value;
            },
            throw(value) {
                if (this !== receiver) throw 'throw receiver';
                trace += 'throw;';
                throw value;
            }
        };
        if (receiver.next((trace += 'arg-next;', 7)) !== 7) throw 'next value';
        if (receiver.return((trace += 'arg-return;', 11)) !== 11) throw 'return value';
        let caught;
        try { receiver.throw((trace += 'arg-throw;', marker)); }
        catch (error) { caught = error; }
        if (caught !== marker) throw 'throw value';
        if (trace !== 'get;arg-next;next;arg-return;return;arg-throw;throw;') throw trace;
        262;
        "#,
    );
}

#[test]
fn named_numeric_receivers_keep_gets_coercions_and_abrupt_operand_order() {
    assert_modes(
        r#"
        let trace = '';
        function power(Number, Math) { return Number.EPSILON ** Math.PI; }
        const number = {
            get EPSILON() {
                trace += 'left-get;';
                return { valueOf() { trace += 'left-number;'; return 3; } };
            }
        };
        const math = {
            get PI() {
                trace += 'right-get;';
                return { valueOf() { trace += 'right-number;'; return 2; } };
            }
        };
        if (power(number, math) !== 9) throw 'shadowed value';
        if (trace !== 'left-get;right-get;left-number;right-number;') throw trace;
        const marker = {};
        trace = '';
        let caught;
        try { power({ get EPSILON() { throw marker; } }, math); }
        catch (error) { caught = error; }
        if (caught !== marker || trace !== '') throw 'abrupt left operand';
        if (Number.MIN_VALUE ** 1 !== 5e-324) throw 'smallest subnormal';
        if ((2 + 1) ** (1 + 1) !== 9) throw 'literal exponentiation';
        262;
        "#,
    );
}
