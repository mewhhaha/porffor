function operand() { print('operand'); return Promise.resolve(0).then(function (value) {
  print('replace'); globalThis.eval = function () { print('wrong-eval'); return 0; }; return value;
}); }
async function run() { var local = 41; var result = eval('local + 1', await operand()); print('eval:' + result); }
run().catch(error => print('error:' + error)); print('called');
