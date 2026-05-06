let resize = ArrayBuffer.prototype.resize;
let desc = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, "resize");
if (desc === undefined) throw "resize descriptor";

__porfAssertThrows(TypeError, function () {
  resize.call(new ArrayBuffer(4), 1);
});

__porfAssertThrows(RangeError, function () {
  resize.call(new ArrayBuffer(1, { maxByteLength: 4 }), -1);
});

__porfAssertThrows(RangeError, function () {
  resize.call(new ArrayBuffer(1, { maxByteLength: 4 }), 5);
});

let detached = new ArrayBuffer(1, { maxByteLength: 4 });
__porfDetachArrayBuffer(detached);
let coercions = 0;
let lengthObject = {
  valueOf: function() {
    coercions += 1;
    return 1;
  }
};
__porfAssertThrows(TypeError, function () {
  resize.call(detached, lengthObject);
});
if (coercions !== 1) throw "coercion before detach";

let resizable = new ArrayBuffer(2, { maxByteLength: 4 });
let resizeResult = resize.call(resizable, 3);
if (resizeResult !== undefined) throw "resize result";
if (resizable.detached !== false) throw "resized detached";
if (resizable.byteLength !== 3) throw "resized byteLength";
if (resizable.maxByteLength !== 4) throw "resized maxByteLength";

let fixed = new ArrayBuffer(4);
__porfAssertThrows(TypeError, function () {
  fixed.resize(0);
});
if (fixed.detached !== false) throw "fixed direct resize detached";
if (fixed.byteLength !== 4) throw "fixed direct resize byteLength";

let sameSize = new ArrayBuffer(2, { maxByteLength: 4 });
let sameSizeView = new DataView(sameSize);
sameSize.resize(2);
if (sameSize.detached !== false) throw "same-size direct resize detached";
if (sameSize.byteLength !== 2) throw "same-size direct resize byteLength";
if (sameSizeView.byteLength !== 2) throw "same-size direct resize view";

let grow = new ArrayBuffer(2, { maxByteLength: 4 });
let growView = new DataView(grow);
grow.resize(4);
if (grow.detached !== false) throw "grow direct resize detached";
if (grow.byteLength !== 4) throw "grow direct resize byteLength";
if (growView.byteLength !== 4) throw "grow direct resize view";

let shrink = new ArrayBuffer(2, { maxByteLength: 4 });
let shrinkView = new DataView(shrink);
shrink.resize(1);
if (shrink.detached !== false) throw "shrink direct resize detached";
if (shrink.byteLength !== 1) throw "shrink direct resize byteLength";
if (shrinkView.byteLength !== 1) throw "shrink direct resize view";

let detachedDuringCoercion = new ArrayBuffer(1, { maxByteLength: 4 });
let directCoercions = 0;
__porfAssertThrows(TypeError, function () {
  detachedDuringCoercion.resize({
    valueOf: function() {
      directCoercions += 1;
      __porfDetachArrayBuffer(detachedDuringCoercion);
      return 1;
    }
  });
});
if (directCoercions !== 1) throw "direct coercion before detach";
if (detachedDuringCoercion.detached !== true) throw "direct coercion detach side effect";

123;
