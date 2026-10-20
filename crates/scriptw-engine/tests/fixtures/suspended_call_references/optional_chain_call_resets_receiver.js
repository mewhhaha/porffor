var object = { method: function () {
  'use strict'; print('inner:' + (this === object));
  return function (value) { 'use strict'; print('outer:' + (this === undefined) + ':' + value); };
} };
function operand() { print('operand'); return Promise.resolve(7); }
async function run() { (object?.method?.())(await operand()); print('done'); }
run().catch(error => print('error:' + error)); print('called');
