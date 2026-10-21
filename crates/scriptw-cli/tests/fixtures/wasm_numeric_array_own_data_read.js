(function () {
  function check(actual, expected, label) {
    if (actual !== expected) throw new Error(label);
  }
  function read(target, key) { return target[key]; }
  var a = [11, 22];
  a['-0'] = 33;
  check(read(a, -0), 11, 'negative zero numeric');
  check(read(a, '-0'), 33, 'negative zero string');
  a[4294967294] = 44;
  a[4294967295] = 55;
  a[4294967296] = 66;
  check(read(a, 4294967294), 44, 'largest index');
  check(read(a, 4294967295), 55, 'first named integer');
  check(read(a, 4294967296), 66, 'large named integer');
  var named = [-1, 1.5, NaN, Infinity, -Infinity];
  for (var i = 0; i < named.length; i++) {
    a[named[i]] = i + 70;
    check(read(a, named[i]), i + 70, 'nonindex number');
  }
  var calls = 0;
  Object.defineProperty(a, '0', {configurable: true, get: function () {
    check(this, a, 'own getter receiver'); calls++; return 88;
  }});
  check(read(a, 0), 88, 'own accessor');
  check(calls, 1, 'own getter calls');
  delete a[0];
  var proto = Object.create(Array.prototype);
  Object.defineProperty(proto, '0', {get: function () {
    check(this, a, 'prototype getter receiver'); calls++; return 99;
  }});
  Object.setPrototypeOf(a, proto);
  check(read(a, 0), 99, 'hole prototype');
  Object.defineProperty(a, '0', {value: {x: 101}, configurable: true});
  var held = read(a, 0);
  for (var i = 0; i < 512; i++) { var garbage = {a: [i, i + 1]}; }
  check(held.x, 101, 'retained result');
  Object.defineProperty(a, '0', {value: 102});
  check(read(a, 0), 102, 'redefined data');
  check(calls, 2, 'getters not repeated');
  var proxyCalls = 0;
  var proxy = new Proxy([7], {get: function (target, key, receiver) {
    check(key, '0', 'proxy property key');
    check(receiver, proxy, 'proxy receiver'); proxyCalls++; return 103;
  }});
  check(read(proxy, 0), 103, 'proxy fallback');
  check(proxyCalls, 1, 'proxy calls');
  var revoked = Proxy.revocable([1], {});
  revoked.revoke();
  var caught = false;
  try { read(revoked.proxy, 0); } catch (e) { caught = e instanceof TypeError; }
  check(caught, true, 'revoked proxy');
  var keyCalls = 0;
  var key = {toString: function () { keyCalls++; return '1'; }};
  check(read(a, key), 22, 'object key');
  check(keyCalls, 1, 'object key coercion once');
  var sentinel = {};
  var badKey = {toString: function () { throw sentinel; }};
  caught = false;
  try { read(a, badKey); } catch (e) { caught = e === sentinel; }
  check(caught, true, 'key abrupt');
  caught = false;
  try { read(null, key); } catch (e) { caught = e instanceof TypeError; }
  check(caught, true, 'null target');
  check(keyCalls, 1, 'null checked before coercion');
  var methods = [function () { return this; }];
  var zero = 0;
  check(methods[zero](), methods, 'method receiver');
  print('numeric-array-own-data:ok');
})();
