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
        .expect("source functions must execute in their own Global Environment");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn static_functions_use_constructor_global_bindings_and_preserve_caller_bindings() {
    run_boolean(
        r#"
var counter = 1;
let lexical = 10;
var other = __lilaCreateRealm().global;
other.counter = 100;
var foreign = other.Function("delta", "counter += delta; created = counter; return [counter, typeof lexical, globalThis, this];");
function callWithLocalShadow() {
  var counter = 999;
  return foreign(2);
}
var result = callWithLocalShadow();
var local = Function("return lexical;");
counter === 1 && lexical === 10 && local() === 10
  && other.counter === 102 && other.created === 102
  && result[0] === 102 && result[1] === "undefined"
  && result[2] === other && result[3] === other
  && Object.getPrototypeOf(result) === other.Array.prototype
  && typeof created === "undefined";
"#,
    );
}

#[test]
fn static_functions_share_global_lexical_cells_and_enforce_tdz_and_const() {
    run_boolean(
        r#"
var read = Function("return lexical;");
var write = Function("lexical = 42; return lexical;");
var tdz;
try { read(); } catch (error) { tdz = error; }
let lexical = 10;
const constant = 20;
var writes = write() === 42 && lexical === 42 && read() === 42;
var constantError;
try { Function("constant = 21;")(); } catch (error) { constantError = error; }
var typeofTdz;
try { Function("return typeof later;")(); } catch (error) { typeofTdz = error; }
let later = 1;
tdz instanceof ReferenceError && writes && constant === 20
  && constantError instanceof TypeError && typeofTdz instanceof ReferenceError
  && Function("return typeof later;")() === "number";
"#,
    );
}

#[test]
fn nested_foreign_functions_keep_their_global_environment_and_literal_prototypes() {
    run_boolean(
        r#"
var other = __lilaCreateRealm().global;
other.counter = 0;
var factory = other.Function("return function nested() { counter++; return [globalThis, {}, counter]; };");
var nested = factory();
var bound = nested.bind(null);
var first = bound();
var second = nested.call(null);
Object.getPrototypeOf(nested) === other.Function.prototype
  && Object.getPrototypeOf(bound) === other.Function.prototype
  && Object.getPrototypeOf(first) === other.Array.prototype
  && Object.getPrototypeOf(first[1]) === other.Object.prototype
  && first[0] === other && first[2] === 1
  && second[0] === other && second[2] === 2 && other.counter === 2;
"#,
    );
}

#[test]
fn resumed_foreign_generators_resolve_globals_in_their_defining_realm() {
    run_boolean(
        r#"
var marker = 1;
var other = __lilaCreateRealm().global;
other.marker = 10;
var create = other.Function("return function* () { yield marker; yield marker; return globalThis; };");
var generator = create()();
var first = generator.next();
other.marker = 20;
var second = generator.next();
var end = generator.next();
first.value === 10 && first.done === false
  && second.value === 20 && second.done === false
  && end.value === other && end.done === true && marker === 1;
"#,
    );
}

#[test]
fn foreign_global_reference_errors_use_foreign_intrinsics_and_preserve_getter_throws() {
    run_boolean(
        r#"
var other = __lilaCreateRealm().global;
var missing = other.Function("return missingGlobal;");
var strictWrite = other.Function("'use strict'; absentGlobal = 1;");
var missingError;
var writeError;
try { missing(); } catch (error) { missingError = error; }
try { strictWrite(); } catch (error) { writeError = error; }
var sentinel = {};
Object.defineProperty(other, "observable", {get: function () { throw sentinel; }});
var getterError;
try { other.Function("return observable;")(); } catch (error) { getterError = error; }
missingError instanceof other.ReferenceError
  && !(missingError instanceof ReferenceError)
  && writeError instanceof other.ReferenceError
  && getterError === sentinel && typeof absentGlobal === "undefined";
"#,
    );
}

#[test]
fn foreign_regex_class_arguments_and_function_families_use_defining_intrinsics() {
    run_boolean(
        r#"
var other = __lilaCreateRealm().global;
var create = other.Function("return [/a/g, class C {}, function () { return arguments; }, function* () {}, async function () {}, async function* () {}];");
var values = create();
var C = values[1];
var args = values[2](1);
Object.getPrototypeOf(values[0]) === other.RegExp.prototype
  && values[0].test("a") === true
  && Object.getPrototypeOf(C) === other.Function.prototype
  && Object.getPrototypeOf(C.prototype) === other.Object.prototype
  && Object.getPrototypeOf(new C()) === C.prototype
  && Object.getPrototypeOf(args) === other.Object.prototype && args[0] === 1
  && Object.getPrototypeOf(values[2]) === other.Function.prototype
  && Object.getPrototypeOf(Object.getPrototypeOf(values[3])) === other.Function.prototype
  && Object.getPrototypeOf(Object.getPrototypeOf(values[4])) === other.Function.prototype
  && Object.getPrototypeOf(Object.getPrototypeOf(values[5])) === other.Function.prototype;
"#,
    );
}

