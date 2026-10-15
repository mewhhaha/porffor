use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_function_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
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
        .expect("prepared Function source must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion
    );
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "source:\n{source}");
}

#[test]
fn prepared_global_constructor_hints_run_unmodified_generator_runtime_in_both_modes() {
    const STA: &str = include_str!("../../../../test262/vendor/test262/harness/sta.js");
    const ASSERT: &str = include_str!("../../../../test262/vendor/test262/harness/assert.js");
    const COMPARE: &str =
        include_str!("../../../../test262/vendor/test262/harness/compareArray.js");
    const SOURCE: &str =
        include_str!("../../../../test262/vendor/test262/test/staging/sm/generators/runtime.js");
    for prefix in ["", "'use strict';"] {
        assert_function_trace(
            &format!("{prefix}{STA}\n{ASSERT}\n{COMPARE}\n{SOURCE}"),
            &[],
        );
    }
}

#[test]
fn prepared_global_constructor_hints_keep_callee_conversion_getter_and_throw_order_live() {
    let source = r#"
var Factory = (function* () {}).constructor;
var original = Factory;
var trace = '';
var marker = {};
function replacement(argument) { trace += 'replacement:' + typeof argument + ';'; return 32; }
var run = Function("return Factory({ toString: function () { trace += 'convert;'; Factory = replacement; return 'yield 31;'; } });");
print(run()().next().value);
print(run());
print(trace);
Factory = original;
trace = '';
var ordered = Function("return Factory({ toString: function () { trace += 'param;'; return 'value'; } }, { toString: function () { trace += 'body;'; if (marker) throw marker; return 'yield value;'; } });");
try { ordered(); throw 'missing conversion throw'; }
catch (error) { print(error === marker); }
print(trace);
trace = '';
var getter = Function("return Factory({ get toString() { trace += 'get;'; throw marker; } });");
try { getter(); throw 'missing getter throw'; }
catch (error) { print(error === marker); }
print(trace);
"#;
    for prefix in ["", "'use strict';"] {
        assert_function_trace(
            &format!("{prefix}{source}"),
            &[
                "31",
                "32",
                "convert;replacement:object;",
                "true",
                "param;body;",
                "true",
                "get;",
            ],
        );
    }
}

#[test]
fn prepared_global_constructor_hints_keep_foreign_globals_realms_and_replacements_live() {
    let source = r#"
var Factory = (function* () {}).constructor;
var foreign = __lilaCreateRealm().global;
var source = foreign.Function('return function* () {};')();
var foreignFactory = Object.getPrototypeOf(source).constructor;
foreign.Factory = foreignFactory;
var run = foreign.Function("return Factory('yield globalThis;');");
var created = run();
print(created().next().value === foreign);
print(Object.getPrototypeOf(created) === foreignFactory.prototype);
foreign.Factory = function (body) { return body; };
print(run());
var shadowed = foreign.Function('Factory', "return Factory('yield 44;');");
print(shadowed(function (body) { return body; }));
var withObject = foreign.Function('object', "with (object) { return Factory('yield 45;'); }");
print(withObject({ Factory: function (body) { return body; } }));
"#;
    for prefix in ["", "'use strict';"] {
        assert_function_trace(
            &format!("{prefix}{source}"),
            &[
                "true",
                "true",
                "yield globalThis;",
                "yield 44;",
                "yield 45;",
            ],
        );
    }
}

