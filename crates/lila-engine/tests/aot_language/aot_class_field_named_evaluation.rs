use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

const STA: &str = include_str!("../../../../test262/vendor/test262/harness/sta.js");
const ASSERT: &str = include_str!("../../../../test262/vendor/test262/harness/assert.js");
const PINNED_NUMERIC_FIELDS: &str =
    include_str!("../../../../test262/vendor/test262/test/staging/sm/fields/numeric-fields.js");

fn assert_names_in_both_modes(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let source = format!(
            "{directive}{STA}\n{ASSERT}\n\
             function checkName(value, expected) {{\n\
               var descriptor = Object.getOwnPropertyDescriptor(value, 'name');\n\
               assert.sameValue(value.name, expected);\n\
               assert.sameValue(descriptor.value, expected);\n\
               assert.sameValue(descriptor.writable, false);\n\
               assert.sameValue(descriptor.enumerable, false);\n\
               assert.sameValue(descriptor.configurable, true);\n\
             }}\n{source}\ntrue;"
        );
        let outcome = Engine::new(RealmBuilder::new().build())
            .run_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(60_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("class field names failed: {error}\n{source}"));
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }
}

#[test]
fn unchanged_pinned_numeric_and_bigint_fields_receive_original_key_names() {
    assert_names_in_both_modes(PINNED_NUMERIC_FIELDS);
}

#[test]
fn public_private_static_and_instance_names_keep_canonical_keys_and_descriptors() {
    assert_names_in_both_modes(
        r#"
class Fields {
  128 = class {};
  128n = class { constructor() {} };
  1e2 = class {};
  9007199254740993n = class {};
  [-0] = class {};
  literal = class {};
  #secret = class {};
  static 128 = class {};
  static 128n = class { constructor() {} };
  static #secretStatic = class {};
  secret() { return this.#secret; }
  static secret() { return this.#secretStatic; }
}
var fields = new Fields();
checkName(fields[128], '128');
checkName(fields[100], '100');
checkName(fields['9007199254740993'], '9007199254740993');
checkName(fields[0], '0');
checkName(fields.literal, 'literal');
checkName(fields.secret(), '#secret');
checkName(Fields[128], '128');
checkName(Fields.secret(), '#secretStatic');
assert.sameValue(new Fields()[128] === fields[128], false);
"#,
    );
}

#[test]
fn computed_key_coercion_is_cached_once_and_symbol_names_preserve_descriptions() {
    assert_names_in_both_modes(
        r#"
var calls = 0;
var trace = '';
var key = {
  [Symbol.toPrimitive](hint) {
    calls++;
    assert.sameValue(hint, 'string');
    trace += 'key;';
    return 128n;
  }
};
class Cached {
  [key] = class { static { trace += 'instance:' + this.name + ';'; } };
  static [key] = class { static { trace += 'static:' + this.name + ';'; } };
}
assert.sameValue(calls, 2);
assert.sameValue(trace, 'key;key;static:128;');
key[Symbol.toPrimitive] = function() { throw new Test262Error('repeated key coercion'); };
checkName(new Cached()[128], '128');
checkName(new Cached()[128], '128');
assert.sameValue(trace, 'key;key;static:128;instance:128;instance:128;');
assert.sameValue(calls, 2);
var described = Symbol('field');
var empty = Symbol('');
var absent = Symbol();
class Symbols {
  [described] = class {};
  [empty] = class {};
  [absent] = class {};
  static [Symbol.iterator] = class {};
}
var symbols = new Symbols();
checkName(symbols[described], '[field]');
checkName(symbols[empty], '[]');
checkName(symbols[absent], '');
checkName(Symbols[Symbol.iterator], '[Symbol.iterator]');
var marker = {};
var initialized = false;
var abrupt = { [Symbol.toPrimitive]() { throw marker; } };
var caught;
try { class Abrupt { [abrupt] = class { static { initialized = true; } }; } }
catch (error) { caught = error; }
assert.sameValue(caught, marker);
assert.sameValue(initialized, false);
"#,
    );
}

#[test]
fn names_precede_nested_class_effects_and_preserve_named_and_non_definition_values() {
    assert_names_in_both_modes(
        r#"
var trace = '';
class Declared { static { trace += 'declaration:' + this.name + ';'; } }
var existing = class Existing {};
class Fields {
  ['parenthesized'] = ((class { static { trace += this.name + ';'; } }));
  ['explicit'] = class Original { static { trace += this.name + ';'; } };
  ['reference'] = existing;
  ['comma'] = (0, class { static { trace += 'comma:[' + this.name + '];'; } });
  ['overridden'] = class { static name = 'replacement'; };
  ['deleted'] = class { static { assert.sameValue(this.name, 'deleted'); delete this.name; } };
  ['method'] = class { static name() { return 7; } };
  ['derived'] = class extends Declared { static { trace += this.name + ';'; } };
  static ['static'] = class { static { trace += this.name + ';'; } };
}
assert.sameValue(trace, 'declaration:Declared;static;');
var fields = new Fields();
checkName(fields.parenthesized, 'parenthesized');
checkName(fields.explicit, 'Original');
checkName(fields.reference, 'Existing');
assert.sameValue(fields.reference, existing);
checkName(fields.comma, '');
assert.sameValue(fields.overridden.name, 'replacement');
assert.sameValue(Object.hasOwn(fields.deleted, 'name'), false);
assert.sameValue(fields.method.name(), 7);
checkName(fields.derived, 'derived');
checkName(Declared, 'Declared');
assert.sameValue(trace, 'declaration:Declared;static;parenthesized;Original;comma:[];derived;');
"#,
    );
}
