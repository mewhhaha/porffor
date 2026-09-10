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
        .expect("prepared direct eval must execute against caller Environment Records");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn direct_eval_shares_caller_closure_and_lexical_cells() {
    run_boolean(
        r#"
function outer() {
  let enclosed = 4;
  return function inner(parameter) {
    let local = 3;
    const immutable = 5;
    var initial = eval("enclosed + local + parameter");
    eval("enclosed += 1; local += 2; parameter += 3;");
    var constant = false;
    try { eval("immutable = 9"); } catch (error) { constant = error instanceof TypeError; }
    var tdz = false;
    { try { eval("typeof later"); } catch (error) { tdz = error instanceof ReferenceError; } let later = 1; }
    return initial === 9 && enclosed === 5 && local === 5 && parameter === 5 && immutable === 5 && constant && tdz;
  };
}
outer()(2);
"#,
    );
}

#[test]
fn lexical_declarations_shadow_globals_in_the_same_cells_seen_by_eval_and_closures() {
    run_boolean(
        r#"
var mutable = 1;
var immutable = 2;
var Constructor = 3;
function caller(eval) {
  var tdz = false;
  try { eval('mutable;'); } catch (error) { tdz = error instanceof ReferenceError; }
  let mutable = 31;
  const immutable = 32;
  class Constructor { static value() { return 33; } }
  var read = () => mutable;
  var initial = mutable === 31 && immutable === 32 && Constructor.value() === 33;
  var observed = eval('mutable + immutable + Constructor.value();');
  eval('mutable += 1;');
  var constant = false;
  try { eval('immutable = 0;'); } catch (error) { constant = error instanceof TypeError; }
  return tdz && initial && observed === 96 && mutable === 32 && read() === 32 && constant;
}
caller(eval) && mutable === 1 && immutable === 2 && Constructor === 3;
"#,
    );
}

#[test]
fn parameter_captures_and_nested_blocks_keep_distinct_shadowed_lexical_cells() {
    run_boolean(
        r#"
var shadow = 7;
function caller(readOuter = () => shadow) {
  let shadow = 31;
  var readBody = () => shadow;
  var nested;
  { let shadow = 41; nested = eval('shadow;'); }
  return readOuter() === 7 && readBody() === 31 && eval('shadow;') === 31 && nested === 41;
}
caller() && shadow === 7;
"#,
    );
}

#[test]
fn captured_root_lexicals_shadow_globals_without_eval() {
    run_boolean(
        r#"
var shadow = 7;
function caller() {
  let shadow = 31;
  return () => shadow;
}
var read = caller();
read() === 31 && shadow === 7;
"#,
    );
}

#[test]
fn eval_added_bindings_change_later_resolution_and_preserve_existing_references() {
    run_boolean(
        r#"
let value = 1;
function observe() {
  var read = () => value;
  var before = read();
  value = eval("var value = 10; 2");
  var after = read();
  var removed = eval("delete value");
  var restored = read();
  eval("var recreated = 1");
  recreated = eval("delete recreated; var recreated; 8");
  return before === 1 && after === 10 && removed && restored === 2 && recreated === 8;
}
var result = observe();
result && value === 2 && typeof recreated === "undefined";
"#,
    );
}

#[test]
fn direct_eval_declarations_validate_before_mutation_and_keep_scope_lifetimes() {
    run_boolean(
        r#"
function declarations(parameter) {
  var collision = false;
  {
    let blocked = 1;
    try { eval("var added = 2; function fresh() {} var blocked = 3;"); }
    catch (error) { collision = error instanceof SyntaxError; }
  }
  var atomic = typeof added === "undefined" && typeof fresh === "undefined";
  eval("var parameter = 7; var existing = 3; function callable() { return existing; }");
  var first = callable;
  eval("function callable() { return existing + 1; }");
  var second = callable;
  var stable = eval("delete parameter") === false;
  var isolated = eval('"use strict"; var hidden = 9; hidden;');
  eval("let lexical = 10; const fixed = 11;");
  return collision && atomic && parameter === 7 && arguments[0] === 7
    && first !== second && first() === 3 && second() === 4 && stable
    && isolated === 9 && typeof hidden === "undefined"
    && typeof lexical === "undefined" && typeof fixed === "undefined";
}
function strictCaller() { "use strict"; eval("var hidden = 1"); return typeof hidden === "undefined"; }
declarations(1) && strictCaller();
"#,
    );
}

