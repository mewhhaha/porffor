var failures = 0;

var stringTarget = new Proxy(new String("str"), {});
var stringProxy = new Proxy(stringTarget, {
  get: null,
});

if (!Reflect.has(stringProxy, "length")) failures |= 1;
if (!(0 in stringProxy)) failures |= 2;
if (4 in stringProxy) failures |= 4;

var sym = Symbol();
var target = new Proxy({}, {
  has: function(_target, key) {
    return [sym, "6", "foo"].includes(key);
  },
});

var proxy = new Proxy(target, {
  get: null,
});

if (!Reflect.has(proxy, sym)) failures |= 8;
if (!("6" in proxy)) failures |= 16;
if (!("foo" in Object.create(proxy))) failures |= 32;
if ("bar" in proxy) failures |= 64;

failures === 0;
