var firstTemplate;
var original = {
  value: 10,
  get tag() {
    print('get');
    return function (strings, first, second) {
      print('tag:' + (this === original) + ':' + this.value + ':' + first + ':' + second);
      print(strings[0] + ':' + strings[1] + ':' + strings[2]);
      print(Object.isFrozen(strings) + ':' + Object.isFrozen(strings.raw) + ':' + (strings === firstTemplate));
      firstTemplate = strings;
      return first;
    };
  }
};
var selected = original;
function after() { print('after'); return 9; }
function* values() { return selected[yield 'key?']`head${yield 'substitution?'}tail${after()}end`; }
var iterator = values();
print(iterator.next().value);
print(iterator.next('tag').value);
selected = { tag: function () { print('unexpected-tag'); } };
original.value = 20;
var result = iterator.next(3);
print(result.value + ':' + result.done);
selected = original;
iterator = values();
print(iterator.next().value);
print(iterator.next('tag').value);
result = iterator.next(4);
print(result.value + ':' + result.done);