#[test]
fn foreign_runtime_conversion_errors_and_typed_array_globals_use_defining_intrinsics() {
    run_boolean(
        r#"
var other = __lilaCreateRealm().global;
var convert = other.Function("value", "return +value;");
var invoke = other.Function("value", "return value();");
var readProperty = other.Function("value", "return value.x;");
var writeProperty = other.Function("value", "'use strict'; value.x = 1;");
var conversionError;
var callError;
var readError;
var writeError;
try { convert(Symbol()); } catch (error) { conversionError = error; }
try { invoke(1); } catch (error) { callError = error; }
try { readProperty(null); } catch (error) { readError = error; }
try { writeProperty(Object.freeze({x: 0})); } catch (error) { writeError = error; }
conversionError instanceof other.TypeError && !(conversionError instanceof TypeError)
  && callError instanceof other.TypeError && !(callError instanceof TypeError)
  && readError instanceof other.TypeError && !(readError instanceof TypeError)
  && writeError instanceof other.TypeError && !(writeError instanceof TypeError)
  && other.Function("return Uint8Array;")() === other.Uint8Array;
"#,
    );
}

#[test]
fn sloppy_this_boxes_primitives_in_the_callee_realm() {
    run_boolean(
        r#"
var other = __lilaCreateRealm().global;
var foreign = other.Function("return this;");
var strict = other.Function('"use strict"; return this;');
var ordinary = {};
var boolean = foreign.call(true);
var number = foreign.call(1);
var string = foreign.call("");
var bound = foreign.bind(false);
var proxy = new Proxy(foreign, {});
boolean.constructor === other.Boolean && boolean instanceof other.Boolean
  && number.constructor === other.Number && number instanceof other.Number
  && string.constructor === other.String && string instanceof other.String
  && bound().constructor === other.Boolean
  && Reflect.apply(proxy, 2, []).constructor === other.Number
  && foreign.call(ordinary) === ordinary
  && strict.call(true) === true && strict.call(null) === null
  && foreign.call(null) === other;
"#,
    );
}

#[test]
fn derived_constructor_result_errors_use_the_restored_caller_realm() {
    run_boolean(
        r#"
var other = __lilaCreateRealm().global;
var invalid = other.eval('(class extends Object { constructor() { return null; } })');
var missing = other.eval('(class extends Object { constructor() {} })');
var explicitThis = other.eval('(class extends Object { constructor() { return this; } })');
var entryType = false;
var entryReference = false;
var bodyReference = false;
try { new invalid(); } catch (error) { entryType = error.constructor === TypeError; }
try { new missing(); } catch (error) { entryReference = error.constructor === ReferenceError; }
try { new explicitThis(); } catch (error) { bodyReference = error.constructor === other.ReferenceError; }
class EntryInvalid extends Object { constructor() { return null; } }
class EntryMissing extends Object { constructor() {} }
var foreignType = other.Function("C", "try { new C(); } catch (error) { return error.constructor === TypeError; }");
var foreignReference = other.Function("C", "try { new C(); } catch (error) { return error.constructor === ReferenceError; }");
entryType && entryReference && bodyReference
  && foreignType(EntryInvalid) && foreignReference(EntryMissing);
"#,
    );
}

#[test]
fn derived_constructor_result_selection_survives_finally_and_nested_environments() {
    run_boolean(
        r#"
var explicit = {};
class MissingWithObject extends Object { constructor() { return explicit; } }
class Initialized extends Object {
  constructor() {
    super();
    {
      let value = 12;
      this.read = () => value;
      try { return; } finally { this.done = true; }
    }
  }
}
class InvalidAfterSuper extends Object { constructor() { super(); return 1; } }
var instance = new Initialized();
var rejected = false;
try { new InvalidAfterSuper(); } catch (error) { rejected = error.constructor === TypeError; }
new MissingWithObject() === explicit && instance.read() === 12 && instance.done && rejected;
"#,
    );
}
