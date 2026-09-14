use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_function_coercion(source: &str) {
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
        .unwrap_or_else(|error| panic!("Function coercion failed: {error}\n{source}"));
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

const PINNED_STA: &str = include_str!("../../../test262/vendor/test262/harness/sta.js");
const PINNED_FUNCTION_SLICE: &str = include_str!(
    "../../../test262/vendor/test262/test/built-ins/String/prototype/slice/S15.5.4.13_A1_T5.js"
);

#[test]
fn pinned_function_slice_preserves_sloppy_source_and_assertion() {
    assert_function_coercion(&format!("{PINNED_STA}\n{PINNED_FUNCTION_SLICE}\ntrue;"));
}

#[test]
fn pinned_function_slice_preserves_strict_source_and_assertion() {
    assert_function_coercion(&format!(
        "'use strict';\n{PINNED_STA}\n{PINNED_FUNCTION_SLICE}\ntrue;"
    ));
}

#[test]
fn function_string_conversion_observes_ordinary_hooks_and_fallback_order() {
    assert_function_coercion(
        r#"
var trace = '';
function receiver() {}
receiver.toString = function() {
  if (this !== receiver || arguments.length !== 0) throw 'toString receiver';
  trace += 's';
  return receiver;
};
receiver.valueOf = function() {
  if (this !== receiver || arguments.length !== 0) throw 'valueOf receiver';
  trace += 'v';
  return 'gnulluna';
};
var first = String.prototype.slice.call(receiver, null, 5);
var second = String.prototype.substring.call(receiver, 1, 4);
var third = String.prototype.charAt.call(receiver, 0);
first === 'gnull' && second === 'nul' && third === 'g' && trace === 'svsvsv';
"#,
    );
}

#[test]
fn function_exotic_hooks_receive_the_right_hints_in_receiver_index_order() {
    assert_function_coercion(
        r#"
var trace = '';
function receiver() {}
function start() {}
function end() {}
receiver[Symbol.toPrimitive] = function(hint) {
  if (this !== receiver) throw 'receiver';
  trace += 'r:' + hint + ';';
  return 'abcdef';
};
start[Symbol.toPrimitive] = function(hint) {
  if (this !== start) throw 'start receiver';
  trace += 's:' + hint + ';';
  return 1;
};
end[Symbol.toPrimitive] = function(hint) {
  if (this !== end) throw 'end receiver';
  trace += 'e:' + hint + ';';
  return 4;
};
receiver.toString = start.valueOf = end.valueOf = function() { throw 'ordinary hook'; };
String.prototype.slice.call(receiver, start, end) === 'bcd' &&
  trace === 'r:string;s:number;e:number;';
"#,
    );
}

#[test]
fn function_number_conversion_preserves_numeric_and_bigint_policies() {
    assert_function_coercion(
        r#"
var trace = '';
function index() {}
index.valueOf = function() { trace += 'n'; return 2; };
var slice = 'abcdef'.slice(index, 5);
function bigint() {}
bigint[Symbol.toPrimitive] = function(hint) {
  if (hint !== 'number') throw 'hint';
  trace += 'b';
  return 42n;
};
var converted = Number(bigint);
var rejected = false;
try { 'abcdef'.slice(bigint); } catch (error) { rejected = error instanceof TypeError; }
slice === 'cde' && converted === 42 && rejected && trace === 'nbb';
"#,
    );
}

#[test]
fn function_numeric_conversion_keeps_array_length_rechecks_and_iterator_close() {
    assert_function_coercion(
        r#"
var conversions = 0;
function length() {}
length.valueOf = function() { conversions++; return 2; };
var values = [10, 20, 30];
Object.defineProperty(values, 'length', { value: length });
var marker = Symbol('coercion');
var trace = '';
function limit() {}
limit.valueOf = function() { trace += 'v'; throw marker; };
var iterator = {
  next() { throw 'unexpected next'; },
  return() { trace += 'r'; return {}; }
};
var caught;
try { Iterator.prototype.take.call(iterator, limit); } catch (error) { caught = error; }
values.length === 2 && values[1] === 20 && !(2 in values) && conversions === 2 &&
  caught === marker && trace === 'vr';
"#,
    );
}

#[test]
fn function_conversion_abrupt_completions_preserve_identity_and_stop_later_hooks() {
    assert_function_coercion(
        r#"
var marker = Symbol('hook');
var trace = '';
function receiver() {}
Object.defineProperty(receiver, Symbol.toPrimitive, {
  get() { trace += 'g'; throw marker; }
});
function index() {}
index.valueOf = function() { trace += 'i'; return 1; };
var received;
try { String.prototype.slice.call(receiver, index); }
catch (error) { received = error; }
finally { trace += 'f'; }
function numeric() {}
numeric.valueOf = function() { trace += 'v'; throw marker; };
var numberError;
try { Number(numeric); } catch (error) { numberError = error; }
var lengthError;
var values = [1, 2];
try { Object.defineProperty(values, 'length', { value: numeric }); }
catch (error) { lengthError = error; }
received === marker && numberError === marker && lengthError === marker &&
  values.length === 2 && trace === 'gfvv';
"#,
    );
}

#[test]
fn function_conversion_generated_errors_use_the_builtin_or_source_realm() {
    assert_function_coercion(
        r#"
var realm = __lilaCreateRealm();
var other = realm.global;
function nonprimitive() {}
nonprimitive[Symbol.toPrimitive] = function() { return nonprimitive; };
function symbolic() {}
symbolic[Symbol.toPrimitive] = function() { return Symbol('value'); };
var caught = 0;
try { other.String.prototype.slice.call(nonprimitive); }
catch (error) { if (Object.getPrototypeOf(error) === other.TypeError.prototype) caught++; }
try { other.String.prototype.slice.call(symbolic); }
catch (error) { if (Object.getPrototypeOf(error) === other.TypeError.prototype) caught++; }
try { other.String.prototype.slice.call('abc', nonprimitive); }
catch (error) { if (Object.getPrototypeOf(error) === other.TypeError.prototype) caught++; }
try { other.Number(nonprimitive); }
catch (error) { if (Object.getPrototypeOf(error) === other.TypeError.prototype) caught++; }
var revoked = Proxy.revocable({}, {});
revoked.revoke();
function inherited() {}
Object.setPrototypeOf(inherited, revoked.proxy);
try { other.String.prototype.slice.call(inherited); }
catch (error) { if (Object.getPrototypeOf(error) === other.TypeError.prototype) caught++; }
var sourceRealm = realm.evalScript(`
  function value() {}
  value[Symbol.toPrimitive] = function() { return value; };
  var caught = false;
  try { \`\${value}\`; } catch (error) { caught = error instanceof TypeError; }
  caught;
`);
caught === 5 && sourceRealm;
"#,
    );
}

#[test]
fn intrinsic_function_source_remains_independent_of_coercion_hooks() {
    assert_function_coercion(
        r#"
function receiver() {}
var source = Function.prototype.toString.call(receiver);
receiver[Symbol.toPrimitive] = function(hint) {
  if (hint !== 'string') throw 'hint';
  return 'custom';
};
var empty = Function();
String.prototype.slice.call(receiver, 0, 3) === 'cus' &&
  Function.prototype.toString.call(receiver) === source &&
  String.prototype.slice.call(empty, 0, 5) === 'funct';
"#,
    );
}

#[test]
fn function_array_elements_observe_hooks_and_preserve_thrown_values() {
    assert_function_coercion(
        r#"
var trace = '';
function element() {}
element[Symbol.toPrimitive] = function(hint) {
  if (hint !== 'string' || this !== element) throw 'element hint';
  trace += 'e';
  return 'converted';
};
var converted = String([element, element]);
var marker = Symbol('array element');
element[Symbol.toPrimitive] = function() { throw marker; };
var caught;
function later() {}
later[Symbol.toPrimitive] = function() { trace += 'later'; return 'later'; };
try { String([element, later]); } catch (error) { caught = error; }
converted === 'converted,converted' && trace === 'ee' && caught === marker;
"#,
    );
}

#[test]
fn number_preserves_constructor_bigint_conversion_after_function_hooks() {
    assert_function_coercion(
        r#"
var conversions = 0;
function value() {}
value[Symbol.toPrimitive] = function(hint) {
  if (hint !== 'number' || this !== value) throw 'conversion context';
  conversions++;
  return 9223372036854775808n;
};
var constructor = Number;
var direct = Number(value);
var indirect = constructor(value);
var boxed = new Number(value);
var object = { [Symbol.toPrimitive]() { return 42n; } };
var objectNumber = Number(object);
var rejected = false;
try { +value; } catch (error) { rejected = error instanceof TypeError; }
direct === 9223372036854775808 && indirect === direct && boxed.valueOf() === direct &&
objectNumber === 42 && rejected && conversions === 4;
"#,
    );
}

#[test]
fn foreign_number_constructor_owns_generated_errors_and_preserves_hook_throws() {
    assert_function_coercion(
        r#"
var other = __lilaCreateRealm().global;
function nonprimitive() {}
nonprimitive[Symbol.toPrimitive] = function() { return nonprimitive; };
var rejected = 0;
try { other.Number(nonprimitive); }
catch (error) { if (Object.getPrototypeOf(error) === other.TypeError.prototype) rejected++; }
try { other.Number(Symbol('number')); }
catch (error) { if (Object.getPrototypeOf(error) === other.TypeError.prototype) rejected++; }
var marker = Symbol('hook');
function throwing() {}
throwing[Symbol.toPrimitive] = function() { throw marker; };
var caught;
try { other.Number(throwing); } catch (error) { caught = error; }
function bigint() {}
bigint[Symbol.toPrimitive] = function() { return 42n; };
rejected === 2 && caught === marker && other.Number(bigint) === 42;
"#,
    );
}

#[test]
fn revoked_prototype_reads_reject_in_the_conversion_execution_realm() {
    assert_function_coercion(
        r#"
var other = __lilaCreateRealm().global;
var revoked = Proxy.revocable({}, {});
revoked.revoke();
function value() {}
Object.setPrototypeOf(value, revoked.proxy);
var object = Object.create(revoked.proxy);
var rejected = 0;
try { other.String.prototype.slice.call(value); }
catch (error) { if (Object.getPrototypeOf(error) === other.TypeError.prototype) rejected++; }
try { other.Number(value); }
catch (error) { if (Object.getPrototypeOf(error) === other.TypeError.prototype) rejected++; }
try { other.String.prototype.slice.call(object); }
catch (error) { if (Object.getPrototypeOf(error) === other.TypeError.prototype) rejected++; }
try { value[Symbol.toPrimitive]; }
catch (error) { if (Object.getPrototypeOf(error) === TypeError.prototype) rejected++; }
rejected === 4;
"#,
    );
}
