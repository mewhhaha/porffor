function check(condition, label) { if (!condition) throw label; }
var value = 3, scope = {value: 4}, reads = 0, source = {};
Object.defineProperty(source, 'x', { get: function () {
  reads++; delete scope.value; return undefined;
} });
function* values() {
  with (scope) { ({x: value = (yield 'first', yield 'second')} = source); }
  return value;
}
var iterator = values(), result = iterator.next();
check(result.value === 'first' && !result.done && reads === 1, 'reference-before-get');
scope.extra = 9; gc(); result = iterator.next();
check(result.value === 'second' && !result.done, 'second-default-suspension');
gc(); result = iterator.next(7);
check(result.done && result.value === 3 && scope.value === 7 && reads === 1, 'original-object-record-put');

// A with miss keeps its original global selection even if the object gains it.
var missing = 2, missedScope = {}, input = {};
Object.defineProperty(input, 'x', { get: function () { missedScope.missing = 19; return undefined; } });
function* missed() { with(missedScope) { ({x: missing = yield 'missing-default'} = input); } return missing; }
iterator = missed(); result = iterator.next();
check(result.value === 'missing-default' && !result.done, 'with-miss-before-get');
gc(); result = iterator.next(23);
check(result.done && result.value === 23 && missedScope.missing === 19, 'original-global-reference');
print('generator-object-pattern-with:ok');
