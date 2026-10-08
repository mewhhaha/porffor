function check(condition, label) { if (!condition) throw label; }
var value = 3, scope = {value: 5}, closes = 0;
var input = { [Symbol.iterator]: function () { return {
  next: function () { delete scope.value; return {value: undefined, done: false}; },
  return: function () { closes++; return {}; }
}; } };
function* selectedObject() { with (scope) { [value = yield 'object-default'] = input; } }
var iterator = selectedObject(), result = iterator.next();
check(result.value === 'object-default' && !result.done, 'with-default');
gc(); result = iterator.next(17);
check(result.done && scope.value === 17 && value === 3 && closes === 1, 'original-object-record-after-delete');
scope = {}; closes = 0;
input = { [Symbol.iterator]: function () { return {
  next: function () { scope.value = 9; return {value: undefined, done: false}; },
  return: function () { closes++; return {}; }
}; } };
function* selectedGlobal() { with (scope) { [value = yield 'global-default'] = input; } }
iterator = selectedGlobal(); result = iterator.next();
check(result.value === 'global-default' && !result.done, 'global-reference-default');
gc(); result = iterator.next(23);
check(result.done && value === 23 && scope.value === 9 && closes === 1, 'original-global-after-with-gains-binding');
scope = {}; closes = 0; var nextReads = 0;
input = { [Symbol.iterator]: function () {
  var record = { return: function () { closes++; return {}; } };
  Object.defineProperty(record, 'next', { get: function () {
    nextReads++; scope.value = 27;
    return function () { delete scope.value; return {value: undefined, done: false}; };
  } });
  return record;
} };
iterator = selectedObject(); result = iterator.next();
check(result.value === 'object-default' && !result.done && nextReads === 1, 'acquisition-before-reference-selection');
gc(); result = iterator.next(37);
check(result.done && scope.value === 37 && value === 23 && closes === 1 && nextReads === 1, 'cached-next-get-before-original-object-reference');
print('generator-array-pattern-with:ok');
