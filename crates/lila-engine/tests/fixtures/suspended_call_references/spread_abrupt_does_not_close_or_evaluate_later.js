var marker = {};
var source = {};
source[Symbol.iterator] = function () {
  print('iterator');
  return { next: function () { print('next'); return {
    get done() { print('done-get'); return false; },
    get value() { print('value-get'); throw marker; }
  }; }, return: function () { print('wrong-close'); return {}; } };
};
var object = { get method() { print('callee'); return function () { print('wrong-call'); }; } };
function operand() { print('wrong-operand'); return 1; }
async function run() {
  try { object.method(...source, await operand()); } catch (error) { print('caught:' + (error === marker)); }
  print('done');
}
run().catch(error => print('error:' + error)); print('called');
