function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 71 }; whole.self = whole;
whole[Symbol.toPrimitive] = function () { throw 'unexpected-conversion'; };

async function* collecting(input) {
  var names = [], readers = [], heads = 0;
  for (const key in await Promise.resolve(yield 'head').then(function (value) {
    heads++; gc(); return value;
  })) {
    readers.push(function () { return key; });
    yield key; await Promise.resolve(0); gc(); names.push(key);
  }
  for (var index = 0; index < readers.length; index++) check(readers[index]() === names[index], 'original-distinct-per-key-capture');
  check(heads === 1, 'whole-head-settles-once');
  return names;
}

async function collect(input) {
  var iterator = collecting(input), step = await iterator.next();
  check(step.value === 'head' && step.done === false, 'head-yield-before-enumerator');
  step = await iterator.next(input);
  while (!step.done) { gc(); step = await iterator.next(); }
  return step.value;
}

async function ordered() {
  var parent = { inherited: 1, blocked: 2, tail: 3 };
  var child = Object.create(parent), events = [], reads = 0;
  Object.defineProperty(child, 'first', { get: function () { reads++; throw whole; }, enumerable: true, configurable: true });
  child.deleted = 2;
  Object.defineProperty(child, 'blocked', { value: 4, enumerable: false, configurable: true });
  child[Symbol('excluded')] = 7;
  var proxy = new Proxy(child, {
    ownKeys: function (object) { events.push('keys'); return Reflect.ownKeys(object); },
    getOwnPropertyDescriptor: function (object, key) { events.push('descriptor:' + key); return Reflect.getOwnPropertyDescriptor(object, key); },
    getPrototypeOf: function (object) { events.push('prototype'); return Reflect.getPrototypeOf(object); }
  });
  async function* names() { for (const key in await (yield 'ordered-head')) { yield key; await Promise.resolve(0); } }
  var iterator = names(), first = await iterator.next();
  check(first.value === 'ordered-head' && events.length === 0, 'no-enumeration-before-completed-head');
  first = await iterator.next(proxy); check(first.value === 'first', 'first-own-key');
  delete child.deleted; delete parent.tail; gc();
  var rest = [], step = await iterator.next();
  while (!step.done) { rest.push(step.value); step = await iterator.next(); }
  check(rest.join(',') === 'inherited', 'descriptor-deletion-and-nonenumerable-visited-shadow');
  check(reads === 0 && events.filter(function (event) { return event === 'keys'; }).length === 1, 'keys-once-and-no-property-value-get');
  check(events.indexOf('descriptor:deleted') >= 0 && events.indexOf('prototype') > events.indexOf('descriptor:blocked'), 'live-descriptors-before-prototype-advance');
}

async function nested() {
  var output = [];
  async function* values() {
    outer: for (const key in await (yield 'outer-head')) {
      for (const nested in await Promise.resolve({ one: 1 })) {
        switch (yield key + ':' + nested) {
          case 'continue':
            try { continue outer; } finally { await Promise.resolve(0); yield 'finally:' + key; }
          default: output.push(key + ':' + nested);
        }
      }
    }
  }
  var iterator = values(); check((await iterator.next()).value === 'outer-head', 'nested-head');
  check((await iterator.next({ first: 1, second: 2 })).value === 'first:one', 'first-nested-cursor');
  check((await iterator.next('continue')).value === 'finally:first', 'continue-runs-mixed-finalizer');
  check((await iterator.next()).value === 'second:one', 'separate-original-iteration-cells');
  check((await iterator.next()).done === true && output.join(',') === 'second:one', 'nested-cursors-retire-independently');
}

async function run() {
  check((await collect({ first: 1, second: 2 })).join(',') === 'first,second', 'object-domain');
  check((await collect(['a', 'b'])).join(',') === '0,1', 'array-domain');
  check((await collect('ab')).join(',') === '0,1', 'primitive-string-domain');
  check((await collect(null)).length === 0, 'null-domain');
  check((await collect(undefined)).length === 0, 'undefined-domain');
  check((await collect(17)).length === 0, 'number-domain');
  check((await collect(true)).length === 0, 'boolean-domain');
  check((await collect(Symbol('empty'))).length === 0, 'symbol-domain');
  check((await collect(1n)).length === 0, 'bigint-domain');
  await ordered(); await nested();
  var reader, reached = false, key = { outer: 1 };
  async function* tdz() { for (let key in (reader = function () { return key; }, await (yield 'tdz-head'))) { reached = true; yield key; } }
  var iterator = tdz(); check((await iterator.next()).value === 'tdz-head', 'original-head-suspends-under-tdz');
  try { reader(); throw 'missing-head-tdz'; } catch (error) { check(error instanceof ReferenceError, 'head-reader-keeps-original-tdz-record'); }
  check((await iterator.next({ only: 1 })).value === 'only' && reached === true, 'distinct-per-key-record-initializes');
  try { reader(); throw 'head-reader-was-initialized'; } catch (error) { check(error instanceof ReferenceError, 'head-tdz-record-remains-uninitialized'); }
  check((await iterator.next()).done === true && key.outer === 1, 'head-and-iteration-cleanup-restores-outer-record');
  gc();
  try { reader(); throw 'completed-head-reader-was-initialized'; } catch (error) { check(error instanceof ReferenceError, 'completed-captured-head-tdz-survives-gc'); }
}
run().then(function () { print('mixed-async-generator-for-in-enumeration:ok'); }, function (error) { print(error); throw error; });
