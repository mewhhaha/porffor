use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_wasm_true(source: &str) {
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
        .expect("RegExp construction and iteration execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn constructor_identity_uses_is_regexp_and_the_active_realm_function() {
    assert_wasm_true(
        r#"
const trace = [];
const marker = {};
const like = {
  get [Symbol.match]() { trace.push('match'); return true; },
  get constructor() { trace.push('constructor'); return RegExp; },
  get source() { throw marker; },
  get flags() { throw marker; },
  toString() { throw marker; }
};
const identity = RegExp(like) === like;
const other = __lilaCreateRealm().global;
const foreign = other.RegExp('a', 'g');
const local = /a/g;
const copied = other.RegExp(local);
let received;
try { RegExp({get [Symbol.match]() { throw marker; }}); } catch (error) { received = error; }
const changed = /b/i;
Object.setPrototypeOf(changed, null);
changed.constructor = RegExp;
const forged = Object.create(RegExp.prototype);
Object.defineProperty(forged, Symbol.match, {value: undefined});
forged.toString = function () { return 'c'; };
identity && trace.join(',') === 'match,constructor' && received === marker
  && other.RegExp(foreign) === foreign && copied !== local
  && copied.source === 'a' && copied.flags === 'g'
  && Object.getPrototypeOf(copied) === other.RegExp.prototype
  && RegExp(changed) === changed && RegExp(forged).source === 'c';
"#,
    );
}

#[test]
fn retained_constructor_handles_keep_identity_after_global_replacement() {
    assert_wasm_true(
        r#"
var original = RegExp;
var regexp = /x/i;
var direct = RegExp(regexp);
var called = RegExp.call(null, regexp);
var applied = RegExp.apply(null, [regexp]);
var bound = original.bind(null);
var proxy = new Proxy(original, {});
var throughBound = bound(regexp);
var throughProxy = proxy(regexp);
function invoke(value) { return original(value); }
var nested = invoke(regexp);
var duringArgument = original((globalThis.RegExp = function replacement() {}, regexp));
var duringGetter = {
  [Symbol.match]: true,
  get constructor() {
    globalThis.RegExp = function anotherReplacement() {};
    return original;
  },
  get source() { throw 'unexpected source read'; }
};
var returned = original(duringGetter);
regexp.indicator = 1;
globalThis.RegExp = original;
direct === regexp && called === regexp && applied === regexp
  && throughBound === regexp && throughProxy === regexp && nested === regexp
  && duringArgument === regexp && returned === duringGetter && direct.indicator === 1;
"#,
    );
}

#[test]
fn constructor_observes_properties_before_prototype_and_then_coerces_in_order() {
    assert_wasm_true(
        r#"
const trace = [];
const pattern = {
  get [Symbol.match]() { trace.push('match'); return true; },
  get source() { trace.push('source'); return {toString() { trace.push('source-string'); return 'a+'; }}; },
  get flags() { trace.push('flags'); return {toString() { trace.push('flags-string'); return 'i'; }}; }
};
const target = new Proxy(function () {}, {
  get(object, key) {
    if (key === 'prototype') { trace.push('prototype'); return RegExp.prototype; }
    return object[key];
  }
});
const copy = Reflect.construct(RegExp, [pattern], target);
const original = /a+/gi;
original[Symbol.match] = false;
Object.defineProperty(original, 'source', {get() { throw 'source getter'; }});
Object.defineProperty(original, 'flags', {get() { throw 'flags getter'; }});
const internalCopy = new RegExp(original);
const override = new RegExp(original, 'm');
trace.join(',') === 'match,source,flags,prototype,source-string,flags-string'
  && copy.test('AAA') && internalCopy.source === 'a+' && internalCopy.flags === 'gi'
  && override.source === 'a+' && override.flags === 'm';
"#,
    );
}

#[test]
fn constructor_keeps_source_and_matcher_selected_before_new_target_getter() {
    assert_wasm_true(
        r#"
const original = /old/i;
const target = new Proxy(function () {}, {
  get(object, key) {
    if (key === 'prototype') {
      original.compile('new', '');
      return RegExp.prototype;
    }
    return object[key];
  }
});
const copy = Reflect.construct(RegExp, [original], target);
copy.source === 'old' && copy.flags === 'i' && copy.test('OLD') && !copy.test('new')
  && original.source === 'new' && original.test('new');
"#,
    );
}

#[test]
fn constructor_and_compile_validate_runtime_flags_in_their_defining_realm() {
    assert_wasm_true(
        r#"
const other = __lilaCreateRealm().global;
const invalid = ['r', 'gg', 'uv', 'vu', 'ii', String.fromCharCode(0)];
const original = /kept/g;
let catches = 0;
for (const flags of invalid) {
  try { new RegExp('', flags); } catch (error) { if (Object.getPrototypeOf(error) === SyntaxError.prototype) catches++; }
  try { new other.RegExp('', flags); } catch (error) { if (Object.getPrototypeOf(error) === other.SyntaxError.prototype) catches++; }
  try { original.compile('new', flags); } catch (error) { if (error instanceof SyntaxError) catches++; }
}
const marker = {};
let received;
try { new RegExp({toString() { throw marker; }}, {toString() { throw 'flags ran'; }}); }
catch (error) { received = error; }
catches === 18 && original.source === 'kept' && original.flags === 'g' && received === marker
  && new RegExp('', 'ygimsd').flags === 'dgimsy' && new RegExp('', 'v').unicodeSets;
"#,
    );
}

#[test]
fn modifier_grammar_accepts_empty_removal_and_rejects_invalid_prefixes() {
    assert_wasm_true(
        r#"
const invalid = ['(?1:a)', '(?I:a)', '(?u:a)', '(?é:a)', '(?-:a)', '(?ii:a)', '(?i-i:a)'];
let catches = 0;
for (const pattern of invalid) {
  try { new RegExp(pattern); } catch (error) { if (error instanceof SyntaxError) catches++; }
}
catches === invalid.length && /(?i-:a)/.test('A') && /(?is-:.)/.test('\n')
  && new RegExp('(?m-:^a$)').test('x\na\ny') && !/(?i-:a)b/.test('AB');
"#,
    );
}

#[test]
fn regexp_string_iterators_have_distinct_realm_prototypes_and_next_functions() {
    assert_wasm_true(
        r#"
const other = __lilaCreateRealm().global;
const first = /./g[Symbol.matchAll]('a');
const second = /./g[Symbol.matchAll]('b');
const prototype = Object.getPrototypeOf(first);
const foreign = new other.RegExp('.', 'g')[Symbol.matchAll]('c');
const foreignPrototype = Object.getPrototypeOf(foreign);
const tag = Object.getOwnPropertyDescriptor(prototype, Symbol.toStringTag);
const foreignTag = Object.getOwnPropertyDescriptor(foreignPrototype, Symbol.toStringTag);
const arrayPrototype = Object.getPrototypeOf([][Symbol.iterator]());
let mainError, foreignError, arrayError;
try { prototype.next.call({}); } catch (error) { mainError = error; }
try { foreignPrototype.next.call({}); } catch (error) { foreignError = error; }
try { arrayPrototype.next.call(first); } catch (error) { arrayError = error; }
let calls = 0;
const marker = {};
const savedExec = RegExp.prototype.exec;
RegExp.prototype.exec = function () { calls++; throw marker; };
const lazy = /./g[Symbol.matchAll]('x');
const lazyCalls = calls;
let received;
try { lazy.next(); } catch (error) { received = error; }
RegExp.prototype.exec = savedExec;
prototype === Object.getPrototypeOf(second) && prototype !== arrayPrototype
  && prototype !== foreignPrototype && prototype.next !== foreignPrototype.next
  && Object.getPrototypeOf(prototype) === Iterator.prototype
  && Object.getPrototypeOf(foreignPrototype) === other.Iterator.prototype
  && tag.value === 'RegExp String Iterator' && !tag.writable && !tag.enumerable && tag.configurable
  && foreignTag.value === tag.value && !foreignTag.writable && !foreignTag.enumerable && foreignTag.configurable
  && Object.getPrototypeOf(mainError) === TypeError.prototype
  && Object.getPrototypeOf(foreignError) === other.TypeError.prototype
  && arrayError instanceof TypeError && first.next().value[0] === 'a'
  && foreign.next().value[0] === 'c' && lazyCalls === 0 && calls === 1 && received === marker;
"#,
    );
}
