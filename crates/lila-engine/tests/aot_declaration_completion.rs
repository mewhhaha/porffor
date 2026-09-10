use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_completion(source: &str, expected: &str) {
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
        .expect("binding initialization preserves the Script completion through Wasm");
    assert!(outcome.note.contains(expected), "{}", outcome.note);
}

fn assert_declaration_completions(evaluate: &str, setup: &str, untouched_counter: &str) {
    // Capturing each lexical binding forces tagged environment cells instead
    // of local-only storage, including the class and destructuring cases.
    let cases = [
        (
            "23; { let value; function read() { calls++; return value; } }",
            "23",
        ),
        (
            "'retained'; { const value = {}; function read() { calls++; return value; } }",
            "'retained'",
        ),
        (
            "marker; { let { value } = { value: 9 }; function read() { calls++; return value; } }",
            "marker",
        ),
        (
            "marker; { class Value {} function read() { calls++; return Value; } }",
            "marker",
        ),
        ("23; { function unused() { calls++; } }", "23"),
        (
            "'retained'; {} function unused() { calls++; }",
            "'retained'",
        ),
        (
            "undefined; { let value; function read() { calls++; return value; } }",
            "undefined",
        ),
        (
            "; { let value; function read() { calls++; return value; } }",
            "undefined",
        ),
        (
            "23; { let value; function read() { calls++; return value; } } undefined;",
            "undefined",
        ),
    ];
    let mut source = String::from(setup);
    for (index, (script, expected)) in cases.iter().enumerate() {
        source.push_str(&format!(
            "if ({evaluate}({script:?}) !== {expected}) throw new Error('declaration completion case {index}');\n"
        ));
    }
    source.push_str(&format!("{untouched_counter} === 0;"));
    assert_completion(&source, "boolean(true)");
}

#[test]
fn direct_eval_keeps_typed_completions_through_captured_declarations() {
    assert_declaration_completions("eval", "var marker = {}; var calls = 0;\n", "calls");
}

#[test]
fn indirect_eval_keeps_typed_completions_through_captured_declarations() {
    assert_declaration_completions("(0, eval)", "var marker = {}; var calls = 0;\n", "calls");
}

#[test]
fn realm_scripts_keep_typed_completions_through_captured_declarations() {
    assert_declaration_completions(
        "realm.evalScript",
        "var realm = __lilaCreateRealm(); var marker = {}; realm.global.marker = marker; realm.global.calls = 0;\n",
        "realm.global.calls",
    );
}

#[test]
fn entry_script_exports_the_prior_value_after_captured_declarations() {
    assert_completion(
        "eval(''); 23; { let value; function read() { return value; } }",
        "number(23)",
    );
}

#[test]
fn empty_script_completions_ignore_existing_global_values() {
    assert_completion(
        r#"
var marker = {};
globalThis.cachedNumber = 17;
globalThis.cachedString = 'prior';
globalThis.cachedObject = marker;
var realm = __lilaCreateRealm();
realm.global.cachedNumber = 17;
realm.global.cachedString = 'prior';
realm.global.cachedObject = marker;
var indirect = (0, eval)('var cachedNumber;') === undefined
  && (0, eval)('var cachedString;') === undefined
  && (0, eval)('var cachedObject;') === undefined;
var prepared = realm.evalScript('var cachedNumber;') === undefined
  && realm.evalScript('var cachedString;') === undefined
  && realm.evalScript('var cachedObject;') === undefined;
indirect && prepared
  && globalThis.cachedNumber === 17 && globalThis.cachedString === 'prior'
  && globalThis.cachedObject === marker
  && realm.global.cachedNumber === 17 && realm.global.cachedString === 'prior'
  && realm.global.cachedObject === marker;
"#,
        "boolean(true)",
    );
}

const GLOBAL_FUNCTION_WRITES: &str = r#"
function declared() { return 1; }
var declared;
var original = declared;
if (original !== globalThis.declared || original() !== 1)
  throw new Error('installed global function identity');
