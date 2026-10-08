var trace = [];
var numberKey = new Number(0);
numberKey[Symbol.toPrimitive] = function (hint) {
  trace.push('number:' + hint + ':' + (this === numberKey));
  return 2;
};
var stringKey = new String('unused');
stringKey[Symbol.toPrimitive] = function (hint) {
  trace.push('string:' + hint + ':' + (this === stringKey));
  return 'x';
};
var duplicateKey = new Number(0);
duplicateKey[Symbol.toPrimitive] = function (hint) {
  trace.push('duplicate:' + hint + ':' + (this === duplicateKey));
  return 'x';
};
var list = new Array(6);
var listPrototype = Object.create(Array.prototype);
Object.defineProperty(listPrototype, '0', {
  get: function () {
    trace.push('index:0:' + (this === list));
    list[6] = 'ignored';
    return numberKey;
  }
});
Object.setPrototypeOf(list, listPrototype);
Object.defineProperty(list, '1', {
  get: function () { trace.push('index:1'); return stringKey; }
});
list[2] = '2';
list[3] = duplicateKey;
list[4] = Symbol('ignored');
list[5] = 'tail';
var value = {};
Object.defineProperty(value, 'toJSON', {
  get: function () { trace.push('root:toJSON'); return undefined; }
});
Object.defineProperty(value, '2', {
  get: function () { trace.push('value:2'); return 2; }
});
Object.defineProperty(value, 'x', {
  get: function () { trace.push('value:x'); return 3; }
});
Object.defineProperty(value, 'tail', {
  get: function () { trace.push('value:tail'); return 4; }
});
Object.defineProperty(value, 'ignored', {
  get: function () { throw new Error('replacer length was reread'); }
});
print(JSON.stringify(value, list));
print(trace.join(','));

trace = [];
var proxyTarget = ['b', 'a', 'b'];
var lengthValue = new Number(0);
lengthValue[Symbol.toPrimitive] = function (hint) {
  trace.push('length:' + hint);
  return 3;
};
var proxyList = new Proxy(proxyTarget, {
  get: function (target, key, receiver) {
    trace.push('list:' + key);
    if (key === 'length') return lengthValue;
    if (key === '0') target.push('ignored');
    return Reflect.get(target, key, receiver);
  }
});
var proxyValue = {};
Object.defineProperty(proxyValue, 'a', {
  get: function () { trace.push('value:a'); return 1; }
});
Object.defineProperty(proxyValue, 'b', {
  get: function () { trace.push('value:b'); return 2; }
});
Object.defineProperty(proxyValue, 'ignored', {
  get: function () { throw new Error('Proxy replacer length was reread'); }
});
print(JSON.stringify(proxyValue, proxyList));
print(trace.join(','));
