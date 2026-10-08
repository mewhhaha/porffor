function equal(actual, expected) { if (actual !== expected) throw new Error(String(actual) + ' !== ' + String(expected)); }
let targets = 0;
function* ordinary() {
  for (var key = yield 'prefix' in (targets++, yield 'head')) { yield key; }
  return key;
}
const iterator = ordinary();
equal(iterator.next().value, 'prefix'); equal(targets, 0);
equal(iterator.next(17).value, 'head'); equal(targets, 1);
equal(iterator.next({ab: 1}).value, 'ab');
const result = iterator.next(); equal(result.done, true); equal(result.value, 'ab'); equal(targets, 1);
let selected = 0;
async function plain() {
  for (var key = await Promise.resolve(17) in (selected++, await Promise.resolve({ab: 1}))) { await key; }
  return key;
}
plain().then(value => { equal(value, 'ab'); equal(selected, 1); print('ok'); });
