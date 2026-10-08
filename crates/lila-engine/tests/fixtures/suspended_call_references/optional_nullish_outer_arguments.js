function base() { print('base'); return null; }
function key() { print('wrong-key'); return 'method'; }
function operand(name) { print('argument:' + name); return Promise.resolve(7); }
var object = { get method() { print('get'); return 0; } };
async function run() {
  try { (base()?.[key()])(await operand('nullish')); }
  catch (error) { print('nullish:' + (error instanceof TypeError)); }
  try { (object?.method)(await operand('noncallable')); }
  catch (error) { print('noncallable:' + (error instanceof TypeError)); }
  print('done');
}
run().catch(error => print('error:' + error)); print('called');
