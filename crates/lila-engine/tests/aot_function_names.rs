use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_function_names(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = format!(
        r#"
function checkName(callable, expected) {{
  var descriptor = Object.getOwnPropertyDescriptor(callable, 'name');
  if (callable.name !== expected || descriptor.value !== expected
      || descriptor.writable !== false || descriptor.enumerable !== false
      || descriptor.configurable !== true) {{
    throw 'function name mismatch: expected ' + expected + ', received ' + callable.name;
  }}
}}
{source}
true;
"#
    );
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            &source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("function names failed: {error}\n{source}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn expression_names_preserve_explicit_names_and_descriptor_mutation() {
    assert_function_names(
        r#"
checkName(function () {}, '');
checkName(function explicit() {}, 'explicit');
checkName(() => {}, '');
checkName(async () => {}, '');
var inferred = function () {};
checkName(inferred, 'inferred');
inferred.name = 'ignored';
checkName(inferred, 'inferred');
Object.defineProperty(inferred, 'name', {value: 'changed'});
checkName(inferred, 'changed');
if (!delete inferred.name || Object.hasOwn(inferred, 'name')) throw 'delete name';
"#,
    );
}

#[test]
fn object_methods_and_accessors_name_every_symbol_description() {
    assert_function_names(
        r#"
var absent = Symbol();
var empty = Symbol('');
var described = Symbol('method');
var methods = {
  plain() {},
  [absent]() {},
  *[empty]() {},
  async [described]() {},
  async *[Symbol.iterator]() {}
};
checkName(methods.plain, 'plain');
checkName(methods[absent], '');
checkName(methods[empty], '[]');
checkName(methods[described], '[method]');
checkName(methods[Symbol.iterator], '[Symbol.iterator]');
var accessors = {
  get plain() {}, set plain(value) {},
  get [absent]() {}, set [absent](value) {},
  get [empty]() {}, set [empty](value) {},
  get [described]() {}, set [described](value) {}
};
var plain = Object.getOwnPropertyDescriptor(accessors, 'plain');
var noDescription = Object.getOwnPropertyDescriptor(accessors, absent);
var emptyDescription = Object.getOwnPropertyDescriptor(accessors, empty);
var description = Object.getOwnPropertyDescriptor(accessors, described);
checkName(plain.get, 'get plain'); checkName(plain.set, 'set plain');
checkName(noDescription.get, 'get '); checkName(noDescription.set, 'set ');
checkName(emptyDescription.get, 'get []'); checkName(emptyDescription.set, 'set []');
checkName(description.get, 'get [method]'); checkName(description.set, 'set [method]');
class Methods {
  [described]() {}
  static *[empty]() {}
  get plain() {} set plain(value) {}
  static get [absent]() {} static set [absent](value) {}
}
checkName(Methods.prototype[described], '[method]');
checkName(Methods[empty], '[]');
var classPlain = Object.getOwnPropertyDescriptor(Methods.prototype, 'plain');
var classAbsent = Object.getOwnPropertyDescriptor(Methods, absent);
checkName(classPlain.get, 'get plain'); checkName(classPlain.set, 'set plain');
checkName(classAbsent.get, 'get '); checkName(classAbsent.set, 'set ');
"#,
    );
}

#[test]
fn computed_values_infer_names_only_for_anonymous_definitions() {
    assert_function_names(
        r#"
var key = Symbol('key');
var absent = Symbol();
var values = {
  [key]: function () {},
  [absent]: () => {},
  ['generator']: (function* () {}),
  ['async']: async function () {},
  ['asyncGenerator']: async function* () {},
  ['arrow']: async () => {},
  ['explicit']: function original() {},
  ['comma']: (0, function () {})
};
checkName(values[key], '[key]');
checkName(values[absent], '');
checkName(values.generator, 'generator');
checkName(values.async, 'async');
checkName(values.asyncGenerator, 'asyncGenerator');
checkName(values.arrow, 'arrow');
checkName(values.explicit, 'original');
checkName(values.comma, '');
var shared = function original() {};
var references = {['changed']: shared, ['unchanged']: (shared)};
checkName(references.changed, 'original'); checkName(references.unchanged, 'original');
"#,
    );
}

#[test]
fn computed_class_names_exist_before_static_initialization_and_can_be_replaced() {
    assert_function_names(
        r#"
var key = Symbol('class');
var observed;
var classes = {
  [key]: class { static { observed = this.name; checkName(this, '[class]'); } },
  ['overridden']: class { static name = 'static field'; },
  ['named']: class Original {},
  ['method']: class { static name() { return 7; } }
};
checkName(classes[key], '[class]');
checkName(classes.named, 'Original');
if (observed !== '[class]' || classes.overridden.name !== 'static field'
    || classes.method.name() !== 7) throw 'class name ordering';
"#,
    );
}

#[test]
fn computed_name_coercion_runs_once_before_function_or_class_creation() {
    assert_function_names(
        r#"
var trace = '';
var key = { [Symbol.toPrimitive](hint) { trace += hint + ';'; return 'computed'; } };
var first = { [key]() {} };
checkName(first.computed, 'computed');
var second = { [key]: class { static { trace += this.name + ';'; } } };
checkName(second.computed, 'computed');
if (trace !== 'string;string;computed;') throw trace;
var marker = {};
var created = false;
var abrupt = { [Symbol.toPrimitive]() { throw marker; } };
var caught;
try { ({ [abrupt]: class { static { created = true; } } }); }
catch (error) { caught = error; }
if (caught !== marker || created) throw 'key coercion order';
"#,
    );
}

#[test]
fn computed_class_name_survives_await_without_repeating_key_coercion() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = r#"
var calls = 0;
var symbol = Symbol('suspended');
var key = { [Symbol.toPrimitive](hint) { calls++; print(hint); return symbol; } };
class Base {}
async function create() {
  return {
    [key]: class extends (await Base) {
      static {
        var descriptor = Object.getOwnPropertyDescriptor(this, 'name');
        print(this.name + ':' + descriptor.writable + ':'
          + descriptor.enumerable + ':' + descriptor.configurable);
      }
    }
  };
}
create().then(function (object) {
  print(object[symbol].name + ':' + calls);
}, function (error) { print('unexpected: ' + error); });
"#;
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("computed class names execute across await");
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        outcome.output_events,
        ["string", "[suspended]:false:false:true", "[suspended]:1"]
            .into_iter()
            .map(|line| HostOutputEvent::PrintLine(line.into()))
            .collect::<Vec<_>>()
    );
}
