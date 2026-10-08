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
function operand() { print('operand'); return Promise.resolve(7).then(function (value) {
  print('replace'); target.method = function () { print('wrong-method'); };
  Object.defineProperty(blocked, 'method', { value: true }); return value;
}); }
async function run() {

  with (scope) { (method)(await operand()); }
  print('done');
}
run().catch(error => print('error:' + error)); print('called');
