use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn run_boolean(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
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
        .expect("prepared Script declarations must follow ordinary global binding rules");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn realm_scripts_share_lexical_cells_and_refresh_function_objects() {
    run_boolean(
        r#"
var realm = __lilaCreateRealm();
var other = realm.global;
var completion = realm.evalScript("let lexical = 1; var variable = 2; function read() { return lexical; } lexical;");
var first = other.read;
realm.evalScript("lexical = 3; variable = 4; function read() { return lexical; }");
var second = other.read;
completion === 1 && first !== second && first() === 3 && second() === 3
  && other.variable === 4 && typeof lexical === "undefined"
  && typeof other.lexical === "undefined"
  && Object.getPrototypeOf(second) === other.Function.prototype
  && Object.getOwnPropertyDescriptor(other, "read").configurable === false;
"#,
    );
}

#[test]
fn declaration_rejection_does_not_publish_other_names() {
    run_boolean(
        r#"
var realm = __lilaCreateRealm();
var other = realm.global;
realm.evalScript("let lexical = 1;");
var lexicalError;
try { realm.evalScript("var untouched; let lexical;"); } catch (error) { lexicalError = error; }
Object.defineProperty(other, "restricted", {value: 1, writable: true, enumerable: false, configurable: false});
var functionError;
try { realm.evalScript("var stillUntouched; function restricted() {}"); } catch (error) { functionError = error; }
Object.preventExtensions(other);
var varError;
try { realm.evalScript("var unavailable;"); } catch (error) { varError = error; }
lexicalError instanceof other.SyntaxError && functionError instanceof other.TypeError
  && varError instanceof other.TypeError
  && !Object.prototype.hasOwnProperty.call(other, "untouched")
  && !Object.prototype.hasOwnProperty.call(other, "stillUntouched")
  && !Object.prototype.hasOwnProperty.call(other, "unavailable")
  && other.restricted === 1;
"#,
    );
}

#[test]
fn indirect_eval_vars_are_configurable_and_may_be_shadowed_by_later_lexicals() {
    run_boolean(
        r#"
var realm = __lilaCreateRealm();
var other = realm.global;
other.eval("var selected = 1; function callable() { return selected; }");
var variableDescriptor = Object.getOwnPropertyDescriptor(other, "selected");
var functionDescriptor = Object.getOwnPropertyDescriptor(other, "callable");
realm.evalScript("let selected = 7; const callable = 8;");
variableDescriptor.configurable === true && functionDescriptor.configurable === true
  && realm.evalScript("selected;") === 7 && realm.evalScript("callable;") === 8
  && other.selected === 1 && typeof other.callable === "function"
  && other.callable() === 7;
"#,
    );
}

#[test]
fn indirect_eval_lexicals_and_strict_variables_remain_invocation_local() {
    run_boolean(
        r#"
var realm = __lilaCreateRealm();
var other = realm.global;
var first = other.eval("let local = 1; local;");
var second = other.eval("let local = 2; local;");
var strictResult = other.eval("'use strict'; var strictVariable = 3; function strictFunction() { return strictVariable; } strictFunction();");
first === 1 && second === 2 && strictResult === 3
  && realm.evalScript("typeof local;") === "undefined"
  && realm.evalScript("typeof strictVariable;") === "undefined"
  && realm.evalScript("typeof strictFunction;") === "undefined";
"#,
    );
}

#[test]
fn annex_b_copies_are_admitted_by_runtime_global_state() {
    run_boolean(
        r#"
var realm = __lilaCreateRealm();
var other = realm.global;
realm.evalScript("let blocked = 1;");
var completion = realm.evalScript("{ function blocked() { return 2; } blocked(); }");
Object.defineProperty(other, "existing", {value: 7, writable: false, enumerable: false, configurable: false});
realm.evalScript("{ function existing() { return 9; } }");
realm.evalScript("{ function accepted() { return 3; } }");
Object.preventExtensions(other);
realm.evalScript("{ function absent() {} }");
completion === 2 && realm.evalScript("blocked;") === 1
  && !Object.prototype.hasOwnProperty.call(other, "blocked")
  && other.existing === 7 && other.accepted() === 3
  && !Object.prototype.hasOwnProperty.call(other, "absent");
"#,
    );
}
