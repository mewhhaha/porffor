let constructorDesc = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype,
  "constructor"
);
if (ArrayBuffer.prototype.constructor !== ArrayBuffer) throw "constructor identity";
if (constructorDesc === undefined) throw "constructor descriptor";

let direct = new ArrayBuffer(4);
if (direct.byteLength !== 4) throw "direct constructor";

let noNewThrew = false;
try {
  ArrayBuffer(4);
} catch (error) {
  noNewThrew = true;
}
if (!noNewThrew) throw "undefined NewTarget";

let objectProtoBuffer = Reflect.construct(ArrayBuffer, [8], Object);
if (Object.getPrototypeOf(objectProtoBuffer) !== Object.prototype) {
  throw "Object NewTarget prototype";
}

let newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, "prototype", {
  get: function() {
    return Array.prototype;
  }
});
let boundProtoBuffer = Reflect.construct(ArrayBuffer, [16], newTarget);
if (Object.getPrototypeOf(boundProtoBuffer) !== Array.prototype) {
  throw "bound NewTarget prototype";
}

let lengthObject = {
  valueOf: function() {
    return 42;
  }
};
if (new ArrayBuffer(lengthObject).byteLength !== 42) throw "object ToIndex";
if (new ArrayBuffer(-0.1).byteLength !== 0) throw "negative fractional ToIndex";

let abruptObject = {
  valueOf: function() {
    throw "length abrupt";
  }
};
let abruptThrew = false;
try {
  new ArrayBuffer(abruptObject);
} catch (error) {
  abruptThrew = true;
}
if (!abruptThrew) throw "object ToIndex abrupt";

123;
