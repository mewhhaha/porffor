var original = {
  value: 10,
  method: function (first, second, third, fourth) {
    print('call:' + this.value + ':' + first + ':' + second + ':' + third + ':' + fourth);
    return this.value + first + second + third + fourth;
  }
};
var selected = original;
var source = {
  [Symbol.iterator]: function () {
    print('iterator');
    var index = 0;
    return {
      get next() {
        print('next-get');
        return function () {
          var current = index++;
          print('next:' + current);
          return { done: current === 2, get value() { print('value:' + current); return current + 1; } };
        };
      },
      return: function () { print('unexpected-close'); return {}; }
    };
  }
};
var tail = {
  [Symbol.iterator]: function () {
    print('tail-iterator');
    var index = 0;
    return { next: function () { print('tail-next:' + index); return { done: index++ !== 0, value: 4 }; } };
  }
};
function* values() { return selected.method(...source, yield 'argument?', ...(yield 'spread?')); }
var iterator = values();
print(iterator.next().value);
source[Symbol.iterator] = function () { print('unexpected-reiteration'); throw new Error('reiteration'); };
original.value = 20;
original.method = function () { print('unexpected-replacement'); };
selected = {};
Array.prototype[Symbol.iterator] = function () { print('unexpected-array-iteration'); throw new Error('private argv exposed'); };
print(iterator.next(3).value);
var result = iterator.next(tail);
print(result.value + ':' + result.done);
