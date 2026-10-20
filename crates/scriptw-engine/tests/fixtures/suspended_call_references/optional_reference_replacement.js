var original = { get method() { print('get'); return function (value) {
  'use strict'; print('call:' + (this === original) + ':' + value);
}; } };
var target = original;
var raw = { toString: function () { print('key-convert'); return 'method'; } };
function base() { print('base'); return target; }
function key() { print('key'); return raw; }
function operand() { print('operand'); return Promise.resolve(7).then(function (value) {
  print('replace'); target = {}; raw = 'wrong';
  Object.defineProperty(original, 'method', { value: function () { print('wrong-method'); } });
  return value;
}); }
async function run() { (base()?.[key()])(await operand()); print('done'); }
run().catch(error => print('error:' + error)); print('called');
