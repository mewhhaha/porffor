function Original(value) { this.value = value; print('construct:' + value + ':' + new.target.name); }
var object = { get Constructor() { print('get-constructor'); return Original; } };
function operand() {
  print('operand'); return Promise.resolve(7).then(function (value) {
    print('replace'); Object.defineProperty(object, 'Constructor', { value: function Wrong() { print('wrong-constructor'); } }); return value;
  });
}
async function run() { var result = new object.Constructor(await operand()); print('result:' + result.value + ':' + (result instanceof Original)); }
run().catch(error => print('error:' + error)); print('called');
