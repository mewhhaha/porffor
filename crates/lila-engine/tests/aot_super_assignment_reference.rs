use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_super_assignment(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("super assignment failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

fn assert_pinned_source(source: &str) {
    const STA: &str = include_str!("../../../test262/vendor/test262/harness/sta.js");
    const ASSERT: &str = include_str!("../../../test262/vendor/test262/harness/assert.js");
    for strict_prelude in ["", "'use strict';\n"] {
        assert_super_assignment(&format!("{strict_prelude}{STA}\n{ASSERT}\n{source}\ntrue;"));
    }
}

#[test]
fn pinned_computed_assignment_evaluates_rhs_before_null_super_put_value() {
    assert_pinned_source(include_str!(
        "../../../test262/vendor/test262/test/language/expressions/assignment/target-super-computed-reference-null.js"
    ));
}

#[test]
fn pinned_identifier_assignment_evaluates_rhs_before_null_super_put_value() {
    assert_pinned_source(include_str!(
        "../../../test262/vendor/test262/test/language/expressions/assignment/target-super-identifier-reference-null.js"
    ));
}

#[test]
fn null_super_put_value_follows_rhs_and_precedes_property_key_coercion() {
    assert_super_assignment(
        r#"
var trace = [], marker = {}, caught;
var key = { [Symbol.toPrimitive]() { trace.push('coerce'); throw marker; } };
function rawKey() { trace.push('key'); return key; }
function rhs() { trace.push('rhs'); return 7; }
class C { static write() { super[rawKey()] = rhs(); } }
Object.setPrototypeOf(C, null);
try { C.write(); } catch (error) { caught = error; }
caught instanceof TypeError && trace.join(',') === 'key,rhs';
"#,
    );
}

#[test]
fn abrupt_rhs_keeps_its_identity_before_null_super_validation() {
    assert_super_assignment(
        r#"
var marker = {}, trace = [], caught;
var key = { [Symbol.toPrimitive]() { trace.push('coerce'); return 'x'; } };
function rhs() { trace.push('rhs'); throw marker; }
class C { static write() { super[key] = rhs(); } }
Object.setPrototypeOf(C, null);
try { C.write(); } catch (error) { caught = error; }
caught === marker && trace.join(',') === 'rhs';
"#,
    );
}

#[test]
fn rhs_prototype_changes_preserve_the_captured_super_base_and_receiver() {
    assert_super_assignment(
        r#"
var trace = [], receiver, written, nullCaught = false;
var oldBase = { set x(value) { trace.push('set'); receiver = this; written = value; } };
var newBase = { set x(value) { throw 'new base used'; } };
var object = { write() { return super.x = (Object.setPrototypeOf(object, newBase), 19); } };
Object.setPrototypeOf(object, oldBase);
var result = object.write();
class C { static write() { super.x = (Object.setPrototypeOf(C, oldBase), trace.push('rhs'), 23); } }
Object.setPrototypeOf(C, null);
try { C.write(); } catch (error) { nullCaught = error instanceof TypeError; }
result === 19 && receiver === object && written === 19 && nullCaught && trace.join(',') === 'set,rhs';
"#,
    );
}

#[test]
fn successful_super_assignment_coerces_the_saved_key_after_rhs_once() {
    assert_super_assignment(
        r#"
var trace = [], symbol = Symbol(), receiver, written;
var base = { set [symbol](value) { trace.push('set'); receiver = this; written = value; } };
var key = { [Symbol.toPrimitive](hint) { trace.push(hint); return symbol; } };
function rawKey() { trace.push('key'); return key; }
function rhs() { trace.push('rhs'); return 29; }
var object = { write() { return super[rawKey()] = rhs(); } };
Object.setPrototypeOf(object, base);
var result = object.write();
result === 29 && receiver === object && written === 29 && trace.join(',') === 'key,rhs,string,set';
"#,
    );
}

#[test]
fn null_super_compound_assignment_and_uninitialized_this_fail_before_rhs() {
    assert_super_assignment(
        r#"
var count = 0, caught = 0;
class C { static write() { super.x += (count++, 1); } }
Object.setPrototypeOf(C, null);
try { C.write(); } catch (error) { if (error instanceof TypeError) caught++; }
class B {}
class D extends B { constructor() { super[(count++, 'x')] = (count++, 1); } }
try { new D(); } catch (error) { if (error instanceof ReferenceError) caught++; }
caught === 2 && count === 0;
"#,
    );
}
