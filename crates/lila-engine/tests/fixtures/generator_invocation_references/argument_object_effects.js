var original = { method: function () { print('unexpected-old-method'); return 0; } };
var selected = original;
function consume(object, value) { return object.method(value); }
function* values() { return consume(selected, yield 'argument?'); }
var iterator = values();
print(iterator.next().value);
selected = {};
original.method = function (value) { print('resumed:' + (this === original) + ':' + value); return value; };
var result = iterator.next(7);
print(result.value + ':' + result.done);
function mutate() {
  print('mutate');
  original.method = function (value) { print('mutated:' + (this === original) + ':' + value); return value; };
  return 8;
}
function consumeThree(object, first, second) { return object.method(first + second); }
function* eager() { return consumeThree(original, mutate(), yield 'eager?'); }
iterator = eager();
print(iterator.next().value);
result = iterator.next(2);
print(result.value + ':' + result.done);
