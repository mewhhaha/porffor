use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_in_order(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let outcome = Engine::new(RealmBuilder::new().build())
            .run_script(
                &source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("in operand order failed: {error}\n{source}"));
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(
            outcome.note.contains("boolean(true)"),
            "{}\n{source}",
            outcome.note
        );
    }
}

#[test]
fn in_retains_key_before_object_evaluation_and_coerces_after_both() {
    assert_in_order(
        r#"
var trace = '', target = { x: 1 }, selected = 'x';
function key() { trace += 'K'; return selected; }
function object() { trace += 'O'; selected = 'absent'; return target; }
if (!(key() in object()) || trace !== 'KO') throw 'in value retention';

trace = '';
var keyHolder = { get value() { trace += 'K'; return 'x'; } };
var objectHolder = { get value() { trace += 'O'; return target; } };
if (!(keyHolder.value in objectHolder.value) || trace !== 'KO') throw 'in GetValue order';

trace = '';
var computed = {
  [Symbol.toPrimitive](hint) {
    if (hint !== 'string') throw 'in property-key hint';
    trace += 'C';
    target.y = 2;
    return 'y';
  }
};
var proxy = new Proxy(target, {
  has(object, property) {
    trace += 'H';
    if (property !== 'y') throw 'in converted property';
    return Reflect.has(object, property);
  }
});
function computedKey() { trace += 'K'; return computed; }
function proxyObject() { trace += 'O'; return proxy; }
if (!(computedKey() in proxyObject()) || trace !== 'KOCH') throw 'in conversion and has order';

trace = '';
if (!Reflect.has(proxyObject(), computedKey()) || trace !== 'OKCH') throw 'Reflect.has argument order';
true;
"#,
    );
}

#[test]
fn in_abrupt_operands_and_object_validation_precede_key_coercion() {
    assert_in_order(
        r#"
var trace = '', marker = {}, caught;
var computed = { [Symbol.toPrimitive]() { trace += 'C'; throw marker; } };
function key() { trace += 'K'; return computed; }
function object() { trace += 'O'; return { x: 1 }; }
function primitive() { trace += 'O'; return 1; }
function failKey() { trace += 'K'; throw marker; }
function failObject() { trace += 'O'; throw marker; }

try { failKey() in object(); } catch (error) { caught = error; }
if (caught !== marker || trace !== 'K') throw 'in abrupt key evaluation';
trace = ''; caught = undefined;
try { key() in failObject(); } catch (error) { caught = error; }
if (caught !== marker || trace !== 'KO') throw 'in abrupt object evaluation';
trace = ''; caught = undefined;
try { key() in primitive(); } catch (error) { caught = error; }
if (!(caught instanceof TypeError) || caught === marker || trace !== 'KO') throw 'in object validation';
trace = ''; caught = undefined;
try { key() in object(); } catch (error) { caught = error; }
if (caught !== marker || trace !== 'KOC') throw 'in abrupt key coercion';
true;
"#,
    );
}

#[test]
fn in_suspended_operands_resume_in_expression_order() {
    assert_in_order(
        r#"
function* contains() { return (yield 'key') in (yield 'object'); }
var iterator = contains();
var first = iterator.next();
var second = iterator.next('x');
var final = iterator.next({ x: 1 });
first.value === 'key' && !first.done && second.value === 'object' && !second.done &&
  final.value === true && final.done;
"#,
    );
}
