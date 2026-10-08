var argument = { method: function () { print('unexpected-old-argument'); return 0; } };
class Parent {
  get method() {
    print('super-get');
    argument.method = function () { print('super-argument'); return 20; };
    return function (first, second) { print('super-call:' + first + ':' + second); };
  }
}
class Child extends Parent {
  get #method() {
    print('private-get');
    argument.method = function () { print('private-argument'); return 10; };
    return function (first, second) { print('private-call:' + first + ':' + second); };
  }
  *values() {
    this.#method(argument.method(), yield 'private?');
    super.method(argument.method(), yield 'super?');
  }
}
var iterator = new Child().values();
print(iterator.next().value);
print(iterator.next(2).value);
print(iterator.next(3).done);
