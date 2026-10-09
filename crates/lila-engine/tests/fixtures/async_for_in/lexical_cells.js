function check(condition, label) { if (!condition) throw label; }

async function run() {
  var pairs = [];
  // Both uncaptured heads occur in a source block, and both remain live while
  // the inner body suspends and forces collection.
  {
    for (let outer in await Promise.resolve({ first: 1, second: 2 })) {
      for (const inner in await Promise.resolve({ a: 1, b: 2 })) {
        await Promise.resolve(0); gc(); pairs.push(outer + ':' + inner);
      }
    }
  }
  check(pairs.join(',') === 'first:a,first:b,second:a,second:b', 'block-nested-uncaptured-keys');

  var names = [], readers = [];
  {
    for (let key in await Promise.resolve({ first: 1, second: 2 })) {
      for (const key in await Promise.resolve({ a: 1, b: 2 })) {
        readers.push(function () { return key; });
        await Promise.resolve(0); gc(); names.push(key);
      }
      names.push(key);
    }
  }
  check(names.join(',') === 'a,b,first,a,b,second', 'nested-key-shadow-restores-outer-cell');
  gc();
  check(readers[0]() === 'a' && readers[1]() === 'b' && readers[2]() === 'a' && readers[3]() === 'b', 'captured-inner-key-keeps-original-per-iteration-cell');

  var value = { original: true }, reached = false, sawTdz = false;
  {
    try {
      for (let value in (await Promise.resolve(0), value)) { reached = true; }
    } catch (error) { sawTdz = error instanceof ReferenceError; }
  }
  check(sawTdz && !reached && value.original === true, 'block-head-tdz-survives-await-and-restores-outer-value');
}
run().then(function () { print('async-for-in-lexical-cells:ok'); }, function (error) { print(error); throw error; });
