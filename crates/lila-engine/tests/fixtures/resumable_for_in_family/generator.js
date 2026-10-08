function equal(actual, expected) { if (actual !== expected) throw new Error(String(actual) + ' !== ' + String(expected)); }
function tdz(read) { let failed = false; try { read(); } catch (error) { failed = error instanceof ReferenceError; } equal(failed, true); }
let headRead;
function* captured() {
  for (let [first, second, missing = yield function readMissing() { return missing; }] in
       (yield function readHead() { return first; }, {ab: 1})) {
    yield function readIteration() { return [first, missing]; };
  }
}
const iterator = captured();
headRead = iterator.next().value;
tdz(headRead);
const missingRead = iterator.next().value;
tdz(missingRead);
gc();
const retained = iterator.next(19).value;
equal(retained()[0], 'a'); equal(retained()[1], 19);
equal(missingRead(), 19); tdz(headRead);
equal(iterator.next().done, true);
gc(); equal(retained()[1], 19); tdz(headRead);

let selected = 0, converted = 0, written = 0;
const original = {set slot(value) { equal(value, 'ab'); written++; }};
let current = original;
const rawKey = {[Symbol.toPrimitive]() { converted++; return 'slot'; }};
function choose() { selected++; return current; }
function* target() { for (choose()[yield 'key'] in {ab: 1}) { yield written; } }
const member = target();
equal(member.next().value, 'key'); equal(selected, 1); equal(converted, 0);
current = {set slot(value) { throw new Error('target was selected twice'); }};
gc(); equal(member.next(rawKey).value, 1);
equal(selected, 1); equal(converted, 1); equal(written, 1);
equal(member.next().done, true);

const originalIterator = String.prototype[Symbol.iterator];
let closed = 0, finalizers = 0;
String.prototype[Symbol.iterator] = function () {
  return {next() { return {done: false, value: undefined}; }, return() { closed++; return {}; }};
};
function* defaultValue() {
  try { for (let [value = yield 'default'] in {ab: 1}) { throw new Error('body was entered'); } }
  finally { finalizers++; }
}
try {
  const failure = {};
  const throwing = defaultValue(); equal(throwing.next().value, 'default');
  let observed;
  try { throwing.throw(failure); } catch (error) { observed = error; }
  equal(observed, failure); equal(closed, 1); equal(finalizers, 1);
  const returning = defaultValue(); equal(returning.next().value, 'default');
  const result = returning.return(31); equal(result.done, true); equal(result.value, 31);
  equal(closed, 2); equal(finalizers, 2);
} finally { String.prototype[Symbol.iterator] = originalIterator; }
print('ok');
