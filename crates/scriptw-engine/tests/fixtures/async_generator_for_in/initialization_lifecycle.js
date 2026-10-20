function check(ok, label) { if (!ok) throw label; }
async function step(iterator, input, expected, done) {
  var result = await iterator.next(input);
  check(result.done === done && result.value === expected, 'for-in-initialization-step');
}
var keyLists = 0, descriptors = [], gets = [], puts = [];
var target = Object.create(null); target.a = 1; target.b = 2;
var source = new Proxy(target, {
  ownKeys: function (object) { keyLists++; return ['a', 'b']; },
  getOwnPropertyDescriptor: function (object, key) { descriptors.push(key); return Object.getOwnPropertyDescriptor(object, key); },
  get: function (object, key) { gets.push(key); throw 'for-in-read-value-or-iterator'; },
  getPrototypeOf: function () { return null; }
});
var destination = {
  set slot(value) { check(this === destination, 'for-in-original-assignment-receiver'); puts.push(value); }
};
async function* members() {
  for ((await (yield 'base'))[await (yield 'slot')] in source) {
    yield puts[puts.length - 1];
    await 0;
  }
}
async function* bindings() {
  for (let [first, second = await (yield function () { return second; })] in { a: 1, b: 2 }) {
    yield function () { return first + ':' + second; };
  }
}
function tdz(read) {
  try { read(); } catch (error) { check(error instanceof ReferenceError, 'per-key-tdz-error'); return; }
  throw 'missing-per-key-tdz';
}
async function run() {
  var iterator = members();
  await step(iterator, undefined, 'base', false);
  check(keyLists === 1 && descriptors.join(',') === 'a' && puts.length === 0, 'first-key-before-initializer');
  await step(iterator, destination, 'slot', false);
  gc();
  check(keyLists === 1 && descriptors.length === 1 && puts.length === 0, 'suspended-target-keeps-enumerator');
  await step(iterator, 'slot', 'a', false);
  check(puts.join(',') === 'a', 'first-key-assigned-once');
  await step(iterator, undefined, 'base', false);
  check(keyLists === 1 && descriptors.join(',') === 'a,b', 'advance-only-after-body');
  await step(iterator, destination, 'slot', false);
  await step(iterator, 'slot', 'b', false);
  await step(iterator, undefined, undefined, true);
  check(puts.join(',') === 'a,b' && gets.length === 0, 'no-repeated-assignment-or-iterator-access');

  iterator = members();
  await step(iterator, undefined, 'base', false);
  var marker = { value: 19 }; marker.self = marker;
  try { await iterator.throw(marker); throw 'missing-head-throw'; }
  catch (error) { check(error === marker && error.self === marker, 'whole-head-throw'); }
  check(puts.length === 2 && gets.length === 0, 'abrupt-head-does-not-put-or-close-for-in');
  iterator = members();
  await step(iterator, undefined, 'base', false);
  await step(iterator, destination, 'slot', false);
  var returned = await iterator.return(marker);
  check(returned.done && returned.value === marker && puts.length === 2 && gets.length === 0, 'whole-head-return-without-put');

  iterator = bindings();
  var pending = await iterator.next(), firstTdz = pending.value;
  check(!pending.done, 'first-pattern-default-suspended'); tdz(firstTdz); gc();
  var body = await iterator.next('A'), firstRead = body.value;
  check(!body.done && firstRead() === 'a:A' && firstTdz() === 'A', 'original-first-key-cells');
  pending = await iterator.next(); var secondTdz = pending.value;
  check(!pending.done && firstRead() === 'a:A', 'fresh-next-key-default'); tdz(secondTdz);
  gc(); body = await iterator.next('B'); var secondRead = body.value;
  check(!body.done && firstRead() === 'a:A' && secondRead() === 'b:B', 'independent-per-key-cells');
  await step(iterator, undefined, undefined, true);
  check(firstTdz() === 'A' && secondTdz() === 'B', 'retired-enumerator-preserves-captured-bindings');
  iterator = bindings(); pending = await iterator.next(); var cancelled = pending.value;
  returned = await iterator.return(marker);
  check(returned.done && returned.value === marker, 'return-through-live-binding-iterator');
  tdz(cancelled);
}
run().then(function () { print('mixed-for-in-initialization:ok'); }, function (error) { print(error); throw error; });
