use std::path::PathBuf;

use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};
use lila_ir::HostSurfacePolicy;
use lila_test262::{load_preludes, LocalHarnessSource, SuiteConfig};

fn assert_harness(source: &str, includes: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let config = SuiteConfig {
        suite_root: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test262/vendor/test262"),
        local_harness: LocalHarnessSource::EmbeddedWasmAot,
        ..SuiteConfig::default()
    };
    let preludes = load_preludes(&config).expect("embedded pinned harness");
    let mut program = String::new();
    for name in ["assert.js", "sta-preamble.js"]
        .into_iter()
        .chain(includes.iter().copied())
    {
        program.push_str(&preludes.get(name).expect("complete named section").contents);
    }
    program.push_str(source);
    program.push_str("\ntrue;");
    let result = Engine::new(RealmBuilder::new().build())
        .run_script(
            &program,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("pinned harness failed: {error}\n{source}"));
    assert!(result.note.contains("boolean(true)"), "{}", result.note);
}

#[test]
fn assertions_require_true_and_throw_test262_error_objects() {
    assert_harness(
        r#"
function failure(callback) {
  var caught;
  try { callback(); } catch (error) { caught = error; }
  if (caught === undefined || caught.constructor !== Test262Error)
    throw new Error('Expected the Test262Error constructor');
  return caught;
}
assert(true);
if (failure(function () { assert(1); }).message !== 'Expected true but got 1')
  throw new Error('strict true assertion');
failure(function () { assert.sameValue({}, {}); });
failure(function () { assert.sameValue(0, -0); });
failure(function () { assert.notSameValue(NaN, NaN); });
assert.sameValue(NaN, NaN);
assert.throws(TypeError, function () { throw new TypeError('expected'); });
failure(function () { assert.throws(TypeError, function () { throw null; }); });
failure(function () { assert.throws(TypeError, function () {}); });
failure(function () { assert.throws(TypeError, 7); });
failure(function () { assert.throws(RangeError, function () { throw new TypeError(); }); });
var originalSameValue = assert._isSameValue;
assert._isSameValue = function () { throw 'marker'; };
var diagnostic = failure(function () { assert.sameValue(1, 1, 'context'); });
assert._isSameValue = originalSameValue;
if (diagnostic.message !== 'context (_isSameValue operation threw) marker')
  throw new Error('SameValue hook diagnostic');
var explicit = new Test262Error('message');
if (explicit.message !== 'message' || explicit.toString() !== 'Test262Error: message')
  throw new Error('Test262Error instance methods');
"#,
        &[],
    );
}

#[test]
fn compare_array_and_value_formatting_follow_upstream_semantics() {
    assert_harness(
        r#"
assert.compareArray([NaN, -0], [NaN, -0]);
if (compareArray([0], [-0])) throw new Error('SameValue array comparison');
var caught;
try { assert.compareArray([1], [2], 'context'); } catch (error) { caught = error; }
if (!caught || caught.constructor !== Test262Error
    || caught.message !== 'Actual [1] and expected [2] should have the same contents. context')
  throw new Error('array mismatch diagnostic');
if (assert._toString(-0) !== '-0' || assert._toString(1n) !== '1n'
    || assert._toString('x') !== '"x"') throw new Error('identity-free formatting');
var value = { toString() { throw new TypeError(); } };
if (assert._toString(value) !== '[object Object]') throw new Error('formatting fallback');
"#,
        &[],
    );
}

#[test]
fn property_verification_checks_undefined_fields_and_restores_destructive_probes() {
    assert_harness(
        r#"
var caught;
try { verifyProperty({ key: 1 }, 'key', { value: undefined }); }
catch (error) { caught = error; }
if (!caught || caught.constructor !== Test262Error) throw new Error('undefined value was skipped');
caught = undefined;
try { verifyProperty({ key: 1 }, 'key', { invalid: true }); }
catch (error) { caught = error; }
if (!caught || caught.constructor !== Test262Error) throw new Error('invalid field was skipped');
var object = { key: NaN };
verifyProperty(object, 'key', {
  value: NaN, writable: true, enumerable: true, configurable: true
}, { restore: true });
var descriptor = Object.getOwnPropertyDescriptor(object, 'key');
if (!descriptor || !Number.isNaN(descriptor.value) || !descriptor.writable
    || !descriptor.enumerable || !descriptor.configurable) throw new Error('restore failed');
verifyProperty(object, 'key', { configurable: true });
if (Object.prototype.hasOwnProperty.call(object, 'key')) throw new Error('deletion probe skipped');
var symbol = Symbol('key');
var symbols = { [symbol]: 1 };
verifyProperty(symbols, symbol, { enumerable: true }, { restore: true });
"#,
        &["propertyHelper.js"],
    );
}

#[test]
fn property_helpers_use_captured_primordials_and_check_callable_names() {
    assert_harness(
        r#"
var object = { key: 1 };
var callable = function actual(value) {};
var holder = {};
Object.defineProperty(holder, 'method', {
  value: callable, writable: true, enumerable: false, configurable: true
});
Object.getOwnPropertyDescriptor = function () { throw 'mutated descriptor method'; };
Object.defineProperty = function () { throw 'mutated definition method'; };
Object.prototype.hasOwnProperty = function () { throw 'mutated hasOwnProperty method'; };
Array.prototype.push = function () { throw 'mutated push method'; };
verifyProperty(object, 'key', { value: 1, writable: true, configurable: true }, { restore: true });
var caught;
try { verifyCallableProperty(holder, 'method', 'wrong', 1, undefined, { restore: true }); }
catch (error) { caught = error; }
if (!caught || caught.constructor !== Test262Error
    || caught.message.indexOf("name") === -1) throw new Error('callable name was skipped');
"#,
        &["propertyHelper.js"],
    );
}
