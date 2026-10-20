var marker = {};
var object = { get method() { print('get'); throw marker; } };
var key = { toString: function () { print('key-convert'); throw marker; } };
function operand() { print('wrong-operand'); return 7; }
async function run() {
  try { (object?.method)(await operand()); } catch (error) { print('getter:' + (error === marker)); }
  try { (object?.[key])(await operand()); } catch (error) { print('key:' + (error === marker)); }
  print('done');
}
run().catch(error => print('error:' + error)); print('called');
