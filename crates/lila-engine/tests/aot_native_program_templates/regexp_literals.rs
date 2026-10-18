use lila_engine::{
    CompileOptions, Engine, ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder,
    RunOptions,
};

#[test]
fn regexp_literal_allocation_keeps_fresh_state_exact_descriptor_and_intrinsic_prototype() {
    let source = r#"
        var intrinsic = RegExp.prototype;
        var setterCalls = 0;
        Object.defineProperty(intrinsic, 'lastIndex', {
            set(value) { setterCalls++; }, configurable: true
        });
        globalThis.RegExp = function () { throw new Error('global constructor called'); };
        function check(value, prototype) {
            gc();
            if (Object.getPrototypeOf(value) !== prototype) throw 1;
            var d = Object.getOwnPropertyDescriptor(value, 'lastIndex');
            if (!d.writable || d.enumerable || d.configurable || 1 / d.value !== Infinity) throw 2;
            if (Object.getOwnPropertyNames(value).join(',') !== 'lastIndex') throw 3;
            if (value.source !== 'a' || value.flags !== 'g') throw 4;
            if (!value.test('a') || value.lastIndex !== 1) throw 5;
        }
        function literal() { return /a/g; }
        function direct() { return eval('/a/g'); }
        var constructed = Function('return /a/g;');
        for (var i = 0; i < 4; i++) {
            var a = literal(), b = literal();
            if (a === b) throw 6;
            check(a, intrinsic);
            check(b, intrinsic);
            check(direct(), intrinsic);
            check(constructed(), intrinsic);
        }
        if (setterCalls !== 0) throw 7;
        var other = $262.createRealm();
        var foreign = other.global.Function('return /a/g;');
        check(foreign(), other.global.RegExp.prototype);
        1;
    "#;
    for strict in [false, true] {
        let text = if strict {
            format!("\"use strict\";\n{source}")
        } else {
            source.to_owned()
        };
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &text,
                CompileOptions {
                    host_surface_policy: lila_ir::HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    timeout_ms: Some(60_000),
                    ..RunOptions::default()
                },
            )
            .unwrap();
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(1.0)))
        );
    }
}