#[test]
fn prepared_global_constructor_hints_keep_unknown_sources_as_typed_rejections() {
    use lila_ir::{DynamicFunctionKind, DynamicSourceRuntimeOperation};
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let error = Engine::new(RealmBuilder::new().build())
        .run_script(
            r#"var Factory = (function* () {}).constructor;
var run = Function("return Factory('yield 1;' + '/*' + Math.random() + '*/');");
try { run(); } catch (error) { print('caught'); }"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect_err("unprepared source remains an uncatchable capability rejection");
    assert_eq!(
        error.runtime_dynamic_source_operations(),
        vec![DynamicSourceRuntimeOperation::Function(
            DynamicFunctionKind::Generator
        )],
        "{error}"
    );
    assert!(error.parse_diagnostic().is_none(), "{error}");
    assert!(error.ir_diagnostic().is_none(), "{error}");
}

#[test]
fn finite_constructor_arrays_and_iterators_keep_all_four_execution_protocols() {
    let definitions = r#"
var Ordinary = Function;
var Generator = (function* () {}).constructor;
var Async = (async function () {}).constructor;
var AsyncGenerator = (async function* () {}).constructor;
var constructors = [Ordinary, Generator, Async, AsyncGenerator];
var iterable = { [Symbol.iterator]() {
  var index = 0;
  return { next() { return { done: index >= 4, value: constructors[index++] }; } };
} };
"#;
    for prefix in ["", "'use strict';"] {
        for iterable in ["constructors", "iterable"] {
            assert_function_trace(
                &format!(
                    r#"{prefix}{definitions}
var created = [];
for (const constructor of {iterable}) {{
  created.push(constructor('value', 'return value;'));
  try {{ constructor('#!\n_'); throw 'missing SyntaxError'; }}
  catch (error) {{ if (!(error instanceof SyntaxError)) throw error; }}
}}
print(created[0](41));
print(created[1](42).next().value);
created[2](43).then(function (value) {{ print(value); }});
created[3](44).next().then(function (result) {{ print(result.value); }});
"#
                ),
                &["41", "42", "43", "44"],
            );
        }
    }
}

#[test]
fn resolved_prototypes_keep_live_constructor_getters_replacements_and_abrupt_completion() {
    let source = r#"
function* source() {}
var read = Object.getPrototypeOf;
var prototype = read(source);
var constructor = prototype.constructor;
print(constructor('yield 31;')().next().value);
var descriptor = Object.getOwnPropertyDescriptor(prototype, 'constructor');
Object.defineProperty(prototype, 'constructor', {
  configurable: true,
  get: function () {
    print('get');
    return function (body) { print(body); return 32; };
  }
});
var replacement = prototype.constructor;
print(replacement('yield }'));
var marker = {};
Object.defineProperty(prototype, 'constructor', {
  configurable: true,
  get: function () { print('throw'); throw marker; }
});
var argument = { toString: function () { print('wrong conversion'); return 'yield 33;'; } };
try { prototype.constructor(argument); throw 'missing getter throw'; }
catch (error) { print(error === marker); }
Object.defineProperty(prototype, 'constructor', descriptor);
var restored = prototype.constructor;
var first = { toString: function () { print('first'); return 'value'; } };
var second = { toString: function () { print('second'); throw marker; } };
try { restored(first, second, 'yield value;'); throw 'missing conversion throw'; }
catch (error) { print(error === marker); }
"#;
    for prefix in ["", "'use strict';"] {
        assert_function_trace(
            &format!("{prefix}{source}"),
            &[
                "31", "get", "yield }", "32", "throw", "true", "first", "second", "true",
            ],
        );
    }
}

#[test]
fn constructor_values_keep_defining_realms_after_array_iteration() {
    let source = r#"
var foreign = __lilaCreateRealm().global;
var constructors = [Function, foreign.Function];
var index = 0;
for (const constructor of constructors) {
  var realm = index++ === 0 ? globalThis : foreign;
  var created = constructor('return globalThis;');
  print(created() === realm);
  print(Object.getPrototypeOf(created) === realm.Function.prototype);
  try { constructor('return }'); throw 'missing SyntaxError'; }
  catch (error) { print(Object.getPrototypeOf(error) === realm.SyntaxError.prototype); }
}
"#;
    for prefix in ["", "'use strict';"] {
        assert_function_trace(&format!("{prefix}{source}"), &["true"; 6]);
    }
}

#[test]
fn synthetic_subclasses_run_unchanged_pinned_function_and_generator_fixtures_in_both_modes() {
    const STA: &str = include_str!("../../../../test262/vendor/test262/harness/sta.js");
    const ASSERT: &str = include_str!("../../../../test262/vendor/test262/harness/assert.js");
    const PROPERTY: &str =
        include_str!("../../../../test262/vendor/test262/harness/propertyHelper.js");
    for source in [
        include_str!("../../../../test262/vendor/test262/test/language/statements/class/subclass/builtin-objects/Function/instance-length.js"),
        include_str!("../../../../test262/vendor/test262/test/language/statements/class/subclass/builtin-objects/Function/instance-name.js"),
        include_str!("../../../../test262/vendor/test262/test/language/statements/class/subclass/builtin-objects/Function/regular-subclassing.js"),
        include_str!("../../../../test262/vendor/test262/test/language/statements/class/subclass/builtin-objects/GeneratorFunction/instance-length.js"),
        include_str!("../../../../test262/vendor/test262/test/language/statements/class/subclass/builtin-objects/GeneratorFunction/instance-name.js"),
        include_str!("../../../../test262/vendor/test262/test/language/statements/class/subclass/builtin-objects/GeneratorFunction/instance-prototype.js"),
        include_str!("../../../../test262/vendor/test262/test/language/statements/class/subclass/builtin-objects/GeneratorFunction/regular-subclassing.js"),
    ] {
        for prefix in ["", "'use strict';"] {
            assert_function_trace(&format!("{prefix}{STA}\n{ASSERT}\n{PROPERTY}\n{source}"), &[]);
        }
    }
}

#[test]
fn synthetic_subclass_dispatch_keeps_super_constructor_callee_conversion_and_new_target_live() {
    let source = r#"
class Created extends Function {}
var created = new Created('value', 'return value + 1;');
print(created(40));
print(Object.getPrototypeOf(created) === Created.prototype);
var marker = {};
var holder = { Created: Created };
Object.defineProperty(holder, 'Created', {
  get: function () { print('callee'); throw marker; }
});
try { new holder.Created((print('wrong argument'), 'return 0;')); }
catch (error) { print(error === marker); }
var body = { toString: function () { print('convert'); return 'return 42;'; } };
var target = new Proxy(function Target() {}, {
  get: function (object, key) {
    if (key === 'prototype') { print('prototype'); return marker; }
    return Reflect.get(object, key);
  }
});
var reflected = Reflect.construct(Created, [body], target);
print(reflected());
print(Object.getPrototypeOf(reflected) === marker);
var failedTarget = new Proxy(function Target() {}, {
  get: function (object, key) {
    if (key === 'prototype') { print('prototype throw'); throw marker; }
    return Reflect.get(object, key);
  }
});
try { Reflect.construct(Created, ['return 42;'], failedTarget); }
catch (error) { print(error === marker); }
Object.setPrototypeOf(Created, function Replacement(body) {
  print('replacement');
  print(body);
  return marker;
});
print(new Created('return }') === marker);
"#;
    for prefix in ["", "'use strict';"] {
        assert_function_trace(
            &format!("{prefix}{source}"),
            &[
                "41",
                "true",
                "callee",
                "true",
                "convert",
                "prototype",
                "42",
                "true",
                "prototype throw",
                "true",
                "replacement",
                "return }",
                "true",
            ],
        );
    }
}

#[test]
fn synthetic_subclasses_keep_active_constructor_realm_and_new_target_prototype_separate() {
    let source = r#"
var foreign = __lilaCreateRealm().global;
class Created extends foreign.Function {}
var created = new Created('return globalThis;');
print(created() === foreign);
print(Object.getPrototypeOf(created) === Created.prototype);
var other = {};
var target = new Proxy(function Target() {}, {
  get: function (object, key) {
    if (key === 'prototype') { return other; }
    return Reflect.get(object, key);
  }
});
var reflected = Reflect.construct(Created, ['return globalThis;'], target);
print(reflected() === foreign);
print(Object.getPrototypeOf(reflected) === other);
try { new Created('return }'); throw 'missing SyntaxError'; }
catch (error) { print(Object.getPrototypeOf(error) === foreign.SyntaxError.prototype); }
"#;
    for prefix in ["", "'use strict';"] {
        assert_function_trace(&format!("{prefix}{source}"), &["true"; 5]);
    }
}

#[test]
fn synthetic_subclasses_keep_runtime_source_capability_rejections_typed() {
    use lila_ir::{DynamicFunctionKind, DynamicSourceRuntimeOperation};

    for (intrinsic, body, kind) in [
        ("Function", "return 1;", DynamicFunctionKind::Ordinary),
        (
            "Object.getPrototypeOf(function* () {}).constructor",
            "yield 1;",
            DynamicFunctionKind::Generator,
        ),
    ] {
        lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
        let error = Engine::new(RealmBuilder::new().build())
            .run_script(
                &format!("var Base = {intrinsic}; class Created extends Base {{}} try {{ new Created({body:?} + '/*' + Math.random() + '*/'); }} catch (error) {{ print('caught'); }}"),
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect_err("unprepared source remains a host capability rejection");
        assert_eq!(
            error.runtime_dynamic_source_operations(),
            vec![DynamicSourceRuntimeOperation::Function(kind)],
            "{error}"
        );
        assert!(error.parse_diagnostic().is_none(), "{error}");
        assert!(error.ir_diagnostic().is_none(), "{error}");
    }
}
