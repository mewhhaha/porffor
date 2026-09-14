use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_completion(source: &str, expected: &str) {
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
        .expect("with-body variable declarations must execute through Wasm AOT");
    assert!(
        outcome.note.contains(expected),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn with_body_var_initializer_writes_its_hoisted_global() {
    assert_completion(
        r#"
var scope = {};
with (scope) { var declared = 17; }
var descriptor = Object.getOwnPropertyDescriptor(globalThis, 'declared');
declared === 17 && descriptor.value === 17 && descriptor.writable &&
  descriptor.enumerable && !descriptor.configurable && !('declared' in scope);
"#,
        "boolean(true)",
    );
}

#[test]
fn with_body_function_vars_are_local_and_can_be_captured() {
    assert_completion(
        r#"
function owner(seed) {
  with ({}) { var captured = seed; }
  return function read() { return captured; };
}
var first = owner(17), second = owner(29);
first() === 17 && second() === 29 && !('captured' in globalThis);
"#,
        "boolean(true)",
    );
}

#[test]
fn nested_with_control_flow_keeps_all_var_bindings_in_the_same_owner() {
    assert_completion(
        r#"
function owner() {
  with ({}) {
    for (var loop = 0; loop < 2; loop++) {
      with ({}) {
        try { var direct = loop + 1; } finally { var final = direct + 10; }
      }
    }
  }
  return loop === 2 && direct === 2 && final === 12;
}
owner();
"#,
        "boolean(true)",
    );
}

#[test]
fn unexecuted_with_bodies_still_instantiate_global_and_function_vars() {
    assert_completion(
        r#"
if (false) { with ({}) { var globalPending = 17; } }
function owner() {
  var before = localPending;
  if (false) { with ({}) { var localPending = 29; } }
  return before === undefined && localPending === undefined;
}
var descriptor = Object.getOwnPropertyDescriptor(globalThis, 'globalPending');
owner() && descriptor.value === undefined && !descriptor.configurable &&
  !('localPending' in globalThis);
"#,
        "boolean(true)",
    );
}

#[test]
fn with_entry_and_initializer_getters_run_in_source_order_once() {
    assert_completion(
        r#"
var trace = [];
var holder = {
  get scope() {
    trace.push('enter');
    return { get initializer() { trace.push('initializer'); return 17; } };
  }
};
with (holder.scope) {
  var first = initializer;
  var second = (trace.push('second'), first + 1);
}
first === 17 && second === 18 && trace.join(',') === 'enter,initializer,second';
"#,
        "boolean(true)",
    );
}

#[test]
fn with_var_initializers_write_selected_setters_without_reading_them() {
    assert_completion(
        r#"
var writes = [], selected = 0;
var scope = {
  get selected() { throw 'initializer must not GetValue'; },
  set selected(value) { writes.push(value); }
};
with (scope) { var selected = 5; }
function owner() {
  var selected = 11;
  with (scope) { var selected = 7; }
  return selected;
}
owner() === 11 && selected === 0 && writes.join(',') === '5,7';
"#,
        "boolean(true)",
    );
}

#[test]
fn with_var_reference_survives_initializer_property_and_unscopables_changes() {
    assert_completion(
        r#"
var scope = { selected: 1 };
with (scope) { var selected = delete scope.selected; }
if (scope.selected !== true || selected !== undefined) throw 'deleted selected property';
var added = 0;
with (scope) { var added = (scope.added = 99, 7); }
if (scope.added !== 99 || added !== 7) throw 'new property changed selected reference';
var blocked = 0;
scope.blocked = 1;
scope[Symbol.unscopables] = { blocked: false };
with (scope) { var blocked = (scope[Symbol.unscopables].blocked = true, 9); }
scope.blocked === 9 && blocked === 0;
"#,
        "boolean(true)",
    );
}

#[test]
fn abrupt_with_var_setter_stops_later_declarators_after_initializer_effects() {
    assert_completion(
        r#"
var marker = {}, trace = [], selected = 0;
var scope = { set selected(value) { trace.push('set:' + value); throw marker; } };
var caught;
try {
  with (scope) {
    var selected = (trace.push('rhs'), 5), later = (trace.push('later'), 7);
  }
} catch (error) { caught = error; }
caught === marker && trace.join(',') === 'rhs,set:5' &&
  selected === 0 && later === undefined;
"#,
        "boolean(true)",
    );
}

#[test]
fn classic_for_var_initializers_use_with_references_before_loop_tests() {
    assert_completion(
        r#"
var selected = 0, trace = [];
var scope = { set selected(value) { trace.push('set:' + value); } };
with (scope) {
  for (var selected = (trace.push('rhs'), 5), later = 7;
       (trace.push('test'), false);) { throw 'unreachable loop body'; }
}
selected === 0 && later === 7 && trace.join(',') === 'rhs,set:5,test';
"#,
        "boolean(true)",
    );
}

#[test]
fn with_var_initializers_keep_empty_declaration_completion() {
    assert_completion(
        "with ({ selected: 1 }) { 23; var selected = 7; }",
        "number(23)",
    );
    assert_completion(
        "with ({ selected: 1 }) { 23; var selected = 7, later = 9; }",
        "number(23)",
    );
}

#[test]
fn an_empty_with_completion_does_not_reuse_the_outer_statement_value() {
    assert_completion("23; with ({}) {}", "undefined");
    assert_completion("23; with ({}) { var declared = 7; }", "undefined");
}

#[test]
fn with_break_and_continue_preserve_only_values_produced_inside_the_body() {
    for (source, expected) in [
        (
            "1; do { 2; with ({}) { 3; break; } 4; } while (false);",
            "number(3)",
        ),
        (
            "5; do { 6; with ({}) { break; } 7; } while (false);",
            "undefined",
        ),
        (
            "8; do { 9; with ({}) { 10; continue; } 11; } while (false);",
            "number(10)",
        ),
        (
            "12; do { 13; with ({}) { continue; } 14; } while (false);",
            "undefined",
        ),
    ] {
        assert_completion(source, expected);
    }
}
