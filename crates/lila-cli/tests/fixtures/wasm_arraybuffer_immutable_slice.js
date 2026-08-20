let source = new ArrayBuffer(4);
let sourceView = new DataView(source);
sourceView.setUint8(0, 17);
sourceView.setUint8(1, 34);

let immutable = source.transferToImmutable();
if (source.detached !== true) throw "transferToImmutable source detached";
if (immutable.byteLength !== 4) throw "immutable length";
if (immutable.maxByteLength !== 4) throw "immutable max";
if (immutable.resizable !== false) throw "immutable fixed";
let immutableView = new DataView(immutable);
if (immutableView.getUint8(0) !== 17) throw "immutable byte 0";
if (immutableView.getUint8(1) !== 34) throw "immutable byte 1";

let transferCoercions = 0;
function transferValueOf() {
  transferCoercions += 1;
  return 2;
}
let transferThrew = false;
try {
  immutable.transfer({
    valueOf: transferValueOf
  });
} catch (error) {
  transferThrew = true;
}
if (!transferThrew) throw "immutable transfer throws";
if (transferCoercions !== 1) throw "immutable transfer coercion order";

let fixedCoercions = 0;
function fixedValueOf() {
  fixedCoercions += 1;
  return 2;
}
let fixedThrew = false;
try {
  immutable.transferToFixedLength({
    valueOf: fixedValueOf
  });
} catch (error) {
  fixedThrew = true;
}
if (!fixedThrew) throw "immutable transferToFixedLength throws";
if (fixedCoercions !== 1) throw "immutable transferToFixedLength coercion order";

let resizeCoercions = 0;
function resizeValueOf() {
  resizeCoercions += 1;
  return 2;
}
let resizeThrew = false;
try {
  immutable.resize({
    valueOf: resizeValueOf
  });
} catch (error) {
  resizeThrew = true;
}
if (!resizeThrew) throw "immutable resize throws";
if (resizeCoercions !== 0) throw "immutable resize coercion order";

let speciesSource = new ArrayBuffer(4);
let speciesConstructor = {};
speciesConstructor[Symbol.species] = function(length) {
  return speciesSource.sliceToImmutable(0, length);
};
speciesSource.constructor = speciesConstructor;

let speciesThrew = false;
try {
  speciesSource.slice(0, 2);
} catch (error) {
  speciesThrew = true;
}
if (!speciesThrew) throw "species immutable rejected";

let slicedImmutable = immutable.sliceToImmutable(1, 3);
if (slicedImmutable.byteLength !== 2) throw "sliceToImmutable length";
let slicedView = new DataView(slicedImmutable);
if (slicedView.getUint8(0) !== 34) throw "sliceToImmutable byte";

123;
