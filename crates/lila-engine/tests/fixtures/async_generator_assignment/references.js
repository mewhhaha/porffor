function check(ok, label) { if (!ok) throw label; }
async function step(iterator, input, expected, done) {
  var result = await iterator.next(input);
  check(result.value === expected && result.done === done, 'assignment-step');
}
async function rejects(promise, expected) {
  try { await promise; } catch (error) {
    check(expected === TypeError ? error instanceof TypeError : error === expected, 'assignment-error');
    return;
  }
  throw 'missing-assignment-error';
}

var trace = [], written, receiver;
var left = { [Symbol.toPrimitive]: function () { trace.push('left'); return 3; } };
var right = { [Symbol.toPrimitive]: function () { trace.push('right'); return 4; } };
var key = { [Symbol.toPrimitive]: function () { trace.push('key'); return 'value'; } };
receiver = {
  get value() { trace.push('get'); return left; },
  set value(value) { check(this === receiver, 'original-receiver'); trace.push('set'); written = value; }
};
async function* compound() {
  yield ((await (yield 'base'))[await (yield 'key')] += await (yield 'rhs'));
}
async function* plain(base, rawKey) {
  (await (yield 'base'))[rawKey] = await (yield 'rhs');
}
async function* logical(object) {
  yield (object[await (yield 'and-key')] &&= await (yield 'and-rhs'));
  yield (object[await (yield 'or-key')] ||= await (yield 'or-rhs'));
  yield (object[await (yield 'nullish-key')] ??= await (yield 'nullish-rhs'));
}
class Box {
  #value = 1;
  gets = 0;
  sets = 0;
  get #slot() { this.gets++; return this.#value; }
  set #slot(value) { this.sets++; this.#value = value; }
  read() { return this.#value; }
  replace(value) { this.#value = value; }
  async *compound() {
    yield ((await (yield 'private-base')).#slot += await (yield 'private-rhs'));
  }
  async *plain() {
    (await (yield 'private-base')).#value = await (yield 'private-rhs');
  }
  async *logical() {
    yield ((await (yield 'private-base')).#slot ||= await (yield 'private-rhs'));
  }
}

async function run() {
  var iterator = compound();
  await step(iterator, undefined, 'base', false);
  await step(iterator, receiver, 'key', false);
  check(trace.length === 0, 'no-get-before-key');
  await step(iterator, key, 'rhs', false);
  check(trace.join(',') === 'key,get', 'get-before-rhs-without-numeric-coercion');
  key[Symbol.toPrimitive] = function () { throw 'key-coerced-twice'; };
  gc();
  await step(iterator, right, 7, false);
  check(written === 7 && trace.join(',') === 'key,get,left,right,set', 'compound-order');
  await step(iterator, undefined, undefined, true);

  iterator = compound();
  await step(iterator, undefined, 'base', false);
  await step(iterator, null, 'key', false);
  await rejects(iterator.next('value'), TypeError);
  var keyCalls = 0;
  iterator = plain(null, { [Symbol.toPrimitive]: function () { keyCalls++; return 'value'; } });
  await step(iterator, undefined, 'base', false);
  await step(iterator, null, 'rhs', false);
  await rejects(iterator.next(9), TypeError);
  check(keyCalls === 0, 'plain-nullish-put-after-rhs-before-key-conversion');

  var object = { a: 0, o: 2, n: 3 }, sets = 0;
  var proxy = new Proxy(object, { set: function (target, name, value) { sets++; target[name] = value; return true; } });
  iterator = logical(proxy);
  await step(iterator, undefined, 'and-key', false);
  await step(iterator, 'a', 0, false);
  await step(iterator, undefined, 'or-key', false);
  await step(iterator, 'o', 2, false);
  await step(iterator, undefined, 'nullish-key', false);
  await step(iterator, 'n', 3, false);
  await step(iterator, undefined, undefined, true);
  check(sets === 0, 'skipped-logical-arms-do-not-set');
  object.a = 1; object.o = 0; object.n = null;
  iterator = logical(proxy);
  await step(iterator, undefined, 'and-key', false);
  await step(iterator, 'a', 'and-rhs', false);
  await step(iterator, 8, 8, false);
  await step(iterator, undefined, 'or-key', false);
  await step(iterator, 'o', 'or-rhs', false);
  await step(iterator, 9, 9, false);
  await step(iterator, undefined, 'nullish-key', false);
  await step(iterator, 'n', 'nullish-rhs', false);
  await step(iterator, 10, 10, false);
  await step(iterator, undefined, undefined, true);
  check(sets === 3 && object.a === 8 && object.o === 9 && object.n === 10, 'selected-logical-put');

  var box = new Box();
  iterator = box.compound();
  await step(iterator, undefined, 'private-base', false);
  await step(iterator, box, 'private-rhs', false);
  check(box.gets === 1 && box.sets === 0, 'private-get-before-rhs');
  box.replace(100); gc();
  await step(iterator, 3, 4, false);
  check(box.read() === 4 && box.sets === 1, 'private-old-value-and-receiver-retained');
  await step(iterator, undefined, undefined, true);
  iterator = box.logical();
  await step(iterator, undefined, 'private-base', false);
  await step(iterator, box, 4, false);
  await step(iterator, undefined, undefined, true);
  check(box.sets === 1, 'private-logical-skip');
  iterator = box.compound();
  await step(iterator, undefined, 'private-base', false);
  await rejects(iterator.next({}), TypeError);
  iterator = box.plain();
  await step(iterator, undefined, 'private-base', false);
  await step(iterator, {}, 'private-rhs', false);
  await rejects(iterator.next(9), TypeError);
  var whole = { marker: 41 }; whole.self = whole;
  iterator = box.compound();
  await step(iterator, undefined, 'private-base', false);
  await step(iterator, box, 'private-rhs', false);
  await rejects(iterator.throw(whole), whole);
  check(box.sets === 1, 'injected-throw-bypasses-private-put');
  iterator = box.compound();
  await step(iterator, undefined, 'private-base', false);
  await step(iterator, box, 'private-rhs', false);
  var returned = await iterator.return(whole);
  check(returned.done && returned.value === whole && whole.self === whole, 'whole-return-through-rhs');
  check(box.sets === 1, 'injected-return-bypasses-private-put');
  box.replace(1n);
  iterator = box.compound();
  await step(iterator, undefined, 'private-base', false);
  await step(iterator, box, 'private-rhs', false);
  await rejects(iterator.next(2), TypeError);
  check(box.read() === 1n && box.sets === 1, 'mixed-numeric-type-error-before-private-put');
}
run().then(function () { print('mixed-assignment-references:ok'); }, function (error) { print(error); throw error; });
