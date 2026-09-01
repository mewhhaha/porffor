var other = __lilaCreateRealm().global;

function throwsOtherTypeError(operation) {
  try {
    operation();
  } catch (error) {
    return Object.getPrototypeOf(error) === other.TypeError.prototype;
  }
  return false;
}

var nonCallableProxy = new Proxy([0], { set: {} });
var revocable;
revocable = Proxy.revocable([0], {
  get: function (target, key, receiver) {
    var value = Reflect.get(target, key, receiver);
    if (key === "length") revocable.revoke();
    return value;
  },
});
var falseResultProxy = new Proxy([0], {
  set: function () {
    return false;
  },
});

var fixedTarget = {};
Object.defineProperty(fixedTarget, "fixed", {
  value: 1,
  writable: false,
  configurable: false,
});
var incompatibleProxy = new Proxy(fixedTarget, {
  set: function () {
    return true;
  },
});

var directReflectRevocable = Proxy.revocable({}, {});
directReflectRevocable.revoke();
var directReflectNonCallable = new Proxy({}, { set: {} });

var prototypeFixedTarget = {};
Object.defineProperty(prototypeFixedTarget, "0", {
  value: 1,
  writable: false,
  configurable: false,
});
var prototypeInvariantProxy = new Proxy(prototypeFixedTarget, {
  set: function () {
    return true;
  },
});
var fillPrototypeReceiver = new Array(1);
Object.setPrototypeOf(fillPrototypeReceiver, prototypeInvariantProxy);

var falsePrototypeProxy = new Proxy({}, {
  set: function () {
    return false;
  },
});
var falsePrototypeReceiver = new Array(1);
Object.setPrototypeOf(falsePrototypeReceiver, falsePrototypeProxy);
var pushFalsePrototypeReceiver = new Array(1);
Object.setPrototypeOf(pushFalsePrototypeReceiver, falsePrototypeProxy);

var reflectPrototypeRevocable = Proxy.revocable({}, {});
var reflectPrototypeReceiver = Object.create(reflectPrototypeRevocable.proxy);
reflectPrototypeRevocable.revoke();

throwsOtherTypeError(function () {
  other.Array.prototype.fill.call(revocable.proxy, 1);
})
  && throwsOtherTypeError(function () {
    other.Array.prototype.fill.call(nonCallableProxy, 1);
  })
  && throwsOtherTypeError(function () {
    other.Array.prototype.fill.call(falseResultProxy, 1);
  })
  && throwsOtherTypeError(function () {
    other.Reflect.set(incompatibleProxy, "fixed", 2);
  })
  && throwsOtherTypeError(function () {
    other.Reflect.set(directReflectRevocable.proxy, "value", 1);
  })
  && throwsOtherTypeError(function () {
    other.Reflect.set(directReflectNonCallable, "value", 1);
  })
  && throwsOtherTypeError(function () {
    other.Array.prototype.fill.call(fillPrototypeReceiver, 2);
  })
  && throwsOtherTypeError(function () {
    other.Array.prototype.fill.call(falsePrototypeReceiver, 2);
  })
  && throwsOtherTypeError(function () {
    other.Array.prototype.push.call(pushFalsePrototypeReceiver, 2);
  })
  && throwsOtherTypeError(function () {
    other.Reflect.set(reflectPrototypeReceiver, "value", 1);
  });
