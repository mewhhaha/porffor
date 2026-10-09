use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn run_boolean(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("escaped direct-eval arrows retain the caller execution context");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn escaped_arrows_keep_strict_this_and_new_target_without_using_their_call_receiver() {
    run_boolean(
        r#"
function capture() { 'use strict'; return eval('() => this'); }
var primitive = capture.call(7);
function Construct() {
  this.marker = 41;
  return eval('() => [this, new.target]');
}
var read = new Construct();
var result = read.call({marker: 99});
primitive.call({}) === 7 && result[0].marker === 41 && result[1] === Construct;
"#,
    );
}

#[test]
fn nested_arrows_and_nested_direct_eval_share_the_original_context() {
    run_boolean(
        r#"
function capture() { 'use strict'; return eval('() => () => eval("this")'); }
var outer = capture.call(13);
var inner = outer.call({});
inner.call({}) === 13;
"#,
    );
}

#[test]
fn arrows_created_before_super_share_the_initialization_cell() {
    run_boolean(
        r#"
class Base { constructor() { this.base = 17; } }
class Derived extends Base {
  field = 19;
  constructor() {
    let read = eval('() => this');
    let initialize = eval('() => super()');
    let before;
    try { read(); } catch (error) { before = error instanceof ReferenceError; }
    initialize();
    this.read = read;
    this.before = before;
  }
}
var value = new Derived();
value.before && value.read() === value && value.base === 17 && value.field === 19;
"#,
    );
}

#[test]
fn super_home_objects_survive_escape_and_ordinary_functions_keep_their_own_this() {
    run_boolean(
        r#"
class Base { read() { return this.marker; } }
class Derived extends Base {
  constructor() { super(); this.marker = 23; }
  capture() { return eval('() => super.read()'); }
}
var original = new Derived();
var arrow = original.capture();
function captureOrdinary() { return eval('(function() { return this; })'); }
var ordinary = captureOrdinary();
var receiver = {};
arrow.call({marker: 99}) === 23 && ordinary.call(receiver) === receiver;
"#,
    );
}
