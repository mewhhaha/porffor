(function () {
  function check(value, expected, name) { if (value !== expected) throw new Error(name); }
  var a = [1, 2, 3];
  check(a.length, 3, 'initial length');
  a.length = 1;
  check(a.length, 1, 'assignment shrink');
  check(a[1], undefined, 'shrink removes index');
  Object.defineProperty(a, 'length', {value: 5, writable: false});
  check(a.length, 5, 'nonwritable definition');
  a.length = 9;
  check(a.length, 5, 'nonwritable assignment');
  check(delete a.length, false, 'nonconfigurable length');
  var caught = false;
  try { Object.defineProperty(a, 'length', {get: function () { return 99; }}); }
  catch (e) { caught = e instanceof TypeError; }
  check(caught, true, 'length cannot become accessor');
  var symbol = Symbol('length');
  a[symbol] = 17;
  a.named = 19;
  Object.defineProperty(a, '0', {get: function () { return 23; }, configurable: true});
  check(a[symbol], 17, 'symbol named slot');
  check(a.named, 19, 'named own slot');
  check(a[0], 23, 'index accessor');
  check(a.length, 5, 'named slots do not shadow length');
  var calls = 0;
  var proxy = new Proxy(a, {get: function (target, key, receiver) {
    calls++; return Reflect.get(target, key, receiver);
  }});
  check(proxy.length, 5, 'proxy get result');
  check(calls, 1, 'proxy trap preserved');
  check(Object.create({length: 29}).length, 29, 'ordinary inherited length');
  check(Object.create(a).length, 5, 'array prototype length');
  var descriptor = Object.getOwnPropertyDescriptor(a, 'length');
  check(descriptor.value, 5, 'length descriptor value');
  check(descriptor.writable, false, 'length descriptor writable');
  check(descriptor.configurable, false, 'length descriptor configurable');
  check(descriptor.enumerable, false, 'length descriptor enumerable');
  print('array-length-fast-path:ok');
})();
