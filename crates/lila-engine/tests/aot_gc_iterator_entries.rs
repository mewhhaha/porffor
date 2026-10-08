use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_gc_iterator_modes(source: &str, line: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("GC iterator control must compile and execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(line.into())],
            "{source}"
        );
    }
}

#[test]
fn gc_array_iterator_advances_before_abrupt_get_and_revalidates_typed_buffers() {
    assert_gc_iterator_modes(
        r#"
function check(condition, label) { if (!condition) throw new Error(label); }
var token = { token: 'element' };
var gets = [];
var arrayLike = {
  get length() { gets.push('length'); return 3; },
  get 0() { gets.push('0'); throw token; },
  get 1() { gets.push('1'); return 17; },
  get 2() { gets.push('2'); return 23; }
};
var iterator = Array.prototype.values.call(arrayLike);
var caught;
try { iterator.next(); } catch (error) { caught = error; }
check(caught === token, 'whole original element throw');
var step = iterator.next();
check(step.value === 17 && step.done === false, 'no replay after abrupt Get');
check(gets.join(',') === 'length,0,length,1', 'length before each live Get');
step = iterator.next();
check(step.value === 23 && step.done === false, 'next live value');
check(iterator.next().done === true, 'array exhausted');
Object.defineProperty(arrayLike, 'length', { get: function () { throw token; } });
check(iterator.next().done === true, 'terminal array never reads length');

var buffer = new ArrayBuffer(4, { maxByteLength: 8 });
var fixed = new Uint8Array(buffer, 0, 2);
fixed[0] = 5; fixed[1] = 6;
var fixedIterator = fixed.values();
check(fixedIterator.next().value === 5, 'initial fixed view');
buffer.resize(1);
caught = undefined;
try { fixedIterator.next(); } catch (error) { caught = error; }
check(caught instanceof TypeError, 'out of bounds is abrupt');
buffer.resize(4);
fixed[1] = 9;
step = fixedIterator.next();
check(step.value === 9 && step.done === false, 'invalid view did not advance');
check(fixedIterator.next().done === true, 'fixed iterator done');
buffer.resize(8);
check(fixedIterator.next().done === true, 'done remains permanent after grow');

var trackingBuffer = new ArrayBuffer(1, { maxByteLength: 4 });
var tracking = new Uint8Array(trackingBuffer);
tracking[0] = 31;
var trackingIterator = tracking.entries();
step = trackingIterator.next();
check(step.value[0] === 0 && step.value[1] === 31, 'entries pair');
trackingBuffer.resize(3);
tracking[1] = 37; tracking[2] = 41;
check(trackingIterator.next().value[1] === 37, 'tracking growth');
trackingBuffer.resize(1);
check(trackingIterator.next().done === true, 'tracking shrink terminates');
trackingBuffer.resize(4);
check(trackingIterator.next().done === true, 'tracking done never restarts');

var detachedBuffer = new ArrayBuffer(2);
var detachedIterator = new Uint8Array(detachedBuffer).keys();
$262.detachArrayBuffer(detachedBuffer);
caught = undefined;
try { detachedIterator.next(); } catch (error) { caught = error; }
check(caught instanceof TypeError, 'detached keys require live bounds');
print('gc-iterator-array:ok');
262;
"#,
        "gc-iterator-array:ok",
    );
}

