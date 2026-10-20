let view = new DataView(new ArrayBuffer(8));
let boxed = Object(Symbol("boxed"));
let score = 0;

if (Reflect.apply(Symbol.prototype[Symbol.toPrimitive], boxed, ["number"]) === boxed.valueOf()) {
  score += 1;
}

boxed[Symbol.toPrimitive] = function () { return 0; };
if (view.getBigInt64(boxed) === 0n) score += 1;
if (view.getBigUint64(boxed) === 0n) score += 1;

score === 3 ? 3 : score;
