function check(condition, label) { if (!condition) throw label; }
async function step(iterator, input, value, done) {
  var result = await iterator.next(input);
  check(result.value === value && result.done === done, 'mixed-step');
}

var reads = [], entered = 0, tests = 0, updates = 0;
async function* phases() {
  for (let index = await Promise.resolve(0);
       await Promise.resolve((tests++, index < 2));
       index = (yield 'update:' + index) + await Promise.resolve((updates++, 0))) {
    entered++;
    reads.push(function () { return index; });
    const before = await Promise.resolve(index);
    yield 'body:' + before;
    if (await Promise.resolve(index === 0)) {
      yield 'then';
      await Promise.resolve(0);
    } else {
      await Promise.resolve(0);
      yield 'else';
    }
    gc();
  }
  return Promise.resolve(42);
}

async function* whileAndDo() {
  while (await (yield 'while-test')) {
    await Promise.resolve(0);
    yield 'while-body';
  }
  do {
    yield 'do-body';
    await Promise.resolve(0);
  } while (await (yield 'do-test'));
  return 9;
}

async function* lexicalHead() {
  for (let index = await (yield function () { return index; }); false;) {}
}

var coercions = [], receiver = { key: 1 }, received;
var left = { [Symbol.toPrimitive]: function () { coercions.push('left'); return 3; } };
var right = { [Symbol.toPrimitive]: function () { coercions.push('right'); return 4; } };
var whole = { marker: 29 }; whole.self = whole;
async function* operators() {
  for (let once = 0; once < 1; once++) {
    yield ((yield 'left') + (await (yield 'right')));
    yield receiver[yield 'key'];
    receiver.key = await (yield 'rhs');
    yield received;
    var plain = await Promise.resolve(2);
    plain += yield 'compound';
    plain &&= await Promise.resolve(plain + 1);
    plain ||= await { get then() { throw 'skipped-logical-rhs'; } };
    yield plain;
    yield `a${await Promise.resolve(1)}b${yield 'template'}`;
  }
}

async function run() {
  var iterator = phases();
  await step(iterator, undefined, 'body:0', false);
  await step(iterator, undefined, 'then', false);
  await step(iterator, undefined, 'update:0', false);
  await step(iterator, 1, 'body:1', false);
  await step(iterator, undefined, 'else', false);
  await step(iterator, undefined, 'update:1', false);
  await step(iterator, 2, 42, true);
  check(entered === 2 && tests === 3 && updates === 2, 'phase-evaluation-once');
  check(reads.length === 2 && reads[0]() === 0 && reads[1]() === 1, 'original-for-per-iteration-cells');
  iterator = whileAndDo();
  await step(iterator, undefined, 'while-test', false);
  await step(iterator, true, 'while-body', false);
  await step(iterator, undefined, 'while-test', false);
  await step(iterator, false, 'do-body', false);
  await step(iterator, undefined, 'do-test', false);
  await step(iterator, false, 9, true);
  iterator = lexicalHead();
  var head = await iterator.next(), headRead = head.value;
  check(!head.done, 'suspended-head-before-binding-initialization');
  try { headRead(); throw 'missing-head-tdz'; }
  catch (error) { check(error instanceof ReferenceError, 'original-head-tdz-before-yield-resume'); }
  gc();
  await step(iterator, 0, undefined, true);
  check(headRead() === 0, 'same-original-head-cell-after-await');
  iterator = operators();
  await step(iterator, undefined, 'left', false);
  await step(iterator, left, 'right', false);
  check(coercions.length === 0, 'both-values-before-coercion');
  await step(iterator, right, 7, false);
  check(coercions.join(',') === 'left,right', 'original-operator-coercion-order');
  await step(iterator, undefined, 'key', false);
  Object.defineProperty(receiver, 'key', { configurable: true, get: function () { return 8; } });
  gc();
  await step(iterator, 'key', 8, false);
  await step(iterator, undefined, 'rhs', false);
  Object.defineProperty(receiver, 'key', { configurable: true, set: function (value) { received = value; } });
  await step(iterator, whole, whole, false);
  check(received === whole && received.self === whole, 'retained-raw-property-reference-after-await');
  await step(iterator, undefined, 'compound', false);
  await step(iterator, 3, 6, false);
  await step(iterator, undefined, 'template', false);
  await step(iterator, 2, 'a1b2', false);
  await step(iterator, undefined, undefined, true);
}
run().then(function () { print('mixed-async-generator-phases:ok'); }, function (error) { print(error); throw error; });