#[test]
fn direct_eval_respects_with_unscopables_catch_grammar_and_named_function_bindings() {
    run_boolean(
        r#"
var x = 1;
var object = { x: 10, check: function() { return this === object; } };
var withReceiver;
with (object) { withReceiver = eval("x += 1; check();"); }
object[Symbol.unscopables] = { x: true };
var excluded;
with (object) { excluded = eval("x"); }
function catchScopes() {
  var simple;
  try { throw 1; } catch (caught) { eval("var caught = 2;"); simple = caught === 2; }
  var rejected = false;
  try { throw { caught: 1 }; } catch ({ caught }) {
    try { eval("var added = 3; var caught = 4;"); } catch (error) { rejected = error instanceof SyntaxError; }
  }
  return simple && caught === undefined && rejected && typeof added === "undefined";
}
var named = function self() { return eval("self = 2; self;"); };
var strictNamed = function strictSelf() {
  try { eval('"use strict"; strictSelf = 2;'); } catch (error) { return error instanceof TypeError; }
};
withReceiver && object.x === 11 && excluded === 1 && catchScopes() && named() === named && strictNamed();
"#,
    );
}

#[test]
fn object_environment_writes_recheck_the_held_binding_after_rhs_effects() {
    run_boolean(
        r#"
eval('');
var log = [];
var target = { value: 0 };
var scope = new Proxy(target, {
  has(target, key) {
    if (key === 'value') log.push('has');
    return Reflect.has(target, key);
  },
  get(target, key, receiver) {
    if (key === Symbol.unscopables) log.push('unscopables');
    if (key === 'value') log.push('get');
    return Reflect.get(target, key, receiver);
  },
  set(target, key, value, receiver) {
    if (key === 'value') log.push('set');
    return Reflect.set(target, key, value, receiver);
  }
});
with (scope) { value = (log.push('rhs'), 1); }
var assignment = log.join(',') === 'has,unscopables,rhs,has,set';
log.length = 0;
with (scope) { value += (log.push('rhs'), 2); }
assignment && target.value === 3
  && log.join(',') === 'has,unscopables,has,get,rhs,has,set';
"#,
    );
}

#[test]
fn strict_object_environment_writes_reject_bindings_deleted_after_resolution() {
    run_boolean(
        r#"
var scope = { removed: 0 };
var rhsDeleted = false;
with (scope) {
  try { eval('"use strict"; removed = (delete scope.removed, 123)'); }
  catch (error) { rhsDeleted = error instanceof ReferenceError; }
}
var unscopablesCalls = 0;
var deletedByGetter = {
  removed: 0,
  get [Symbol.unscopables]() {
    unscopablesCalls++;
    delete deletedByGetter.removed;
    return null;
  }
};
var getterDeleted = false;
with (deletedByGetter) {
  try { eval('"use strict"; removed = 123'); }
  catch (error) { getterDeleted = error instanceof ReferenceError; }
}
globalThis.removedGlobal = 0;
var globalDeleted = false;
try { eval('"use strict"; removedGlobal = (delete globalThis.removedGlobal, 123)'); }
catch (error) { globalDeleted = error instanceof ReferenceError; }
rhsDeleted && getterDeleted && globalDeleted && unscopablesCalls === 1
  && !Object.prototype.hasOwnProperty.call(scope, 'removed')
  && !Object.prototype.hasOwnProperty.call(deletedByGetter, 'removed')
  && !Object.prototype.hasOwnProperty.call(globalThis, 'removedGlobal');
"#,
    );
}

