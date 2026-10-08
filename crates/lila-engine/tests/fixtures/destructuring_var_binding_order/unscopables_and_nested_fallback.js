var trace = '';
var innerObject = { target: 1, [Symbol.unscopables]: { target: true } };
var inner = new Proxy(innerObject, {
  has(object, key) { if (key === 'target') trace += 'i'; return Reflect.has(object, key); },
  get(object, key, receiver) { if (key === Symbol.unscopables) trace += 'u'; return Reflect.get(object, key, receiver); }
});
var outerObject = { target: 2 };
var outer = new Proxy(outerObject, {
  has(object, key) { if (key === 'target') trace += 'o'; return Reflect.has(object, key); },
  set(object, key, value, receiver) { if (key === 'target') trace += 's'; return Reflect.set(object, key, value, receiver); }
});
var source = { get p() { trace += 'g'; innerObject.target = 5; innerObject[Symbol.unscopables].target = false; return 9; } };
var target = 3;
with (outer) { with (inner) { var { p: target } = source; } }
trace === 'iuogos' && target === 3 && outerObject.target === 9 && inner.target === 5;