globalThis.declared = function replacement() { return 2; };
if (declared !== globalThis.declared || declared() !== 2)
  throw new Error('replaced global function');
globalThis.declared = 5;
var compound = declared += 2;
var previous = declared++;
{
  let declared = 30;
  declared += 1;
  if (declared !== 31) throw new Error('lexical shadow');
}
compound === 7 && previous === 7 && declared === 8 && globalThis.declared === 8;
"#;

#[test]
fn entry_function_and_var_bindings_follow_global_identity_and_writes() {
    assert_completion(GLOBAL_FUNCTION_WRITES, "boolean(true)");
}

#[test]
fn prepared_function_and_var_bindings_follow_global_identity_and_writes() {
    assert_completion(
        &format!(
            "var realm = __lilaCreateRealm(); eval({GLOBAL_FUNCTION_WRITES:?}) && (0, eval)({GLOBAL_FUNCTION_WRITES:?}) && realm.evalScript({GLOBAL_FUNCTION_WRITES:?});"
        ),
        "boolean(true)",
    );
}

#[test]
fn global_compound_results_preserve_runtime_kinds_and_abrupt_order() {
    assert_completion(
        r#"
var numberValue = 5;
var numberResult = numberValue += 2;
var stringValue = 'a';
var stringResult = stringValue += 2;
var bigValue = 9223372036854775808n;
var bigResult = bigValue += 2n;
var bitValue = 6n;
var bitResult = bitValue &= 3n;
var marker = {};
var trace = [];
var operand = {valueOf() { trace.push('coerce'); throw marker; }};
var received;
try { operand += (trace.push('rhs'), 1); } catch (error) { received = error; }
var realm = __lilaCreateRealm();
var missing = realm.global.eval("var missing = 17; var calls = 0; delete missing; var received = false; try { missing += calls++; } catch (error) { received = error instanceof ReferenceError; } received && calls === 0;");
numberResult === 7 && numberValue === 7 && typeof numberResult === 'number'
  && stringResult === 'a2' && stringValue === 'a2'
  && bigResult === 9223372036854775810n && bigValue === bigResult
  && bitResult === 2n && bitValue === 2n
  && received === marker && trace.join(',') === 'rhs,coerce' && missing;
"#,
        "boolean(true)",
    );
}

#[test]
fn coercive_add_promotes_bigints_and_preserves_conversion_order() {
    assert_completion(
        r#"
var trace = [];
var left = {valueOf() { trace.push('left-primitive'); return 9223372036854775807n; }};
var right = {valueOf() { trace.push('right-primitive'); return 1n; }};
function readLeft() { trace.push('left-eval'); return left; }
function readRight() { trace.push('right-eval'); return right; }
var promoted = readLeft() + readRight();
var negative = -9223372036854775808n;
var negativeResult = negative += -1n;
var heap = 9223372036854775808n;
var heapResult = heap += 9223372036854775808n;
var mixedLeft = 9223372036854775808n;
var mixedRight = 1;
var leftError = false;
var rightError = false;
try { mixedLeft += 1; } catch (error) { leftError = error instanceof TypeError; }
try { mixedRight += 9223372036854775808n; } catch (error) { rightError = error instanceof TypeError; }
var text = {valueOf() { return 'value='; }};
var concatenated = text + 9223372036854775808n;
promoted === 9223372036854775808n && typeof promoted === 'bigint'
  && trace.join(',') === 'left-eval,right-eval,left-primitive,right-primitive'
  && negativeResult === -9223372036854775809n && negative === negativeResult
  && heapResult === 18446744073709551616n && heap === heapResult
  && leftError && rightError && mixedLeft === 9223372036854775808n && mixedRight === 1
  && concatenated === 'value=9223372036854775808';
"#,
        "boolean(true)",
    );
}

