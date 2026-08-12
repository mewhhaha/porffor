var TA = Uint8Array;
var called = 0;

function ctor(len) {
  called++;
  if (len !== 3) throw "captured ctor length";
  return new TA(len);
}

var result = TA.from.call(ctor, [5, 6, 7]);

if (called !== 1) throw "captured ctor called";
if (result.length !== 3) throw "captured ctor result length";
if (result[0] !== 5) throw "captured ctor first";
if (result[1] !== 6) throw "captured ctor second";
if (result[2] !== 7) throw "captured ctor third";
if (result.constructor !== TA) throw "captured ctor constructor";
if (Object.getPrototypeOf(result) !== TA.prototype) throw "captured ctor prototype";

262;
