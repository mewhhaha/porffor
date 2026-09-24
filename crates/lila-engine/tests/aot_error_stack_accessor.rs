//! `Error.prototype.stack` (proposal-error-stack-accessor) through the Wasm-AOT
//! product path: the accessor's shape, the [[ErrorData]]-only getter, the
//! `SetterThatIgnoresPrototypeProperties` setter and per-realm home objects.

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

/// Runs `source`, whose completion value is `"all-ok"` or the `|`-joined
/// labels of the checks that failed.
fn assert_all_ok(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let prelude =
        "var failures = [];\nfunction check(label, ok) { if (!ok) failures.push(label); }\n";
    let epilogue = "\nfailures.length === 0 ? 'all-ok' : failures.join('|');\n";
    let script = format!("{prelude}{source}{epilogue}");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            &script,
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
        .unwrap_or_else(|error| panic!("Error.prototype.stack script failed: {error}\n{script}"));
    assert!(
        outcome.note.contains("all-ok"),
        "{}\n{script}",
        outcome.note
    );
}

#[test]
fn accessor_lives_only_on_error_prototype_with_builtin_function_metadata() {
    assert_all_ok(
        r#"
var desc = Object.getOwnPropertyDescriptor(Error.prototype, 'stack');
check('accessor', typeof desc.get === 'function' && typeof desc.set === 'function');
check('attributes', desc.enumerable === false && desc.configurable === true && !('value' in desc));
check('getter name', desc.get.name === 'get stack' && desc.get.length === 0);
check('setter name', desc.set.name === 'set stack' && desc.set.length === 1);
var threw = false;
try { new desc.get(); } catch (e) { threw = e instanceof TypeError; }
check('getter is not a constructor', threw);
var natives = [Error, EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError];
for (var i = 0; i < natives.length; ++i) {
  var err = new natives[i]('msg');
  check(natives[i].name + ' no own stack', !Object.prototype.hasOwnProperty.call(err, 'stack'));
  check(natives[i].name + ' own names', Object.getOwnPropertyNames(err).join() === 'message');
  check(natives[i].name + ' keys', Object.keys(err).length === 0 && JSON.stringify(err) === '{}');
  if (natives[i] !== Error) {
    check(natives[i].name + ' prototype has no own stack',
      Object.getOwnPropertyDescriptor(natives[i].prototype, 'stack') === undefined);
  }
}
check('prototype keys unchanged',
  Object.getOwnPropertyNames(Error.prototype).sort().join() === 'constructor,message,name,stack,toString');
"#,
    );
}

#[test]
fn getter_answers_from_the_receivers_own_error_data() {
    assert_all_ok(
        r#"
var get = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').get;
check('Error instance', typeof get.call(new Error('m')) === 'string');
check('property access', typeof new TypeError('m').stack === 'string');
check('called without new', typeof get.call(RangeError('m')) === 'string');
check('AggregateError', typeof new AggregateError([], 'm').stack === 'string');
check('subclass', typeof new (class extends SyntaxError {})('m').stack === 'string');
var foreign = Reflect.construct(Error, ['m'], function NotAnError() {});
check('foreign new.target slot', typeof get.call(foreign) === 'string');
check('foreign new.target lookup', foreign.stack === undefined);
check('prototype', get.call(Error.prototype) === undefined && Error.prototype.stack === undefined);
check('NativeError prototype', get.call(URIError.prototype) === undefined);
check('plain object', get.call({}) === undefined);
check('inherits Error.prototype', get.call(Object.create(Error.prototype)) === undefined);
check('instance as prototype', Object.create(new Error('m')).stack === undefined);
check('proxy of error', get.call(new Proxy(new Error('m'), {})) === undefined);
var shadow = new Error('m');
Object.defineProperty(shadow, 'stack', { value: 'own', writable: true, enumerable: true, configurable: true });
check('own data shadows', shadow.stack === 'own' && typeof get.call(shadow) === 'string');
var bad = [undefined, null, true, 1, '', Symbol('s'), 0n];
for (var i = 0; i < bad.length; ++i) {
  var threw = false;
  try { get.call(bad[i]); } catch (e) { threw = e instanceof TypeError; }
  check('primitive receiver ' + i, threw);
}
"#,
    );
}