#[test]
fn coercive_add_symbol_errors_preserve_handlers_targets_and_defining_realm() {
    assert_completion(
        r#"
var realm = __lilaCreateRealm();
var foreignAdd = realm.global.eval('(function add(left, right) { return left + right; })');
function add(left, right) { return left + right; }
var finalized = 0;
function catchesInRealm(operation, prototype) {
  var received;
  try { operation(); }
  catch (error) { received = error; }
  finally { finalized++; }
  return received !== undefined && Object.getPrototypeOf(received) === prototype;
}
var symbol = Symbol('main');
var foreignSymbol = realm.global.Symbol('foreign');
var mainLeft = catchesInRealm(function () { return add(foreignSymbol, 1); }, TypeError.prototype);
var mainRight = catchesInRealm(function () { return add(1, foreignSymbol); }, TypeError.prototype);
var foreignLeft = catchesInRealm(function () { return foreignAdd(symbol, 1); }, realm.global.TypeError.prototype);
var foreignRight = catchesInRealm(function () { return foreignAdd(1, symbol); }, realm.global.TypeError.prototype);
var trace = [];
var left = {[Symbol.toPrimitive](hint) { trace.push('left:' + hint); return symbol; }};
var right = {[Symbol.toPrimitive](hint) { trace.push('right:' + hint); return 1; }};
var afterBothPrimitives = catchesInRealm(function () { return add(left, right); }, TypeError.prototype);
var target = symbol;
var unchanged = catchesInRealm(function () { return target += 1; }, TypeError.prototype);
mainLeft && mainRight && foreignLeft && foreignRight && afterBothPrimitives && unchanged
  && finalized === 6 && target === symbol && trace.join(',') === 'left:default,right:default';
"#,
        "boolean(true)",
    );
}

#[test]
fn coercive_arithmetic_preserves_numeric_order_tags_and_abrupt_completions() {
    assert_completion(
        r#"
const trace = [];
const left = {[Symbol.toPrimitive](hint) { trace.push('left:' + hint); return 2n; }};
const right = {[Symbol.toPrimitive](hint) { trace.push('right:' + hint); return 3; }};
let mixed = 0;
let finalized = 0;
try { left - right; } catch (error) { if (error instanceof TypeError) mixed++; } finally { finalized++; }
try { left * right; } catch (error) { if (error instanceof TypeError) mixed++; } finally { finalized++; }
try { left / right; } catch (error) { if (error instanceof TypeError) mixed++; } finally { finalized++; }
try { left % right; } catch (error) { if (error instanceof TypeError) mixed++; } finally { finalized++; }
try { left ** right; } catch (error) { if (error instanceof TypeError) mixed++; } finally { finalized++; }
if (mixed !== 5 || finalized !== 5 || trace.join(',') !==
    'left:number,right:number,left:number,right:number,left:number,right:number,left:number,right:number,left:number,right:number')
  throw new Error('mixed arithmetic conversion order');

const huge = {valueOf() { trace.push('left-primitive'); return 4611686018427387904n; }};
const multiplier = {valueOf() { trace.push('right-primitive'); return 2n; }};
const directProduct = huge * multiplier;
trace.length = 0;
const orderedProduct = (trace.push('left-eval'), huge) * (trace.push('right-eval'), multiplier);
if (directProduct !== 9223372036854775808n || typeof directProduct !== 'bigint'
    || orderedProduct !== directProduct || trace.join(',') !== 'left-eval,right-eval,left-primitive,right-primitive')
  throw new Error('BigInt arithmetic evaluation and runtime tag');

const marker = {};
let received;
const throwingRight = {[Symbol.toPrimitive]() { trace.push('right-throw'); throw marker; }};
trace.length = 0;
try { left ** throwingRight; } catch (error) { received = error; }
if (received !== marker || trace.join(',') !== 'left:number,right-throw')
  throw new Error('right conversion throw identity');

const symbolLeft = {[Symbol.toPrimitive](hint) { trace.push('symbol:' + hint); return Symbol(); }};
trace.length = 0;
let symbolError;
try { symbolLeft - right; } catch (error) { symbolError = error; }
if (!(symbolError instanceof TypeError) || trace.join(',') !== 'symbol:number')
  throw new Error('left numeric conversion skips right conversion');
trace.length = 0;
symbolError = undefined;
try { symbolLeft + right; } catch (error) { symbolError = error; }
symbolError instanceof TypeError && trace.join(',') === 'symbol:default,right:default';
"#,
        "boolean(true)",
    );
}

