var original = {
  get method() {
    print('get');
    return function (left, right) { 'use strict'; print('call:' + (this === original) + ':' + left + ':' + right); };
  }
};
var target = original;
var key = { toString: function () { print('key-convert'); return 'method'; } };
function base() { print('base'); return target; }
function property() { print('key-source'); return key; }
function first() { print('first'); return 1; }
function operand() {
  print('operand');
  return Promise.resolve(2).then(function (value) {
    print('replace');
    Object.defineProperty(original, 'method', { value: function () { print('wrong-method'); } });
    target = {}; key = 'wrong'; return value;
  });
}
async function run() { base()[property()](first(), await operand()); print('done'); }
run().catch(error => print('error:' + error)); print('called');
