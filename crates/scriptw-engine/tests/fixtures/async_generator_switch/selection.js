function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 81 }; whole.self = whole;
whole[Symbol.toPrimitive] = function () { throw 'switch-coerced-whole-value'; };
var events = [], selected = 'outside', retainedReader;

async function* selection() {
  switch (await (yield 'discriminant')) {
    case await (yield 'first-selector'):
      events.push('first'); yield 'first-body'; break;
    default:
      events.push('default'); yield 'default-body';
    case await (yield 'last-selector'):
      events.push('last'); yield whole; break;
  }
  return whole;
}

async function* records(input) {
  switch (await input) {
    case 1:
      let shared = await (yield 'shared');
      retainedReader = function () { return shared; };
      yield hoisted();
      await Promise.resolve(0); gc();
      shared = whole;
    default:
      yield retainedReader();
      function hoisted() { return shared; }
  }
  return retainedReader;
}

async function* discriminatorScope() {
  try {
    switch (selected) {
      case typeof selected: yield 'incorrect-selector-read'; break;
      default: let selected = 1;
    }
  } catch (error) { check(error instanceof ReferenceError, 'selector-sees-original-case-block-tdz'); return 'tdz'; }
  throw 'missing-case-block-tdz';
}

async function* eager(input) {
  switch (input) { default: return hoisted(); function hoisted() { return whole; } }
}

async function run() {
  var iterator = selection(), result = await iterator.next();
  check(result.value === 'discriminant' && events.length === 0, 'discriminant-outside-case-block');
  result = await iterator.next(Promise.resolve(whole));
  check(result.value === 'first-selector' && events.length === 0, 'first-selector-before-body');
  result = await iterator.next(Promise.resolve({}));
  check(result.value === 'last-selector' && events.length === 0, 'default-waits-for-later-selector');
  whole.marker = 82; gc();
  result = await iterator.next(Promise.resolve(whole));
  check(result.value === whole && result.value.self === whole && events.join(',') === 'last', 'whole-strict-match-no-default-no-conversion');
  result = await iterator.next(); check(result.done && result.value === whole, 'matched-body-does-not-replay-selector');
  events = []; iterator = selection(); await iterator.next(); await iterator.next({}); await iterator.next({});
  result = await iterator.next({});
  check(result.value === 'default-body' && events.join(',') === 'default', 'fallback-after-all-failed-selectors');
  result = await iterator.next();
  check(result.value === whole && events.join(',') === 'default,last', 'fallthrough-skips-selector');
  result = await iterator.next(); check(result.done && result.value === whole, 'fallback-normal-exit');
  iterator = records(Promise.resolve(1)); result = await iterator.next(); check(result.value === 'shared', 'captured-case-initializer');
  result = await iterator.next(Promise.resolve(17)); check(result.value === 17, 'hoisted-function-uses-shared-case-cell');
  result = await iterator.next(); check(result.value === whole && retainedReader() === whole, 'fallthrough-retains-same-captured-cell-through-gc');
  result = await iterator.next(); check(result.done && result.value === retainedReader && retainedReader() === whole, 'escaping-case-closure');
  iterator = discriminatorScope(); result = await iterator.next();
  check(result.done && result.value === 'tdz' && selected === 'outside', 'discriminant-precedes-case-block-instantiation');
  iterator = eager(0); result = await iterator.next(); check(result.done && result.value === whole, 'eager-switch-still-owns-case-phases');
}
run().then(function () { print('mixed-async-generator-switch-selection:ok'); }, function (error) { print(error); throw error; });