#[test]
fn configurable_eval_functions_follow_deletion_and_accessor_replacement() {
    assert_completion(
        r#"
var other = __lilaCreateRealm().global;
var missingType = other.eval('function deleted() {} delete globalThis.deleted; typeof deleted;');
var identifierDeleted = other.eval('function byName() {} delete byName;');
var missingError;
try {
  other.eval('function absent() {} delete globalThis.absent; absent;');
} catch (error) {
  missingError = error;
}
var deletedHostError;
try {
  other.eval('function parseInt() {} delete globalThis.parseInt; parseInt;');
} catch (error) {
  deletedHostError = error;
}
var getterResult = other.eval("function replaced() { return 1; } var replaced; var gets = 0; var replacement = function() { return 9; }; Object.defineProperty(globalThis, 'replaced', {configurable: true, get() { gets++; return replacement; }}); var selected = replaced; var result = replaced(); selected === replacement && result === 9 && gets === 2;");
missingType === 'undefined' && identifierDeleted === true
  && !Object.prototype.hasOwnProperty.call(other, 'byName')
  && missingError instanceof other.ReferenceError
  && deletedHostError instanceof other.ReferenceError && getterResult;
"#,
        "boolean(true)",
    );
}

#[test]
fn source_global_functions_override_host_spelling_signatures() {
    let script = "function parseInt() { return 'replacement'; } parseInt('17') === 'replacement';";
    assert_completion(script, "boolean(true)");
    assert_completion(
        &format!(
            "var realm = __lilaCreateRealm(); (0, eval)({script:?}) && realm.evalScript({script:?});"
        ),
        "boolean(true)",
    );
}

#[test]
fn declarations_and_initializers_do_not_invoke_global_getters() {
    assert_completion(
        r#"
var realm = __lilaCreateRealm();
var marker = {};
var entryGets = 0;
var entrySets = [];
var foreignGets = 0;
var foreignSets = [];
Object.defineProperty(globalThis, 'existing', {
  configurable: true,
  get() { entryGets++; return 17; },
  set(value) { entrySets.push(value); }
});
Object.defineProperty(realm.global, 'existing', {
  configurable: true,
  get() { foreignGets++; throw marker; },
  set(value) { foreignSets.push(value); }
});
if ((0, eval)('var existing;') !== undefined || entryGets !== 0)
  throw new Error('indirect declaration invoked getter');
if (realm.evalScript('var existing;') !== undefined || foreignGets !== 0)
  throw new Error('realm declaration invoked getter');
if ((0, eval)('var existing = 23;') !== undefined || entryGets !== 0
    || entrySets.join(',') !== '23')
  throw new Error('indirect initializer performed more than Set');
if (realm.evalScript('var existing = 31;') !== undefined || foreignGets !== 0
    || foreignSets.join(',') !== '31')
  throw new Error('realm initializer performed more than Set');
var observed = (0, eval)('existing;');
var received;
try { realm.evalScript('existing;'); } catch (error) { received = error; }
observed === 17 && entryGets === 1 && received === marker && foreignGets === 1;
"#,
        "boolean(true)",
    );
}

