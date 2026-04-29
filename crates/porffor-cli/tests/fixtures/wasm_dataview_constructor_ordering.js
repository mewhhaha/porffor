let buffer = new ArrayBuffer(8);
let touched = false;
function touchValueOf() {
  touched = true;
  return 0;
}
try {
  DataView(buffer, { valueOf: touchValueOf });
} catch (error) {
  order = order;
}
if (touched) throw "coerced before newTarget";

let detached = new ArrayBuffer(8);
let order = "";
function detachedOffsetValueOf() {
  order = "offset";
  return 0;
}
let offset = {
  valueOf: detachedOffsetValueOf
};
__porfDetachArrayBuffer(detached);
try {
  new DataView(detached, offset);
} catch (error) {
  order = order;
}
if (order !== "offset") throw "detached order";

let duringPrototype = new ArrayBuffer(8);
let customNewTarget = function () {};
function detachAndReturnPrototype() {
  __porfDetachArrayBuffer(duringPrototype);
  return {};
}
Object.defineProperty(customNewTarget, "prototype", {
  get: detachAndReturnPrototype
});
try {
  Reflect.construct(DataView, [duringPrototype, 0], customNewTarget);
} catch (error) {
  order = order;
}

let zeroLength = new ArrayBuffer(0);
let touchedPrototype = false;
let throwingNewTarget = Object.defineProperty(function () {}.bind(), "prototype", {
  get: function () {
    touchedPrototype = true;
    throw "prototype touched";
  }
});
try {
  Reflect.construct(DataView, [zeroLength, 10], throwingNewTarget);
  throw "expected range error";
} catch (error) {
  order = order;
}
if (touchedPrototype) throw "prototype touched before offset validation";

456;
