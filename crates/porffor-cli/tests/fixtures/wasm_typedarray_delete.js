function assertSame(actual, expected, label) {
  if (actual !== expected) throw label;
}

function deleteStrict(view, key) {
  "use strict";
  return delete view[key];
}

function assertStrictDeleteThrows(view, key, label) {
  var threw = false;
  try {
    deleteStrict(view, key);
  } catch (error) {
    threw = error instanceof TypeError;
  }
  assertSame(threw, true, label);
}

var numeric = new Uint8Array([42]);
var dynamicNegativeZero = -0;
assertSame(delete numeric[0], false, "numeric index");
assertSame(delete numeric[-0], false, "numeric minus zero becomes zero");
assertSame(delete numeric[dynamicNegativeZero], false, "dynamic numeric minus zero becomes zero");
assertSame(delete numeric["-0"], true, "string minus zero");
assertSame(delete numeric["1.1"], true, "fractional index");
assertSame(delete numeric[-1], true, "negative index");
assertSame(delete numeric[1], true, "out of bounds index");
assertSame(delete numeric.Infinity, true, "infinity index");
assertStrictDeleteThrows(numeric, 0, "strict numeric index");
assertStrictDeleteThrows(numeric, -0, "strict dynamic numeric minus zero index");
assertSame(deleteStrict(numeric, "-0"), true, "strict invalid index");

var bigint = new BigInt64Array([42n]);
assertSame(delete bigint[0], false, "bigint index");
assertStrictDeleteThrows(bigint, 0, "strict bigint index");

var sharedNumeric = new Uint8Array(new SharedArrayBuffer(1));
var sharedBigint = new BigInt64Array(new SharedArrayBuffer(8));
assertSame(delete sharedNumeric[0], false, "shared numeric index");
assertSame(delete sharedBigint[0], false, "shared bigint index");

numeric.ordinary = 1;
assertSame(delete numeric.ordinary, true, "ordinary configurable property");
assertSame(Reflect.has(numeric, "ordinary"), false, "ordinary property removed");
Object.defineProperty(numeric, "fixed", { value: 1, configurable: false });
assertSame(delete numeric.fixed, false, "ordinary non-configurable property");
assertStrictDeleteThrows(numeric, "fixed", "strict ordinary non-configurable property");

var getterCalls = 0;
Object.defineProperty(numeric, "accessor", {
  configurable: true,
  get: function() {
    getterCalls = getterCalls + 1;
    throw "getter called";
  }
});
assertSame(delete numeric.accessor, true, "ordinary accessor property");
assertSame(getterCalls, 0, "delete does not invoke getter");

var symbol = Symbol("typed array delete");
numeric[symbol] = 1;
assertSame(delete numeric[symbol], true, "symbol property");
assertSame(Reflect.has(numeric, symbol), false, "symbol property removed");

var detached = new Uint8Array([9]);
detached.ordinary = 1;
__porfDetachArrayBuffer(detached.buffer);
assertSame(delete detached[0], true, "detached index");
assertSame(delete detached.ordinary, true, "detached ordinary property");

var other = __porfCreateRealm().global;
var otherDetached = new other.Uint8Array(1);
__porfDetachArrayBuffer(otherDetached.buffer);
assertSame(delete otherDetached[0], true, "cross-realm detached index");

assertSame(Reflect.deleteProperty(numeric, 0), false, "Reflect valid index");
assertSame(Reflect.deleteProperty(numeric, "-0"), true, "Reflect invalid index");

var trapCount = 0;
var proxy = new Proxy(numeric, {
  deleteProperty: function(target, key) {
    trapCount = trapCount + 1;
    return true;
  }
});
assertSame(delete proxy[0], true, "proxy trap result");
assertSame(trapCount, 1, "proxy trap runs before typed array delete");

var rejectingProxy = new Proxy(numeric, {
  deleteProperty: function() {
    return false;
  }
});
assertSame(delete rejectingProxy[0], false, "proxy false result");
assertStrictDeleteThrows(rejectingProxy, 0, "strict proxy false result");

true;
