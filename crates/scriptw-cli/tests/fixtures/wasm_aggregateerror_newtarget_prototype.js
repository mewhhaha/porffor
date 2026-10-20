let customProto = { marker: 1 };
function CustomNewTarget() {}
CustomNewTarget.prototype = customProto;

let custom = Reflect.construct(AggregateError, [[]], CustomNewTarget);
if (Object.getPrototypeOf(custom) !== customProto) {
  throw "custom NewTarget prototype";
}

let proxyProto = { marker: 2 };
let trapCount = 0;
function ProxyTarget() {}
let proxyNewTarget = new Proxy(ProxyTarget, {
  get: function(target, property) {
    if (property === "prototype") {
      trapCount = trapCount + 1;
      return proxyProto;
    }
    return target[property];
  }
});

let proxyResult = Reflect.construct(AggregateError, [[]], proxyNewTarget);
if (Object.getPrototypeOf(proxyResult) !== proxyProto) {
  throw "proxy NewTarget prototype";
}
if (trapCount !== 1) throw "prototype trap count";

function PrimitiveNewTarget() {}
PrimitiveNewTarget.prototype = 7;
let primitiveFallback = Reflect.construct(AggregateError, [[]], PrimitiveNewTarget);
if (Object.getPrototypeOf(primitiveFallback) !== AggregateError.prototype) {
  throw "primitive prototype fallback";
}

function UndefinedNewTarget() {}
UndefinedNewTarget.prototype = undefined;
let undefinedFallback = Reflect.construct(AggregateError, [[]], UndefinedNewTarget);
if (Object.getPrototypeOf(undefinedFallback) !== AggregateError.prototype) {
  throw "undefined prototype fallback";
}

function NullNewTarget() {}
NullNewTarget.prototype = null;
let nullFallback = Reflect.construct(AggregateError, [[]], NullNewTarget);
if (Object.getPrototypeOf(nullFallback) !== AggregateError.prototype) {
  throw "null prototype fallback";
}

let loopTrapCount = 0;
let values = [8, { marker: 3 }];
function LoopProxyTarget() {}
for (const value of values) {
  let loopProxyNewTarget = new Proxy(LoopProxyTarget, {
    get(target, property) {
      if (property === "prototype") {
        loopTrapCount = loopTrapCount + 1;
        return value;
      }
      return target[property];
    }
  });
  let result = Reflect.construct(AggregateError, [[]], loopProxyNewTarget);
  let expected = value === 8 ? AggregateError.prototype : value;
  if (Object.getPrototypeOf(result) !== expected) {
    throw "for-of proxy NewTarget prototype";
  }
}
if (loopTrapCount !== 2) throw "for-of prototype trap count";

if (AggregateError.length !== 2) throw "AggregateError length";
if (Object.getPrototypeOf(AggregateError) !== Error) throw "AggregateError inheritance";
if (AggregateError.prototype.constructor !== AggregateError) throw "prototype constructor";

123;
