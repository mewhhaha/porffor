use super::succeeds;
use lila_engine::{
    CompileOptions, Engine, ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder,
    RunOptions,
};

#[test]
fn method_names_preserve_utf16_symbols_prefixes_and_descriptors_across_native_compilation() {
    let source = r#"
        function checkNames() {
            function check(fn, expected) {
                gc();
                var descriptor = Object.getOwnPropertyDescriptor(fn, 'name');
                if (fn.name !== expected || descriptor.value !== expected) throw 1;
                if (descriptor.writable || descriptor.enumerable || !descriptor.configurable) throw 2;
                if (Object.getOwnPropertyNames(fn).slice(0, 2).join(',') !== 'length,name') throw 3;
            }
            var keys = ['text\ud800', Symbol(), Symbol(''), Symbol('\ud800x')];
            var names = ['text\ud800', '', '[]', '[\ud800x]'];
            for (var i = 0; i < keys.length; i++) {
                var key = keys[i];
                var method = { [key]() { return 1; } };
                var accessor = { get [key]() { return 2; }, set [key](value) {} };
                check(method[key], names[i]);
                var descriptor = Object.getOwnPropertyDescriptor(accessor, key);
                check(descriptor.get, 'get ' + names[i]);
                check(descriptor.set, 'set ' + names[i]);
            }
        }
        checkNames();
        var compiled = Function('return (' + checkNames.toString() + ')();');
        gc();
        compiled();
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
