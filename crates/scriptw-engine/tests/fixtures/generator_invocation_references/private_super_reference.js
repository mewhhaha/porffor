class Parent {
  get method() { print('unexpected-old-super'); return function () {}; }
}
class Child extends Parent {
  get #method() {
    print('private-get');
    return function (value) { print('private:' + (this === child) + ':' + value); return value; };
  }
  *values() {
    this.#method(yield 'private?');
    super[yield 'super-key?'](yield 'super-argument?');
    return (yield 'private-base?').#method(yield 'private-argument?');
  }
  *brand() { return (yield 'brand?').#method(yield 'unexpected-argument'); }
}
var child = new Child();
var iterator = child.values();
print(iterator.next().value);
print(iterator.next(1).value);
Object.defineProperty(Parent.prototype, 'method', { configurable: true, get: function () {
  print('super-get');
  return function (value) { print('super:' + (this === child) + ':' + value); };
} });
var key = { toString: function () { print('key'); return 'method'; } };
print(iterator.next(key).value);
Object.defineProperty(Parent.prototype, 'method', { value: function () { print('unexpected-replacement'); } });
print(iterator.next(2).value);
print(iterator.next(child).value);
var result = iterator.next(3);
print(result.value + ':' + result.done);
iterator = child.brand();
print(iterator.next().value);
try { iterator.next({}); } catch (error) { print(error instanceof TypeError); }