#[test]
fn object_environment_recheck_throws_preserve_identity_and_stop_the_set() {
    run_boolean(
        r#"
eval('');
var marker = {};
var hasCalls = 0;
var setCalls = 0;
var target = { value: 1 };
var scope = new Proxy(target, {
  has(target, key) {
    if (key === 'value' && ++hasCalls === 2) throw marker;
    return Reflect.has(target, key);
  },
  set(target, key, value) { setCalls++; target[key] = value; return true; }
});
var caught = false;
try {
  try { with (scope) { value = 2; } }
  catch (error) { throw error; }
} catch (error) { caught = error === marker; }
caught && hasCalls === 2 && setCalls === 0 && target.value === 1;
"#,
    );
}

#[test]
fn initially_unresolvable_writes_keep_their_original_reference_kind() {
    run_boolean(
        r#"
eval('');
var hasCalls = 0;
var target = {};
var scope = new Proxy(target, {
  has(target, key) {
    if (key === 'initiallyMissing') hasCalls++;
    return Reflect.has(target, key);
  }
});
with (scope) { initiallyMissing = (scope.initiallyMissing = 9, 17); }
var sloppy = hasCalls === 1 && target.initiallyMissing === 9
  && globalThis.initiallyMissing === 17;
var strict = false;
try { eval('"use strict"; strictMissing = (globalThis.strictMissing = 11, 19)'); }
catch (error) { strict = error instanceof ReferenceError; }
sloppy && strict && globalThis.strictMissing === 11;
"#,
    );
}

#[test]
fn parameter_expression_eval_uses_parameter_scope_and_body_variables_stay_separate() {
    run_boolean(
        r#"
var bodyOnly = "outer";
function parameterScope(
  first = eval("bodyOnly"),
  second = eval("var fromParameter = 4; fromParameter"),
  capture = () => fromParameter
) {
  var bodyOnly = "body";
  var fromParameter = 9;
  return first === "outer" && second === 4 && capture() === 4 && fromParameter === 9;
}
parameterScope();
"#,
    );
}

#[test]
fn parameter_defaults_reject_eval_parameter_collisions_and_preserve_body_copies() {
    run_boolean(
        r#"
var rejected = false;
try { (function(parameter = eval('var parameter;')) {})(); }
catch (error) { rejected = error instanceof SyntaxError; }
function copied(parameter = 3, capture = () => parameter) {
  var parameter;
  parameter = 7;
  return capture() === 3 && parameter === 7;
}
function declared(parameter = 3, capture = () => parameter) {
  function parameter() { return 9; }
  return capture() === 3 && parameter() === 9;
}
function pattern({ [eval('var fromKey = 4; "value"')]: value } = { value: 6 }, capture = () => fromKey) {
  var fromKey = 8;
  return value === 6 && capture() === 4 && fromKey === 8;
}
rejected && copied() && declared() && pattern();
"#,
    );
}

#[test]
fn direct_eval_annex_b_copy_targets_variable_record_beyond_with_and_catch() {
    run_boolean(
        r#"
function copies() {
  var object = { copied: 1 };
  with (object) { eval('{ function copied() { return 3; } }'); }
  var caughtCopy;
  try { throw 2; } catch (caught) {
    eval('{ function caught() { return 4; } }');
    caughtCopy = caught === 2;
  }
  return object.copied === 1 && copied() === 3 && caughtCopy && caught() === 4;
}
copies();
"#,
    );
}

#[test]
fn ordinary_and_class_defaults_capture_outer_names_hidden_by_body_declarations() {
    run_boolean(
        r#"
function enclosing() {
  let outer = 6;
  function ordinary(value = outer, capture = () => outer) {
    var outer = 9;
    return value === 6 && capture() === 6 && outer === 9;
  }
  class Example {
    method(value = outer, capture = () => outer) {
      var outer = 10;
      return value === 6 && capture() === 6 && outer === 10;
    }
  }
  var named = function self(capture = () => self) {
    var self;
    return capture() === named && self === undefined;
  };
  return ordinary() && new Example().method() && named();
}
enclosing();
"#,
    );
}