#[test]
fn readonly_globals_keep_their_values_after_sloppy_declaration_initializers() {
    assert_completion(
        r#"
var realm = __lilaCreateRealm();
Object.defineProperty(globalThis, 'fixed', {value: 17, writable: false, configurable: true});
Object.defineProperty(realm.global, 'fixed', {value: 19, writable: false, configurable: true});
var indirect = (0, eval)('var fixed = 23; fixed;');
var prepared = realm.evalScript('var fixed = 31; fixed;');
indirect === 17 && globalThis.fixed === 17
  && prepared === 19 && realm.global.fixed === 19;
"#,
        "boolean(true)",
    );
}

#[test]
fn entry_global_vars_and_lexical_shadows_remain_non_deletable() {
    assert_completion(
        r#"
var declared = 17;
var removed = delete declared;
var throughWith;
with ({}) { throughWith = declared; }
{
  let declared = 23;
  if (delete declared || declared !== 23)
    throw new Error('lexical shadow deletion');
}
function localBinding() {
  var local = 31;
  return !delete local && local === 31;
}
!removed && declared === 17 && throughWith === 17 && typeof declared === 'number'
  && !Object.getOwnPropertyDescriptor(globalThis, 'declared').configurable
  && localBinding();
"#,
        "boolean(true)",
    );
}

#[test]
fn prepared_var_deletion_uses_runtime_descriptors_and_identifier_presence() {
    assert_completion(
        r#"
var realm = __lilaCreateRealm();
var indirect = realm.global.eval("var temporary = 17; var removed = delete temporary; var missing = false; try { temporary; } catch (error) { missing = error instanceof ReferenceError; } removed && typeof temporary === 'undefined' && missing;");
var missingWith = realm.global.eval("var withTemporary = 19; delete withTemporary; var withMissing = false; try { with ({}) { withTemporary; } } catch (error) { withMissing = error instanceof ReferenceError; } withMissing;");
var propertyDelete = realm.global.eval("var propertyTemporary = 19; delete globalThis.propertyTemporary; var missingProperty = false; try { propertyTemporary; } catch (error) { missingProperty = error instanceof ReferenceError; } typeof propertyTemporary === 'undefined' && missingProperty;");
Object.defineProperty(realm.global, 'existing', {value: 23, configurable: true});
var reused = realm.evalScript('var existing; delete existing;');
var retained = realm.evalScript("var retained = 31; var removedRetained = delete retained; !removedRetained && retained === 31 && typeof retained === 'number';");
indirect && missingWith && propertyDelete && reused && retained
  && !Object.prototype.hasOwnProperty.call(realm.global, 'temporary')
  && !Object.prototype.hasOwnProperty.call(realm.global, 'propertyTemporary')
  && !Object.prototype.hasOwnProperty.call(realm.global, 'existing')
  && !Object.getOwnPropertyDescriptor(realm.global, 'retained').configurable;
"#,
        "boolean(true)",
    );
}

#[test]
fn annex_b_copies_publish_only_admitted_functions_without_reading_prior_values() {
    assert_completion(
        r#"
var realm = __lilaCreateRealm();
var prior = {};
realm.global.accepted = prior;
realm.evalScript('let blocked = 7;');
var accepted = realm.evalScript('23; { function accepted() { return 31; } }');
var rejected = realm.evalScript('29; { function blocked() { return 37; } }');
accepted === 23 && rejected === 29 && realm.global.accepted !== prior
  && realm.global.accepted() === 31 && realm.evalScript('blocked;') === 7
  && !Object.prototype.hasOwnProperty.call(realm.global, 'blocked');
"#,
        "boolean(true)",
    );
}

