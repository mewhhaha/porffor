use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, RealmBuilder, RunOptions, WasmExecutionFailureKind,
};

const RESTRICTED_GLOBAL_FIXTURE: &str = include_str!(
    "../../../../test262/vendor/test262/test/language/global-code/decl-lex-restricted-global.js"
);

fn engine() -> Engine {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    Engine::new(RealmBuilder::new().build())
}

fn options() -> CompileOptions {
    CompileOptions {
        host_surface_policy: HostSurfacePolicy::Test262,
        ..CompileOptions::default()
    }
}

fn execution() -> RunOptions {
    RunOptions {
        backend: ExecutionBackend::WasmAot,
        timeout_ms: Some(30_000),
        ..RunOptions::default()
    }
}

fn script_modes(source: &str) -> [String; 2] {
    [source.to_owned(), format!("'use strict';\n{source}")]
}

fn assert_boolean_script(source: &str) {
    for script in script_modes(source) {
        let observed = engine()
            .observe_script(&script, options(), execution())
            .expect("accepted declaration instantiation evaluates through Wasm");
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{script}\n{}",
            observed.note
        );
        assert!(observed.output_events.is_empty());
    }
}

#[test]
fn pinned_restricted_global_is_an_ordinary_runtime_syntax_error_in_both_script_modes() {
    for script in script_modes(RESTRICTED_GLOBAL_FIXTURE) {
        let failure = engine()
            .run_script(&script, options(), execution())
            .expect_err("the pinned runtime-negative fixture must reject global instantiation");
        assert_eq!(
            failure.wasm_execution_failure_kind(),
            Some(WasmExecutionFailureKind::JavaScriptException)
        );
        assert_eq!(
            failure.wasm_javascript_exception_constructor_name(),
            Some("SyntaxError")
        );
    }
}

#[test]
fn scalar_restricted_lexicals_preserve_intrinsic_exception_constructor_names() {
    for source in ["const NaN = 0;", "let Infinity;", "class Infinity {}"] {
        for script in script_modes(source) {
            let failure = engine()
                .run_script(&script, options(), execution())
                .expect_err("restricted global admission rejects before the body");
            assert_eq!(
                failure.wasm_execution_failure_kind(),
                Some(WasmExecutionFailureKind::JavaScriptException),
                "{script}\n{failure}"
            );
            assert_eq!(
                failure.wasm_javascript_exception_constructor_name(),
                Some("SyntaxError"),
                "{script}\n{failure}"
            );
        }
    }
}

#[test]
fn restricted_let_const_and_class_reject_before_any_body_or_initializer_effect() {
    for source in [
        "print('body'); let undefined = print('initializer');",
        "print('body'); const NaN = (print('initializer'), 1);",
        "print('body'); class Infinity { static { print('initializer'); } }",
    ] {
        for script in script_modes(source) {
            let observed = engine()
                .observe_script(&script, options(), execution())
                .expect("instantiation failure is an ordinary observed JavaScript throw");
            assert_eq!(
                observed.completion,
                ObservedCompletion::Throw(ObservedJsValue::Object)
            );
            assert!(
                observed.output_events.is_empty(),
                "{script}\n{:?}",
                observed.output_events
            );
        }
    }
}

#[test]
fn fresh_names_configurable_builtin_shadows_and_local_restricted_names_are_accepted() {
    assert_boolean_script(
        r#"
var undefined;
var NaN;
var Infinity;
let Array = 7;
const Math = 9;
let safe = 11;
class SafeClass {}
function scoped() {
  let undefined = 1;
  const NaN = 2;
  class Infinity {}
  return undefined === 1 && NaN === 2 && typeof Infinity === 'function';
}
var blockAccepted;
{
  let undefined = 3;
  const NaN = 4;
  let Infinity = 5;
  blockAccepted = undefined === 3 && NaN === 4 && Infinity === 5;
}
Array === 7 && Math === 9 && safe === 11 && typeof SafeClass === 'function'
  && scoped() && blockAccepted
  && typeof globalThis.Array === 'function' && typeof globalThis.Math === 'object'
  && Object.getOwnPropertyDescriptor(globalThis, 'Array').configurable === true
  && Object.getOwnPropertyDescriptor(globalThis, 'Math').configurable === true
  && Object.getOwnPropertyDescriptor(globalThis, 'undefined').configurable === false
  && Object.getOwnPropertyDescriptor(globalThis, 'NaN').configurable === false
  && Object.getOwnPropertyDescriptor(globalThis, 'Infinity').configurable === false
  && globalThis.undefined === void 0 && globalThis.NaN !== globalThis.NaN
  && globalThis.Infinity === 1 / 0;
"#,
    );
}

#[test]
fn prepared_realm_scripts_keep_descriptor_admission_and_intrinsic_error_identity() {
    assert_boolean_script(
        r#"
var realm = __lilaCreateRealm();
var other = realm.global;
var syntaxErrorPrototype = other.SyntaxError.prototype;
other.SyntaxError = function () { throw new Error('public constructor must not be called'); };
other.effects = 0;
Object.defineProperty(other, 'fixed', {value: 1, configurable: false});
Object.defineProperty(other, 'open', {value: 2, configurable: true});
var builtinError;
try {
  realm.evalScript("var untouched; function fresh() {} let undefined = (globalThis.effects++, 1);");
} catch (error) { builtinError = error; }
var descriptorError;
try {
  realm.evalScript("var alsoUntouched; let fixed = (globalThis.effects++, 3);");
} catch (error) { descriptorError = error; }
var accepted = realm.evalScript("let open = 7; let safeName = 8; open + safeName;");
Object.getPrototypeOf(builtinError) === syntaxErrorPrototype
  && Object.getPrototypeOf(descriptorError) === syntaxErrorPrototype
  && other.effects === 0
  && !Object.prototype.hasOwnProperty.call(other, 'untouched')
  && !Object.prototype.hasOwnProperty.call(other, 'fresh')
  && !Object.prototype.hasOwnProperty.call(other, 'alsoUntouched')
  && other.fixed === 1 && other.open === 2 && accepted === 15
  && realm.evalScript("open;") === 7;
"#,
    );
}

#[test]
fn direct_and_indirect_eval_restricted_names_remain_invocation_local() {
    assert_boolean_script(
        r#"
var direct = eval("let undefined = 1; const NaN = 2; class Infinity {} undefined === 1 && NaN === 2 && typeof Infinity === 'function';");
var indirect = (0, eval)("let undefined = 3; let evalOnly = 4; undefined + evalOnly;");
var realm = __lilaCreateRealm();
var foreign = realm.global.eval("const undefined = 5; undefined;");
direct === true && indirect === 7 && foreign === 5
  && typeof evalOnly === 'undefined'
  && realm.evalScript("typeof evalOnly;") === 'undefined'
  && globalThis.undefined === void 0;
"#,
    );
}

#[test]
fn module_lexicals_may_use_restricted_global_property_names() {
    let observed = engine()
        .observe_module(
            "let undefined = 1; const NaN = 2; class Infinity {} if (undefined !== 1 || NaN !== 2 || typeof Infinity !== 'function') throw new Error('Module lexical scope');",
            options(),
            execution(),
        )
        .expect("Module bindings are independent of restricted global object properties");
    assert_eq!(
        observed.completion,
        ObservedCompletion::Normal(ObservedJsValue::Undefined)
    );
    assert!(observed.output_events.is_empty());
}
