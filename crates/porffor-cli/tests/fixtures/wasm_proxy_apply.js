function add(a, b) {
  return a + b;
}

function addThis(a, b) {
  return this.base + a + b;
}

var applyCalls = 0;
var proxy = new Proxy(add, {
  apply(target, thisArg, args) {
    applyCalls = applyCalls + 1;
    return target(args[0], args[1]) + 1;
  }
});

var nestedProxy = new Proxy(new Proxy(addThis, {}), {});
var boundAddThis = addThis.bind({ base: 10 }, 1);
var nestedBoundProxy = new Proxy(new Proxy(boundAddThis, {}), { apply: null });
var generatorAddThis = function* (arg) {
  yield this.base;
  yield arg;
};
var generatorProxy = new Proxy(new Proxy(generatorAddThis, {}), { apply: undefined });
var generatorValues = Array.from(Reflect.apply(generatorProxy, { base: 30 }, [7]));

proxy(2, 3) === 6
  && proxy.call(null, 4, 5) === 10
  && Reflect.apply(nestedProxy, { base: 20 }, [3, 4]) === 27
  && boundAddThis(2) === 13
  && nestedBoundProxy(2) === 13
  && nestedBoundProxy.call({ base: 20 }, 3) === 14
  && generatorValues.length === 2
  && generatorValues[0] === 30
  && generatorValues[1] === 7
  && applyCalls === 2;
