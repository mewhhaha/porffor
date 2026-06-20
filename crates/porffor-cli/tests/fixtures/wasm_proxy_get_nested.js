var plainObject = {
  get 0() {
    return 1;
  },
  foo: 2,
  set bar(_value) {}
};

var plainObjectTarget = new Proxy(plainObject, {});
var plainObjectProxy = new Proxy(plainObjectTarget, {
  get: undefined
});

var array = [1, 2, 3];
var arrayTarget = new Proxy(array, {});
var arrayProxy = new Proxy(arrayTarget, {
  get: undefined
});

plainObject[0] === 1 &&
  Object.create(plainObject)[0] === 1 &&
  Object.create(plainObjectTarget)[0] === 1 &&
  Object.create(plainObjectTarget).foo === 2 &&
  Object.create(plainObjectProxy)[0] === 1 &&
  Object.create(plainObjectProxy).foo === 2 &&
  plainObjectProxy.foo === 2 &&
  plainObjectProxy.bar === undefined &&
  arrayProxy.length === 3 &&
  arrayProxy[0] === 1 &&
  arrayProxy[1] === 2 &&
  arrayProxy[2] === 3;
