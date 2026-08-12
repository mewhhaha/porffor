var sym = Symbol();
var target = new Proxy({}, {
  get: function(_target, key) {
    switch (key) {
      case sym: return 1;
      case "10": return 2;
      case "foo": return 3;
    }
  }
});

var proxy = new Proxy(target, {
  get: null
});

var stringTarget = new Proxy(new String("str"), {});
var stringProxy = new Proxy(stringTarget, {
  get: null
});

stringProxy.length === 3 &&
  stringProxy[0] === "s" &&
  stringProxy[4] === undefined &&
  target[sym] === 1 &&
  target[10] === 2 &&
  target["10"] === 2 &&
  target.foo === 3 &&
  proxy[sym] === 1 &&
  proxy[10] === 2 &&
  Object.create(proxy).foo === 3 &&
  proxy.bar === undefined;
