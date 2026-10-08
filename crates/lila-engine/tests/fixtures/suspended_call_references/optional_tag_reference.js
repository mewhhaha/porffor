var original = { get tag() { print('get'); return function (strings, value) {
  'use strict'; print('tag:' + (this === original) + ':' + strings[0] + ':' + value);
}; } };
var target = original;
function operand() { print('operand'); return Promise.resolve(7).then(function (value) {
  print('replace'); target = null;
  Object.defineProperty(original, 'tag', { value: function () { print('wrong-tag'); } });
  return value;
}); }
function nullishOperand() { print('nullish-operand'); return Promise.resolve(8); }
async function run() {
  (target?.tag)`head${await operand()}tail`;
  try { (target?.tag)`head${await nullishOperand()}tail`; }
  catch (error) { print('nullish:' + (error instanceof TypeError)); }
  print('done');
}
run().catch(error => print('error:' + error)); print('called');