#[test]
fn escaped_global_readers_resolve_deleted_and_inherited_bindings_at_call_time() {
    assert_completion(
        r#"
var realm = __lilaCreateRealm();
var readers = realm.global.eval("var captured = 17; function readCaptured() { return captured; } function typeofCaptured() { return typeof captured; } delete captured; [readCaptured, typeofCaptured];");
var missingVar = false;
try { readers[0](); } catch (error) { missingVar = error instanceof realm.global.ReferenceError; }
var functionReader = realm.global.eval("function gone() {} function readGone() { return gone; } delete globalThis.gone; readGone;");
var missingFunction = false;
try { functionReader(); } catch (error) { missingFunction = error instanceof realm.global.ReferenceError; }
var inheritedReader = realm.global.eval("var inherited = 19; function readInherited() { return inherited; } delete inherited; readInherited;");
Object.getPrototypeOf(realm.global).inherited = 'replacement';
missingVar && missingFunction && readers[1]() === 'undefined'
  && inheritedReader() === 'replacement';
"#,
        "boolean(true)",
    );
}

#[test]
fn entry_global_loop_heads_preserve_empty_and_body_completions() {
    assert_completion("23; for (var key in {a: 1}) {}", "undefined");
    assert_completion("23; for (var value of [7]) {}", "undefined");
    assert_completion("for (var value of [7, 9]) { value; }", "number(9)");
}

#[test]
fn prepared_global_loop_heads_preserve_empty_and_body_completions() {
    let cases = [
        ("23; for (var key in {a: 1}) {}", "undefined"),
        ("23; for (var value of [7]) {}", "undefined"),
        (
            "for (var key in {a: 1, b: 2}) { 'retained'; }",
            "'retained'",
        ),
        ("for (var value of [7, 9]) { value; }", "9"),
        ("for (var [value] of [[7], [9]]) {}", "undefined"),
    ];
    let mut source = String::from("var realm = __lilaCreateRealm();\n");
    for (index, (script, expected)) in cases.iter().enumerate() {
        for evaluate in ["eval", "(0, eval)", "realm.evalScript"] {
            source.push_str(&format!(
                "if ({evaluate}({script:?}) !== {expected}) throw new Error('loop completion case {index}: {evaluate}');\n"
            ));
        }
    }
    source.push_str("true;");
    assert_completion(&source, "boolean(true)");
}

#[test]
fn borrowed_eval_loop_heads_write_the_selected_caller_bindings() {
    assert_completion(
        r#"
function run() {
  var value = 3;
  var ofCompletion = eval('for (var value of [7, 9]) { value; }');
  if (ofCompletion !== 9 || value !== 9) throw new Error('borrowed for-of');
  var inCompletion = eval('for (var value in {a: 1, b: 2}) { value; }');
  if (inCompletion !== 'b' || value !== 'b') throw new Error('borrowed for-in');
  var initializers = 0;
  var empty = eval('for (var value = (initializers++, 23) in {}) {}');
  return empty === undefined && value === 23 && initializers === 1;
}
var head = 1;
var sets = [];
var gets = 0;
var scope = {
  get head() { gets++; throw 'unexpected read'; },
  set head(value) { sets.push(value); }
};
with (scope) { eval('for (var head of [7, 9]) {}'); }
var marker = {};
var closeMarker = {};
var closes = 0;
var bodyCalls = 0;
var iterable = {
  [Symbol.iterator]() {
    return {
      next() { return {value: 1, done: false}; },
      return() { closes++; throw closeMarker; }
    };
  }
};
var throwing = {set head(value) { throw marker; }};
var received;
with (throwing) {
  try { eval('for (var head of iterable) { bodyCalls++; }'); }
  catch (error) { received = error; }
}
run() && head === 1 && gets === 0 && sets.join(',') === '7,9'
  && received === marker && closes === 1 && bodyCalls === 0;
"#,
        "boolean(true)",
    );
}

