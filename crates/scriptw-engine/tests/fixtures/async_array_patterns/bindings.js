function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 17 }; whole.self = whole;
whole[Symbol.toPrimitive] = function () { throw 'unexpected-value-conversion'; };
function stream(values, events, label) {
  var source = { closes: 0 };
  source[Symbol.iterator] = function () {
    events.push(label + 'acquire');
    var position = 0;
    var record = { return: function () {
      source.closes++; events.push(label + 'close');
      if (source.closeError !== undefined) throw source.closeError;
      return {};
    } };
    Object.defineProperty(record, 'next', { configurable: true, get: function () {
      events.push(label + 'next-get');
      return function () {
        var index = position++; events.push(label + 'step' + index);
        return {
          get done() { events.push(label + 'done' + index); return index >= values.length; },
          get value() { events.push(label + 'value' + index); return values[index]; }
        };
      };
    } });
    source.record = record;
    return record;
  };
  return source;
}
async function failedProtocol(kind) {
  var closes = 0;
  var input = { [Symbol.iterator]: function () { return {
    next: function () {
      if (kind === 'next') throw whole;
      if (kind === 'result') return 7;
      return {
        get done() { if (kind === 'done') throw whole; return false; },
        get value() { throw whole; }
      };
    },
    return: function () { closes++; return {}; }
  }; } };
  try { const [value = await 0] = input; throw 'missing-protocol-error'; }
  catch (error) { check(kind === 'result' ? error instanceof TypeError : error === whole, 'protocol-error-identity'); }
  check(closes === 0, 'protocol-failure-marks-done');
}
async function run() {
  var events = [], source = stream([11, undefined, whole], events, '');
  const [, first = await Promise.resolve(whole).then(function (value) {
    Object.defineProperty(source.record, 'next', { value: function () { throw 'reloaded-next'; } });
    gc(); return value;
  }), second = await 2] = source;
  check(first === whole && second === whole, 'cached-next-and-whole-value');
  check(events.join(',') === 'acquire,next-get,step0,done0,step1,done1,value1,step2,done2,value2,close', 'elision-and-close-order');
  check(source.closes === 1, 'normal-truncation-closes-once');

  var observations = 0;
  var skipped = { get then() { observations++; throw whole; } };
  source = stream([whole, 8], [], '');
  let [present = await skipped] = source;
  check(present === whole && observations === 0 && source.closes === 1, 'defined-value-skips-promise-observation');

  var nullBase = null;
  const [shorted = nullBase?.[await skipped]()] = [];
  check(shorted === undefined && observations === 0, 'known-nullish-tail-skips-await-and-call');
  var receiver = { get selected() { return function () { check(this === receiver, 'optional-call-receiver'); return whole; }; } };
  const [called = receiver?.[await 'selected']?.()] = [];
  check(called === whole, 'awaited-key-with-eager-optional-call');

  events = [];
  var inner = stream([undefined, 8], events, 'inner-');
  source = stream([{ inner: inner }, 9], events, 'outer-');
  let [{ [await 'inner']: [nested = await Promise.resolve(whole)] }] = source;
  check(nested === whole && events.slice(-2).join(',') === 'inner-close,outer-close', 'nested-record-close-order');

  events = [];
  inner = stream([undefined], events, 'inner-');
  source = stream([inner], events, 'outer-');
  inner.closeError = 23; source.closeError = 24;
  try { let [[failure = await Promise.reject(whole)]] = source; throw 'missing-default-rejection'; }
  catch (error) { check(error === whole && events.slice(-2).join(',') === 'inner-close,outer-close', 'rejection-closes-all-original-records'); }

  for (var kind of ['next', 'result', 'done', 'value']) await failedProtocol(kind);
  try { const [tdz = await Promise.resolve(tdz)] = []; throw 'missing-tdz'; }
  catch (error) { check(error instanceof ReferenceError, 'original-lexical-tdz'); }
  var label = 7;
  var [label = class { static seen = label }, suspended = await 1] = [];
  check(label.name === 'label' && label.seen === 7 && suspended === 1, 'inferred-class-has-no-synthetic-name-binding');
  const [fn = function () {}, arrow = () => 1, wait = await 2] = [];
  check(fn.name === 'fn' && arrow.name === 'arrow' && wait === 2, 'original-anonymous-default-names');
  class Parent {}
  const [suspendedClass = class extends (await Parent) {
    [await 'value']() { return whole; }
    async nested() { return await Promise.resolve(whole); }
  }] = [];
  var instance = new suspendedClass();
  check(suspendedClass.name === 'suspendedClass' && instance.value() === whole && await instance.nested() === whole, 'class-default-evaluation-and-independent-method-await');

  var readers = [];
  for (let item of [1, 2]) {
    const [value = await Promise.resolve(item)] = [];
    readers.push(() => value);
  }
  gc(); check(readers[0]() === 1 && readers[1]() === 2, 'original-per-iteration-captured-cells');
  const [existing] = await Promise.resolve([whole]);
  check(existing === whole, 'existing-whole-awaited-initializer');
}
run().then(function () { print('async-array-bindings:ok'); }, function (error) { print(error); throw error; });
