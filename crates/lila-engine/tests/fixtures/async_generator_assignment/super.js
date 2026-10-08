function check(ok, label) { if (!ok) throw label; }
async function step(iterator, input, expected, done) {
  var result = await iterator.next(input);
  check(result.value === expected && result.done === done, 'super-assignment-step');
}
async function typeError(promise) {
  try { await promise; } catch (error) { check(error instanceof TypeError, 'super-type-error'); return; }
  throw 'missing-super-type-error';
}
var trace = [], expectedReceiver, written;
var original = {
  get value() { trace.push('get'); return 3; },
  set value(value) { check(this === expectedReceiver, 'super-receiver'); trace.push('set'); written = value; },
  set later(value) { check(this === expectedReceiver, 'super-plain-receiver'); trace.push('set-later'); written = value; }
};
var replacement = {
  get value() { throw 'new-super-base-get'; },
  set value(value) { throw 'new-super-base-set'; },
  set later(value) { throw 'new-super-base-plain-set'; }
};
var object = {
  __proto__: original,
  async *compound() { yield (super[await (yield 'key')] += await (yield 'rhs')); },
  async *plain() { yield (super[await (yield 'key')] = await (yield 'rhs')); },
  async *and() { yield (super[await (yield 'key')] &&= await (yield 'rhs')); },
  async *or() { yield (super[await (yield 'key')] ||= await (yield 'rhs')); }
};
async function run() {
  expectedReceiver = { marker: 7 };
  var rawKey = { [Symbol.toPrimitive]: function () { trace.push('key'); return 'value'; } };
  var iterator = object.compound.call(expectedReceiver);
  await step(iterator, undefined, 'key', false);
  await step(iterator, rawKey, 'rhs', false);
  check(trace.join(',') === 'key,get', 'super-get-before-rhs');
  Object.setPrototypeOf(object, replacement);
  rawKey[Symbol.toPrimitive] = function () { throw 'super-key-coerced-twice'; };
  gc();
  await step(iterator, 4, 7, false);
  check(written === 7 && trace.join(',') === 'key,get,set', 'retained-super-base-and-key');
  await step(iterator, undefined, undefined, true);

  trace = []; Object.setPrototypeOf(object, original);
  rawKey = { [Symbol.toPrimitive]: function () { trace.push('key'); return 'value'; } };
  iterator = object.plain.call(expectedReceiver);
  await step(iterator, undefined, 'key', false);
  await step(iterator, rawKey, 'rhs', false);
  check(trace.length === 0, 'plain-super-no-get-or-key-conversion');
  Object.setPrototypeOf(object, replacement);
  rawKey[Symbol.toPrimitive] = function () { trace.push('later-key'); return 'later'; };
  await step(iterator, 11, 11, false);
  check(written === 11 && trace.join(',') === 'later-key,set-later', 'raw-key-after-rhs-original-super-base');
  await step(iterator, undefined, undefined, true);

  trace = []; Object.setPrototypeOf(object, original);
  iterator = object.or.call(expectedReceiver);
  await step(iterator, undefined, 'key', false);
  await step(iterator, 'value', 3, false);
  await step(iterator, undefined, undefined, true);
  check(trace.join(',') === 'get', 'super-logical-skips-rhs-and-set');
  trace = [];
  iterator = object.and.call(expectedReceiver);
  await step(iterator, undefined, 'key', false);
  await step(iterator, 'value', 'rhs', false);
  Object.setPrototypeOf(object, replacement); gc();
  await step(iterator, 17, 17, false);
  await step(iterator, undefined, undefined, true);
  check(written === 17 && trace.join(',') === 'get,set', 'selected-super-logical-original-base');

  trace = []; Object.setPrototypeOf(object, null);
  rawKey = { [Symbol.toPrimitive]: function () { trace.push('invalid-key'); return 'value'; } };
  iterator = object.plain.call(expectedReceiver);
  await step(iterator, undefined, 'key', false);
  await step(iterator, rawKey, 'rhs', false);
  Object.setPrototypeOf(object, original);
  await typeError(iterator.next(19));
  check(trace.length === 0, 'saved-null-super-base-rejected-after-rhs-before-key');
  Object.setPrototypeOf(object, null);
  iterator = object.compound.call(expectedReceiver);
  await step(iterator, undefined, 'key', false);
  await typeError(iterator.next(rawKey));
  check(trace.length === 0, 'null-super-get-before-rhs-and-key-conversion');
}
run().then(function () { print('mixed-super-assignment:ok'); }, function (error) { print(error); throw error; });