#[test]
fn setter_ignores_prototype_properties_on_every_receiver_kind() {
    assert_all_ok(
        r#"
var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;
function throwsTypeError(f) { try { f(); } catch (e) { return e instanceof TypeError; } return false; }
var err = new Error('m');
check('returns undefined', set.call(err, 'x') === undefined);
var d = Object.getOwnPropertyDescriptor(err, 'stack');
check('creates own data', d.value === 'x' && d.writable && d.enumerable && d.configurable);
var viaAssignment = new ReferenceError('m');
viaAssignment.stack = 'assigned';
check('assignment', Object.getOwnPropertyDescriptor(viaAssignment, 'stack').value === 'assigned');
var known = new Error('m');
known.stack = 'written';
check('read after assignment', known.stack === 'written');
check('delete own', delete known.stack);
check('read after delete', typeof known.stack === 'string' && known.stack !== 'written');
check('non-string value', throwsTypeError(function () { set.call(new Error('m'), 1); }));
check('missing value', throwsTypeError(function () { set.call(new Error('m')); }));
check('boxed string value', throwsTypeError(function () { set.call({}, new String('s')); }));
check('assignment of null', throwsTypeError(function () { new Error('m').stack = null; }));
check('primitive receiver', throwsTypeError(function () { set.call(1, ''); }));
check('home receiver', throwsTypeError(function () { set.call(Error.prototype, ''); }));
check('home assignment', throwsTypeError(function () { Error.prototype.stack = ''; }));
check('home unchanged', typeof Object.getOwnPropertyDescriptor(Error.prototype, 'stack').get === 'function');
set.call(TypeError.prototype, 'on-native-prototype');
check('other prototype', Object.getOwnPropertyDescriptor(TypeError.prototype, 'stack').value === 'on-native-prototype');
delete TypeError.prototype.stack;
var frozen = Object.freeze(new Error('m'));
check('non-extensible', throwsTypeError(function () { set.call(frozen, 'x'); }) &&
  !Object.prototype.hasOwnProperty.call(frozen, 'stack'));
var fixed = new Error('m');
Object.defineProperty(fixed, 'stack', { value: 'old', writable: true, enumerable: false, configurable: false });
set.call(fixed, 'new');
var fd = Object.getOwnPropertyDescriptor(fixed, 'stack');
check('existing own data keeps attributes', fd.value === 'new' && !fd.enumerable && !fd.configurable);
var readOnly = new Error('m');
Object.defineProperty(readOnly, 'stack', { value: 'old', writable: false, configurable: true });
check('non-writable own', throwsTypeError(function () { set.call(readOnly, 'new'); }));
var seen;
var withAccessor = new Error('m');
Object.defineProperty(withAccessor, 'stack', { get: function () {}, set: function (v) { seen = v; }, configurable: true });
set.call(withAccessor, 'through-setter');
check('own accessor setter', seen === 'through-setter');
var fn = function () {};
set.call(fn, 'fn');
check('function receiver', fn.stack === 'fn');
var arr = [];
set.call(arr, 'arr');
check('array receiver', arr.stack === 'arr' && arr.length === 0);
var log = [];
var target = {};
var proxy = new Proxy(target, {
  getOwnPropertyDescriptor: function (t, k) { log.push('gopd:' + String(k)); return Reflect.getOwnPropertyDescriptor(t, k); },
  defineProperty: function (t, k, desc) {
    log.push('define:' + String(k) + ':' + Object.keys(desc).sort().join(','));
    return Reflect.defineProperty(t, k, desc);
  },
  set: function () { log.push('set'); return true; }
});
set.call(proxy, 'p');
check('proxy create path', log.join(';') === 'gopd:stack;define:stack:configurable,enumerable,value,writable' && target.stack === 'p');
log = [];
set.call(proxy, 'q');
check('proxy set path', log.join(';') === 'gopd:stack;set');
var rejecting = new Proxy({}, { defineProperty: function () { return false; } });
check('rejected define', throwsTypeError(function () { set.call(rejecting, 'v'); }));
var wrapped = [];
set.call(new Proxy(Error.prototype, {
  set: function (t, k, v) { wrapped.push(String(k) + '=' + v); return true; }
}), 'w');
check('proxy of home is not home', wrapped.join() === 'stack=w');
"#,
    );
}

#[test]
fn each_realm_has_its_own_accessor_and_home_object() {
    assert_all_ok(
        r#"
var other = __lilaCreateRealm().global;
var mine = Object.getOwnPropertyDescriptor(Error.prototype, 'stack');
var theirs = Object.getOwnPropertyDescriptor(other.Error.prototype, 'stack');
check('distinct getters', typeof theirs.get === 'function' && theirs.get !== mine.get);
check('distinct setters', typeof theirs.set === 'function' && theirs.set !== mine.set);
check('their getter on my error', typeof theirs.get.call(new Error('m')) === 'string');
check('my getter on their error', typeof mine.get.call(new other.Error('m')) === 'string');
check('my getter on their prototype', mine.get.call(other.Error.prototype) === undefined);
var foreignError = new other.Error('m');
mine.set.call(foreignError, 'crossed');
check('my setter on their error', Object.getOwnPropertyDescriptor(foreignError, 'stack').value === 'crossed');
var realmError;
try { mine.set.call(other.Error.prototype, 'x'); } catch (e) { realmError = e; }
check('their home rejects through their setter',
  realmError instanceof other.TypeError && !(realmError instanceof TypeError));
var badReceiver;
try { theirs.get.call(1); } catch (e) { badReceiver = e; }
check('their getter throws their TypeError', badReceiver instanceof other.TypeError);
"#,
    );
}
