var trace = '';
var object = { target: 1, [Symbol.unscopables]: { target: false } };
var scope = new Proxy(object, {
  has(object, key) { if (key === 'target') trace += 'h'; return Reflect.has(object, key); },
  get(object, key, receiver) { if (key === Symbol.unscopables) trace += 'u'; return Reflect.get(object, key, receiver); },
  set(object, key, value, receiver) { if (key === 'target') trace += 's'; return Reflect.set(object, key, value, receiver); }
});
var source = { get p() { trace += 'g'; object[Symbol.unscopables].target = true; delete object.target; return 8; } };
var target = 3;
with (scope) { var { p: target } = source; }
trace === 'hughs' && target === 3 && object.target === 8;
