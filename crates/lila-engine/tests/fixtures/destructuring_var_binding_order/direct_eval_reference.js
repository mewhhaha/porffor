var trace = '';
var object = { target: 1 };
var scope = new Proxy(object, {
  has(object, key) { if (key === 'target') trace += 'h'; return Reflect.has(object, key); },
  set(object, key, value, receiver) { if (key === 'target') trace += 's'; return Reflect.set(object, key, value, receiver); }
});
var source = { get p() { trace += 'g'; return 9; } };
function run() {
  var target = 3;
  with (scope) { eval('var { p: target } = source;'); }
  return target;
}
run() === 3 && object.target === 9 && trace === 'hghs';
