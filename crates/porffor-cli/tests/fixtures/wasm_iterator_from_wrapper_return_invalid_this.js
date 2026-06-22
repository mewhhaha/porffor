function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual;
  }
}

function checkThrowsTypeError(callback, label) {
  var threw = false;
  try {
    callback();
  } catch (error) {
    threw = error instanceof TypeError;
  }
  check(threw, true, label);
}

var wrapperPrototype = Object.getPrototypeOf(Iterator.from({}));

checkThrowsTypeError(function () {
  wrapperPrototype.return.call({});
}, "plain object invalid this");

var calls = [];
var original = {
  return: function () {
    return { value: 5, done: true };
  },
};
var method = original.return;
original.return = function () {
  calls.push("call return");
  return method.apply(original, arguments);
};
var observed = new Proxy(original, {
  get: function (target, key, receiver) {
    calls.push("get");
    return Reflect.get(target, key, receiver);
  },
});

checkThrowsTypeError(function () {
  wrapperPrototype.return.call(observed);
}, "proxy invalid this");
check(calls.length, 0, "proxy calls");

true;