#[test]
fn gc_string_results_and_iterator_from_records_preserve_called_realms_and_identity() {
    assert_gc_iterator_modes(
        r#"
function check(condition, label) { if (!condition) throw new Error(label); }
var input = '\uD800A\uDC00\uD83D\uDE00';
var iterator = input[Symbol.iterator]();
var pieces = [];
for (var count = 0; count < 4; ++count) {
  var step = iterator.next();
  check(step.done === false, 'string active'); pieces.push(step.value);
}
check(pieces[0] === '\uD800' && pieces[1] === 'A' && pieces[2] === '\uDC00' && pieces[3] === '\uD83D\uDE00', 'UTF16 code points preserve lone surrogates');
check(iterator.next().done === true && iterator.next().value === undefined, 'terminal String result');

var foreign = $262.createRealm().global;
var getPrototype = Object.getPrototypeOf;
var ownNames = Object.getOwnPropertyNames;
var localObjectPrototype = Object.prototype;
var foreignObjectPrototype = foreign.Object.prototype;
var foreignTypeErrorPrototype = foreign.TypeError.prototype;
var localNext = getPrototype(''[Symbol.iterator]()).next;
var foreignNext = getPrototype(foreign.String.prototype[Symbol.iterator].call('')).next;
var foreignFrom = foreign.Iterator.from;
var source = {
  get next() { ++nextGets; return nextMethod; },
  get return() { ++returnGets; return returnMethod; }
};
var nextGets = 0;
var returnGets = 0;
var calls = 0;
var nextMethod = new Proxy(function () {}, {
  apply: function (target, receiver, args) {
    check(receiver === source && args.length === 0, 'cached next receiver and argc');
    ++calls; return 71;
  }
});
var returnMethod = new Proxy(function () {}, {
  apply: function (target, receiver, args) {
    check(receiver === source && args.length === 0, 'fresh return receiver and argc');
    return 79;
  }
});
var wrapped = foreignFrom(source);
var wrapperPrototype = getPrototype(wrapped);
var wrapperNext = wrapperPrototype.next;
var wrapperReturn = wrapperPrototype.return;
check(nextGets === 1 && calls === 0, 'acquisition caches without call');
check(ownNames(wrapped).length === 0, 'private wrapper record is not JS properties');
// The accessor has no setter; use a replacement data property explicitly.
Object.defineProperty(source, 'next', { value: undefined, configurable: true });
check(wrapperNext.call(wrapped, 'ignored') === 71 && calls === 1 && nextGets === 1, 'cached callable result forwarded');
check(wrapperReturn.call(wrapped) === 79 && wrapperReturn.call(wrapped) === 79 && returnGets === 2, 'return acquired each time');
returnMethod = null;
var identity = Object.create(foreign.Iterator.prototype);
Object.defineProperty(identity, 'next', { get: function () { ++identityGets; return 0; } });
var identityGets = 0;
check(foreignFrom(identity) === identity && identityGets === 1, 'identity after next Get');

var originalLocalObject = globalThis.Object;
var originalForeignObject = foreign.Object;
var originalForeignTypeError = foreign.TypeError;
try {
  globalThis.Object = { poisoned: true };
  foreign.Object = { poisoned: true };
  foreign.TypeError = { poisoned: true };
  var localResult = localNext.call(foreign.String.prototype[Symbol.iterator].call('x'));
  check(localResult.value === 'x' && getPrototype(localResult) === localObjectPrototype, 'local called result Realm');
  var foreignResult = foreignNext.call('y'[Symbol.iterator]());
  check(foreignResult.value === 'y' && getPrototype(foreignResult) === foreignObjectPrototype, 'foreign called result Realm');
  var absentReturn = wrapperReturn.call(wrapped);
  check(absentReturn.done === true && absentReturn.value === undefined && getPrototype(absentReturn) === foreignObjectPrototype, 'fresh defining Realm return result');
  var traps = 0;
  var invalid = new Proxy({ '$LilaIteratorFromWrapper': true }, { get: function () { ++traps; throw source; } });
  var caught;
  try { wrapperNext.call(invalid); } catch (error) { caught = error; }
  check(getPrototype(caught) === foreignTypeErrorPrototype && traps === 0, 'private brand without traps');
} finally {
  globalThis.Object = originalLocalObject;
  foreign.Object = originalForeignObject;
  foreign.TypeError = originalForeignTypeError;
}
print('gc-iterator-string-from:ok');
262;
"#,
        "gc-iterator-string-from:ok",
    );
}
