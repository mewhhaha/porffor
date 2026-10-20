var marker = {};
var object = { get method() { print('get'); throw marker; } };
var key = { toString: function () { print('key'); throw marker; } };
function operand() { print('wrong-operand'); return 1; }
async function run() {
  try { object.method(await operand()); } catch (error) { print('getter:' + (error === marker)); }
  try { object[key](await operand()); } catch (error) { print('key:' + (error === marker)); }
  var proxy = new Proxy({}, { get: function () { print('proxy-get'); throw marker; } });
  try { proxy.method(await operand()); } catch (error) { print('proxy:' + (error === marker)); }
  print('done');
}
run().catch(error => print('error:' + error)); print('called');
