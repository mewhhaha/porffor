function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 73 }; whole.self = whole;
whole[Symbol.toPrimitive] = function () { throw 'whole-was-coerced'; };
function sequence(values, events, name) {
  var position = 0, record = {
    next: function () {
      events.push(name + ':next');
      if (position === values.length) return { done: true };
      var value = values[position++];
      return { done: false, get value() { events.push(name + ':value'); return value; } };
    },
    return: function () { events.push(name + ':close'); return { done: true }; }
  }, source = {};
  Object.defineProperty(record, 'next', { get: function () { events.push(name + ':next-get'); return next; }, configurable: true });
  function next() {
    events.push(name + ':next');
    if (position === values.length) return { done: true };
    var value = values[position++];
    return { done: false, get value() { events.push(name + ':value'); return value; } };
  }
  source[Symbol.iterator] = function () { events.push(name + ':iterator'); return record; };
  return { source: source, record: record };
}

async function run() {
  var events = [], inner = sequence([undefined], events, 'inner'), outer = sequence([{ inner: inner.source }], events, 'outer');
  async function* nested(input) {
    const [{ [yield 'key']: [received = await (yield 'default')] }] = input;
    gc(); yield received;
  }
  var iterator = nested(outer.source);
  check((await iterator.next()).value === 'key', 'computed-object-key-after-outer-step');
  check(events.join(',') === 'outer:iterator,outer:next-get,outer:next,outer:value', 'original-outer-acquisition-and-value-order');
  check((await iterator.next('inner')).value === 'default', 'nested-default-lazy-yield');
  Object.defineProperty(inner.record, 'next', { value: function () { throw 'cached-next-was-reselected'; } });
  check((await iterator.next(41)).value === 41, 'nested-mixed-default-awaited-received-value');
  check(events.slice(-2).join(',') === 'inner:close,outer:close', 'nested-close-inner-before-outer');
  check((await iterator.next()).done === true, 'nested-pattern-completes');

  var forbidden = { get then() { throw 'skipped-default-then'; } };
  async function* skipped(input) { const [received = await (yield forbidden)] = input; yield received; }
  iterator = skipped([17]);
  check((await iterator.next()).value === 17 && (await iterator.next()).done === true, 'present-value-skips-complete-default');

  var reader;
  async function* tdz(input) { const [received = (reader = function () { return received; }, await (yield 'tdz'))] = input; yield received; }
  iterator = tdz([undefined]); check((await iterator.next()).value === 'tdz', 'lexical-default-before-initialize');
  try { reader(); throw 'missing-pattern-tdz'; } catch (error) { check(error instanceof ReferenceError, 'same-original-tdz-cell'); }
  check((await iterator.next(23)).value === 23, 'lexical-initialization-after-mixed-default'); gc();
  check(reader() === 23 && (await iterator.next()).done === true, 'captured-original-cell-after-gc');

  async function* heads(input) { for (let [index = await (yield 'head'), step] = input; index < 2; index++) { yield function () { return [index, step]; }; } }
  iterator = heads([undefined, 9]); check((await iterator.next()).value === 'head', 'classic-pattern-head-suspends-once');
  var first = (await iterator.next(0)).value, second = (await iterator.next()).value;
  check((await iterator.next()).done === true, 'classic-pattern-head-exits'); gc();
  check(first().join(',') === '0,9' && second().join(',') === '1,9', 'actual-for-per-iteration-pattern-cells');

  async function* interrupted(input) { const [received = await (yield 'pause')] = input; yield received; }
  events = []; var source = sequence([undefined], events, 'throw'); iterator = interrupted(source.source);
  check((await iterator.next()).value === 'pause', 'injected-throw-at-real-default-yield');
  try { await iterator.throw(whole); throw 'missing-whole-throw'; } catch (error) { check(error === whole, 'whole-injected-throw-identity'); }
  check(events[events.length - 1] === 'throw:close', 'injected-throw-closes-original-record');

  events = []; source = sequence([undefined], events, 'return'); iterator = interrupted(source.source);
  await iterator.next(); var returned = await iterator.return(whole);
  check(returned.done === true && returned.value === whole && events[events.length - 1] === 'return:close', 'injected-return-close-and-whole-identity');

  var closeCount = 0, broken = {};
  broken[Symbol.iterator] = function () { return { next: function () { throw whole; }, return: function () { closeCount++; return {}; } }; };
  try { await interrupted(broken).next(); throw 'missing-step-error'; } catch (error) { check(error === whole && closeCount === 0, 'step-failure-done-prevents-close'); }
  var nullish = interrupted(null);
  try { await nullish.next(); throw 'missing-nullish-acquisition'; } catch (error) { check(error instanceof TypeError && closeCount === 0, 'acquisition-outside-own-close'); }
}
run().then(function () { print('mixed-async-generator-pattern-bindings:ok'); }, function (error) { print(error); throw error; });
