let fixed = new ArrayBuffer(4);
if (fixed.byteLength !== 4) throw "fixed byteLength";
if (fixed.maxByteLength !== 4) throw "fixed maxByteLength";
if (fixed.resizable !== false) throw "fixed resizable";

let resizable = new ArrayBuffer(0, { maxByteLength: 23 });
if (resizable.byteLength !== 0) throw "resizable byteLength";
if (resizable.maxByteLength !== 23) throw "resizable maxByteLength";
if (resizable.resizable !== true) throw "resizable flag";

if (new ArrayBuffer(5, undefined).maxByteLength !== 5) throw "undefined options";
if (new ArrayBuffer(6, 1).maxByteLength !== 6) throw "non-object options";
if (new ArrayBuffer(7, { maxByteLength: undefined }).resizable !== false) {
  throw "undefined maxByteLength";
}

let maxCoercions = 0;
let maxObject = {
  valueOf: function() {
    maxCoercions += 1;
    return 9;
  }
};
let maxOptions = { maxByteLength: maxObject };
let coerced = new ArrayBuffer(3, maxOptions);
if (coerced.maxByteLength !== 9) throw "object maxByteLength";
if (maxCoercions !== 1) throw "object maxByteLength coercion count";

__porfAssertThrows(RangeError, function () {
  new ArrayBuffer(4, { maxByteLength: 3 });
});

__porfAssertThrows(RangeError, function () {
  new ArrayBuffer(0, { maxByteLength: -1 });
});

let abruptMax = {
  valueOf: function() {
    throw "max abrupt";
  }
};
let abruptOptions = { maxByteLength: abruptMax };
let abruptThrew = false;
try {
  new ArrayBuffer(0, abruptOptions);
} catch (error) {
  abruptThrew = true;
}
if (!abruptThrew) throw "maxByteLength abrupt";

123;