#[test]
fn global_loop_publishers_invoke_only_set_and_preserve_setter_throws() {
    assert_completion(
        r#"
var realm = __lilaCreateRealm();
var marker = {};
var sets = [];
var gets = 0;
Object.defineProperty(realm.global, 'head', {
  configurable: true,
  get() { gets++; throw marker; },
  set(value) { sets.push(value); }
});
var ofCompletion = realm.evalScript('for (var head of [7, 9]) {}');
var inCompletion = realm.evalScript('for (var head in {a: 1, b: 2}) {}');
realm.global.bodyCalls = 0;
Object.defineProperty(realm.global, 'throwing', {
  configurable: true,
  get() { gets++; throw 'getter ran'; },
  set(value) { throw marker; }
});
var received;
var closes = 0;
var closeMarker = {};
realm.global.iterable = {
  [Symbol.iterator]() {
    return {
      next() { return {value: 1, done: false}; },
      return() { closes++; throw closeMarker; }
    };
  }
};
try {
  realm.evalScript('for (var throwing of iterable) { bodyCalls++; }');
} catch (error) {
  received = error;
}
ofCompletion === undefined && inCompletion === undefined
  && sets.join(',') === '7,9,a,b' && gets === 0
  && received === marker && realm.global.bodyCalls === 0 && closes === 1;
"#,
        "boolean(true)",
    );
}

#[test]
fn abrupt_lexical_initialization_keeps_the_thrown_value() {
    assert_completion(
        r#"
var marker = {};
var received;
try {
  eval('23; { let value = (() => { throw marker; })(); function read() { return value; } }');
} catch (error) {
  received = error;
}
received === marker;
"#,
        "boolean(true)",
    );
}

#[test]
fn borrowed_eval_var_initializers_preserve_empty_declaration_completions() {
    assert_completion(
        r#"
var marker = {};
var trace = [];
if (eval('23; var initialized = 123;') !== 23 || initialized !== 123)
  throw new Error('initialized var');
if (eval("'retained'; var first = (trace.push('first'), 1), second = (trace.push('second'), 2);") !== 'retained'
    || first !== 1 || second !== 2 || trace.join(',') !== 'first,second')
  throw new Error('ordered declarators');
var source = {get value() { trace.push('get'); return 7; }};
if (eval('marker; var {value: selected} = source;') !== marker || selected !== 7
    || trace.join(',') !== 'first,second,get')
  throw new Error('object destructuring');
if (eval('23; var [head, tail] = [11, 19];') !== 23 || head !== 11 || tail !== 19)
  throw new Error('array destructuring');
if (eval("'retained'; var {} = {};") !== 'retained')
  throw new Error('empty object destructuring');
if (eval('marker; var [] = [];') !== marker)
  throw new Error('empty array destructuring');
if (eval('23; { var nested = 31; }') !== 23 || nested !== 31)
  throw new Error('nested var');
if (eval('undefined; var afterUndefined = 41;') !== undefined || afterUndefined !== 41)
  throw new Error('undefined completion');
true;
"#,
        "boolean(true)",
    );
}

#[test]
fn strict_eval_destructuring_keeps_the_prior_completion_and_private_bindings() {
    assert_completion(
        r#"
var marker = {};
eval("'use strict'; 23; var value = 123;") === 23
  && eval("'use strict'; marker; var {selected} = {selected: 7};") === marker
  && eval("'use strict'; 'retained'; var [head] = [11];") === 'retained'
  && eval("'use strict'; undefined; var {} = {};") === undefined
  && typeof value === 'undefined' && typeof selected === 'undefined'
  && typeof head === 'undefined';
"#,
        "boolean(true)",
    );
}

#[test]
fn abrupt_var_initializers_keep_throw_identity_and_skip_later_declarators() {
    assert_completion(
        r#"
var marker = {};
var calls = 0;
var received;
try {
  eval('23; var value = (() => { throw marker; })(), later = calls++;');
} catch (error) {
  received = error;
}
var destructuringReceived;
var source = {get value() { throw marker; }};
try {
  eval('23; var {value} = source, later = calls++;');
} catch (error) {
  destructuringReceived = error;
}
received === marker && destructuringReceived === marker && calls === 0
  && value === undefined && later === undefined;
"#,
        "boolean(true)",
    );
}
