function check(condition, label) { if (!condition) throw label; }
var whole = { marker: 47 }; whole.self = whole;
whole[Symbol.toPrimitive] = function () { throw 'unexpected-conversion'; };

async function collect(input) {
  var names = [], readers = [], heads = 0;
  for (const key in await Promise.resolve(input).then(function (value) {
    heads++; gc(); return value;
  })) {
    readers.push(function () { return key; });
    await Promise.resolve(0); gc(); names.push(key);
  }
  check(heads === 1, 'head-completes-once');
  for (var i = 0; i < readers.length; i++) check(readers[i]() === names[i], 'original-per-key-captured-cell');
  return names;
}

async function ordered() {
  var parent = { inherited: 1, blocked: 2, tail: 3 };
  var child = Object.create(parent);
  child.first = 1; child.deleted = 2;
  Object.defineProperty(child, 'blocked', { value: 4, enumerable: false, configurable: true });
  var names = [];
  for (const key in await Promise.resolve(child)) {
    names.push(key); gc();
    if (key === 'first') { delete child.deleted; delete parent.tail; }
    await Promise.resolve(0);
  }
  check(names.join(',') === 'first,inherited', 'live-descriptor-deletion-and-nonenumerable-shadow');
}

async function branches(input) {
  var result = [];
  outer: for (const key in await Promise.resolve(input)) {
    if (await Promise.resolve(key === 'first')) {
      const [received = await Promise.resolve(whole)] = [];
      check(received === whole, 'array-close-owner-inside-iteration');
    } else { await Promise.resolve(0); }
    switch (await Promise.resolve(key)) {
      case await Promise.resolve('first'): result.push(key); break;
      default: result.push(key);
    }
    try {
      if (key === 'first') continue outer;
      break outer;
    } finally { await Promise.resolve(0); gc(); result.push('finally:' + key); }
  }
  check(result.join(',') === 'first,finally:first,second,finally:second', 'labelled-continue-break-through-awaiting-finally');
}

async function nested(input) {
  var values = [];
  for (const outer in await Promise.resolve(input)) {
    for (const inner in await Promise.resolve({ one: 1 })) {
      await Promise.resolve(0); gc(); values.push(outer + ':' + inner);
    }
  }
  check(values.join(',') === 'first:one,second:one', 'independent-nested-native-cursors');
}

async function run() {
  check((await collect({ first: 1, second: 2 })).join(',') === 'first,second', 'object-order');
  check((await collect(['a', 'b'])).join(',') === '0,1', 'array-domain');
  check((await collect('ab')).join(',') === '0,1', 'primitive-string-domain');
  check((await collect(null)).length === 0, 'null-domain');
  check((await collect(undefined)).length === 0, 'undefined-domain');
  check((await collect(17)).length === 0, 'number-domain');
  check((await collect(true)).length === 0, 'boolean-domain');
  check((await collect(Symbol('empty'))).length === 0, 'symbol-domain');
  check((await collect(1n)).length === 0, 'bigint-domain');
  await ordered(); await branches({ first: 1, second: 2, third: 3 });
  await nested({ first: 1, second: 2 });
  var sizes = [];
  for (const { length: size } in await Promise.resolve({ aa: 1, bbb: 2 })) {
    await Promise.resolve(0); sizes.push(size);
  }
  check(sizes.join(',') === '2,3', 'original-eager-per-key-binding-pattern');
  var key = { mustNotRead: 1 }, reached = false;
  try { for (let key in await key) { reached = true; } }
  catch (error) { check(error instanceof ReferenceError, 'original-head-tdz-before-await'); }
  check(reached === false && key.mustNotRead === 1, 'tdz-failure-restores-outer-record');
  var eager = [];
  if (true) { for (var name in { one: 1 }) { eager.push(name); } }
  check(eager.join(',') === 'one', 'zero-await-for-in-owns-enclosing-if-phases');
}
run().then(function () { print('async-for-in-enumeration:ok'); }, function (error) { print(error); throw error; });
