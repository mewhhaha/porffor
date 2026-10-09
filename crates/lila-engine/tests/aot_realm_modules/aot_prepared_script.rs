use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn run_boolean(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("prepared source executes as a separate Wasm Script");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn entry_function_bindings_and_global_properties_share_the_same_object() {
    run_boolean(
        r#"
function check() {}
function* generator() {}
async function asynchronous() {}
async function* asyncGenerator() {}
check.same = function (a, b) { return a === b; };
var sameObjects = check === globalThis.check
  && generator === globalThis.generator
  && asynchronous === globalThis.asynchronous
  && asyncGenerator === globalThis.asyncGenerator;
var fromEval = (0, eval)("check.same(1, 1)");
var fromConstructor = Function("return check.same(2, 2);")();
globalThis.check = function replacement() { return 9; };
sameObjects && fromEval && fromConstructor && check() === 9;
"#,
    );
}

#[test]
fn script_completion_and_arbitrary_throw_identity_preserve_the_target_realm() {
    run_boolean(
        r#"
var realm = __lilaCreateRealm();
var other = realm.global;
var marker = {};
other.marker = marker;
var completion = realm.evalScript("1; {}");
var received;
try { realm.evalScript("throw marker;"); } catch (error) { received = error; }
var syntaxError;
try { realm.evalScript("let duplicate; let duplicate;"); } catch (error) { syntaxError = error; }
var missingError;
try { realm.evalScript("missingBinding;"); } catch (error) { missingError = error; }
completion === 1 && received === marker
  && syntaxError instanceof other.SyntaxError
  && missingError instanceof other.ReferenceError
  && Object.getPrototypeOf({}) === Object.prototype
  && Object.getPrototypeOf(new Error()) === Error.prototype;
"#,
    );
}

#[test]
fn callee_and_all_arguments_are_evaluated_before_the_captured_callable_runs() {
    run_boolean(
        r#"
var realm = __lilaCreateRealm();
var trace = [];
var original = realm.evalScript;
Object.defineProperty(realm, 'evalScript', {
  configurable: true,
  get() { trace.push('callee'); return original; }
});
var answer = realm.evalScript("40 + 2;", trace.push('argument'), trace.push('last'));
Object.defineProperty(realm, 'evalScript', {
  configurable: true, writable: true,
  value: function(source, value) { trace.push('replacement'); return value; }
});
var replaced = realm.evalScript("var mustNotExist = 1;", 9);
answer === 42 && replaced === 9
  && trace.join(',') === 'callee,argument,last,replacement'
  && !Object.prototype.hasOwnProperty.call(realm.global, 'mustNotExist');
"#,
    );
}

#[test]
fn repeated_script_invocations_allocate_fresh_declarations_and_retain_global_state() {
    run_boolean(
        r#"
var realm = __lilaCreateRealm();
var first = realm.evalScript("var count = (globalThis.count || 0) + 1; function fresh() { return count; } fresh;");
var second = realm.evalScript("var count = (globalThis.count || 0) + 1; function fresh() { return count; } fresh;");
first !== second && first() === 2 && second() === 2
  && realm.global.count === 2 && typeof count === 'undefined';
"#,
    );
}

#[test]
fn indirect_eval_lexicals_are_fresh_and_escaped_functions_keep_their_cells() {
    run_boolean(
        r#"
var first = eval.call(undefined, "let value = 1; function read() { return value; } read;");
var second = eval.call(undefined, "let value = 2; function read() { return value; } read;");
var strict = eval.call(undefined, "'use strict'; var value = 3; function readStrict() { return value; } readStrict;");
first() === 1 && second() === 2 && strict() === 3
  && typeof value === 'undefined' && typeof readStrict === 'undefined';
"#,
    );
}

#[test]
fn statement_lists_preserve_empty_completions_and_control_statements_supply_undefined() {
    let cases = [
        ("1; {}", "1"),
        ("2; ;", "2"),
        ("3; debugger;", "3"),
        ("4; var emptyVar;", "4"),
        ("5; var initializedVar = Object();", "5"),
        ("6; let lexicalValue = (() => {})();", "6"),
        ("7; const constantValue = {};", "7"),
        ("8; class EmptyClass {}", "8"),
        ("9; function emptyFunction() {}", "9"),
        ("10; { function optionalFunction() {} }", "10"),
        ("11; exit: { break exit; }", "11"),
        ("12; if (true) {}", "undefined"),
        ("13; if (false) {}", "undefined"),
        ("14; if (false) {} else { ; }", "undefined"),
        ("15; if (true) { 16; let branchValue; }", "16"),
        ("17; try {} catch (error) {}", "undefined"),
        ("18; try {} finally { 19; }", "undefined"),
        ("20; try { 21; } finally {}", "21"),
        ("22; try { throw 23; } catch (error) {}", "undefined"),
        (
            "24; try { throw 25; } catch (error) { 26; ; } finally {}",
            "26",
        ),
        ("27; while (false) {}", "undefined"),
        ("28; let index = 0; while (index++ < 2) { 29; {} }", "29"),
        ("30; for (var j = 0; j < 2; j++) { 31; var z; }", "31"),
        ("32; do { 33; } while (false);", "33"),
        ("34; switch (1) { case 1: 35; case 2: ; }", "35"),
        ("36; switch (9) { case 1: ; }", "undefined"),
        ("37; exit: { if (true) break exit; }", "undefined"),
        ("38; while (true) { 39; break; }", "39"),
        (
            "40; var k = 0; function next() { return k++ < 2; } while (next()) { 41; continue; }",
            "41",
        ),
        (
            "42; function update() { m++; return 'ignored'; } for (var m = 0; m < 2; update()) { 43; }",
            "43",
        ),
    ];
    let mut source = String::from("var realm = __lilaCreateRealm();\n");
    for (index, (script, expected)) in cases.iter().enumerate() {
        source.push_str(&format!(
            "if (realm.evalScript({script:?}) !== {expected}) throw new Error('completion case {index}');\n"
        ));
    }
    source.push_str("true;");
    run_boolean(&source);
}

#[test]
fn retained_indirect_eval_keeps_its_defining_realm_after_the_global_is_replaced() {
    run_boolean(
        r#"
var realm = __lilaCreateRealm();
var other = realm.global;
var original = other.eval;
var first = other.eval('globalThis');
other.eval = function() { return 1; };
var retained = original.call(undefined, 'globalThis');
first === other && retained === other && other.eval('globalThis') === 1;
"#,
    );
}
