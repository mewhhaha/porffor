function track(name, target) {
  return new Proxy(target, {
    get: function (object, key, receiver) {
      if (key === 'length' || key === '0' || key === '1' || key === '2') print(name + '-get:' + key);
      return Reflect.get(object, key, receiver);
    },
    has: function (object, key) { print(name + '-has:' + key); return Reflect.has(object, key); }
  });
}
var grandTarget = [4, 5];
var grand = track('grand', grandTarget);
var childTarget = [grand, 2];
var child = track('child', childTarget);
var rootTarget = [child, 3];
var root = track('root', rootTarget);
Object.defineProperty(grandTarget, '0', { get: function () {
  print('grand-zero');
  childTarget[1] = 20;
  childTarget[2] = 99;
  rootTarget[1] = 30;
  grandTarget[2] = 100;
  return 4;
} });
var result = Array.prototype.flat.call(root, 2);
print(result.length + ':' + result[0] + ':' + result[1] + ':' + result[2] + ':' + result[3]);
