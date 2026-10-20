var original = { value: 10, method: function () { print('unexpected-old-method'); } };
var selected = original;
function key() {
  print('key-source');
  original.method = function (value) { print('new:' + (this === original) + ':' + this.value + ':' + value); return value; };
  return 'method';
}
function* values() { return selected[key()](yield 'argument?'); }
var iterator = values();
print(iterator.next().value);
selected = {};
original.value = 20;
original.method = function () { print('unexpected-late-method'); };
var result = iterator.next(7);
print(result.value + ':' + result.done);

function* interleaved(label) { return (yield label)[key()](yield 'resumed'); }
var left = interleaved('left');
var right = interleaved('right');
print(left.next().value);
print(right.next().value);
print(right.next(original).value);
print(left.next(original).value);
result = right.next(2);
print(result.value + ':' + result.done);
result = left.next(1);
print(result.value + ':' + result.done);
