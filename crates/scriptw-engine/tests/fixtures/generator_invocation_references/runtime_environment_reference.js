var target = { method: function (value) { 'use strict'; print('call:' + (this === scope) + ':' + value); } };
var blocked = { get method() { print('blocked'); return false; } };
target[Symbol.unscopables] = blocked;
var scope = new Proxy(target, {
  has: function (object, key) { if (key === 'method') print('has'); return Reflect.has(object, key); },
  get: function (object, key, receiver) {
    if (key === 'method') print('get');
    if (key === Symbol.unscopables) print('unscopables');
    return Reflect.get(object, key, receiver);
  }
});
function* values() { eval('var spare = 1;'); with (scope) { (method)(yield 'environment?'); } }
var iterator = values();
print(iterator.next().value);
target.method = function () { print('unexpected-method'); };
Object.defineProperty(blocked, 'method', { value: true });
print(iterator.next(7).done);
