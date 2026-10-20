var target = [1, 2], firstValue = 2;
Object.defineProperty(target, '0', { configurable: true, get: function () {
  print('zero');
  target[1] = 9;
  return [firstValue];
} });
function Result(length) {
  print('create:' + length);
  return new Proxy({}, { defineProperty: function (object, key, descriptor) {
    print('define:' + key + ':' + descriptor.value);
    return Reflect.defineProperty(object, key, descriptor);
  } });
}
Object.defineProperty(target, 'constructor', { get: function () {
  print('constructor');
  return { get [Symbol.species]() { print('species'); firstValue = 3; return Result; } };
} });
var source = new Proxy(target, {
  get: function (object, key, receiver) {
    print('root-get:' + key);
    if (key === 'length') return { [Symbol.toPrimitive]: function (hint) { print('length-' + hint); return 2; } };
    return Reflect.get(object, key, receiver);
  },
  has: function (object, key) { print('root-has:' + key); return Reflect.has(object, key); }
});
var depth = { [Symbol.toPrimitive]: function (hint) {
  print('depth-' + hint);
  target[2] = 7;
  return 1;
} };
var result = Array.prototype.flat.call(source, depth);
print(result[0] + ':' + result[1] + ':' + ('2' in result));
var live = [1, 2, , 4];
Object.defineProperty(live, '0', { get: function () {
  print('mutate'); delete live[1]; live[2] = 3; live[4] = 5; return 1;
} });
var observed = new Proxy(live, {
  get: function (object, key, receiver) { if (key === '0' || key === '1' || key === '2' || key === '3' || key === '4') print('live-get:' + key); return Reflect.get(object, key, receiver); },
  has: function (object, key) { print('live-has:' + key); return Reflect.has(object, key); }
});
result = Array.prototype.flat.call(observed);
print(result.length + ':' + result[0] + ':' + result[1] + ':' + result[2]);
