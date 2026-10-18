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

#[test]
fn repeated_distinct_eval_sources_do_not_consume_one_store_instance_per_invocation() {
    succeeds(
        r#"
        var total = 0;
        for (var i = 0; i < 11000; i++) {
            total += eval('/* source ' + i + ' */ 17;');
        }
        if (total !== 187000) throw new Error('every eval must execute');
        1;
    "#,
    );
}

#[test]
fn saved_native_functions_keep_their_own_literals_regex_data_and_sources_across_gc() {
    let engine = Engine::new(RealmBuilder::new().build());
    let observed = engine
        .observe_script(
            r#"
        var functions = [];
        for (var i = 0; i < 128; i++) {
            var token = 'payload_' + ('000' + i).slice(-3);
            functions.push(Function('return function(){ return /' + token + '/; };')());
        }
        gc();
        for (var i = 127; i >= 0; i--) {
            var token = 'payload_' + ('000' + i).slice(-3);
            var first = functions[i](), second = functions[i]();
            if (first === second || first.source !== token || second.source !== token) throw 1;
            if (functions[i].toString().indexOf(token) < 0) throw 2;
            first.lastIndex = 7;
            if (second.lastIndex !== 0) throw 3;
            gc();
        }
        1;
    "#,
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

#[test]
fn native_reentry_preserves_outer_and_inner_source_owners() {
    succeeds(
        r#"
        var first = Function('return function(callback) { if (callback) callback(); return "first-payload"; };')();
        var second = Function('return function() { return "other-payload"; };')();
        for (var i = 0; i < 32; i++) {
            if (first(function() { if (second() !== 'other-payload') throw 1; }) !== 'first-payload') throw 2;
        }
        1;
    "#,
    );
}

#[test]
fn shared_native_bodies_keep_each_function_name_source_and_self_binding() {
    let engine = Engine::new(RealmBuilder::new().build());
    let observed = engine
        .observe_script(
            r#"
        var functions = [];
        for (var i = 0; i < 64; i++) {
            var name = 'owned_' + i;
            functions.push(Function('return function ' + name + '(){ return ' + name + '; };')());
        }
        gc();
        for (var i = 63; i >= 0; i--) {
            var name = 'owned_' + i, fn = functions[i];
            if (fn.name !== name || fn() !== fn) throw 1;
            if (fn.toString() !== 'function ' + name + '(){ return ' + name + '; }') throw 2;
        }
        1;
    "#,
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
