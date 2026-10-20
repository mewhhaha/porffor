var object = { get bad() { print('get'); return 1; } };
function operand(name) { print('argument:' + name); return Promise.resolve(2); }
async function run() {
  try { object.bad(await operand('call')); } catch (error) { print('call:' + (error instanceof TypeError)); }
  try { new object.bad(await operand('new')); } catch (error) { print('new:' + (error instanceof TypeError)); }
  print('done');
}
run().catch(error => print('error:' + error)); print('called');
