function expectTypeError(run, label) {
  try {
    run();
  } catch (error) {
    if (!(error instanceof TypeError)) throw label + " wrong error";
    return;
  }
  throw label + " did not throw";
}

function expectRangeError(run, label) {
  try {
    run();
  } catch (error) {
    if (!(error instanceof RangeError)) throw label + " wrong error";
    return;
  }
  throw label + " did not throw";
}

let startDetachSource = new ArrayBuffer(4);
let startDetachOrder = "";
startDetachSource.constructor = {
  [Symbol.species]: function(length) {
    startDetachOrder += "species";
    return new ArrayBuffer(length);
  }
};
expectTypeError(function() {
  startDetachSource.slice({
    valueOf: function() {
      startDetachOrder += "start,";
      __lilaDetachArrayBuffer(startDetachSource);
      return 0;
    }
  }, {
    valueOf: function() {
      startDetachOrder += "end,";
      return 4;
    }
  });
}, "detach in start");
if (startDetachOrder !== "start,end,species") throw "detach in start order";

let speciesDetachSource = new ArrayBuffer(4);
let speciesDetachCalls = 0;
speciesDetachSource.constructor = {
  [Symbol.species]: function(length) {
    speciesDetachCalls += 1;
    __lilaDetachArrayBuffer(speciesDetachSource);
    return new ArrayBuffer(length);
  }
};
expectTypeError(function() {
  speciesDetachSource.slice(0, 4);
}, "detach in species");
if (speciesDetachCalls !== 1) throw "detach in species calls";

let shrinkSource = new ArrayBuffer(8, { maxByteLength: 12 });
let shrinkSourceView = new DataView(shrinkSource);
for (let i = 0; i < 8; i += 1) shrinkSourceView.setUint8(i, 11 + i);
let shrinkTarget;
shrinkSource.constructor = {
  [Symbol.species]: function(length) {
    shrinkTarget = new ArrayBuffer(length);
    let targetView = new DataView(shrinkTarget);
    for (let i = 0; i < length; i += 1) targetView.setUint8(i, 238);
    shrinkSource.resize(4);
    return shrinkTarget;
  }
};
let shrinkResult = shrinkSource.slice(2, 7);
if (shrinkResult !== shrinkTarget) throw "shrink target identity";
if (shrinkResult.byteLength !== 5) throw "shrink requested length";
let shrinkResultView = new DataView(shrinkResult);
if (shrinkResultView.getUint8(0) !== 13) throw "shrink copied byte 0";
if (shrinkResultView.getUint8(1) !== 14) throw "shrink copied byte 1";
if (shrinkResultView.getUint8(2) !== 238) throw "shrink preserved suffix 2";
if (shrinkResultView.getUint8(3) !== 238) throw "shrink preserved suffix 3";
if (shrinkResultView.getUint8(4) !== 238) throw "shrink preserved suffix 4";

let belowFirstSource = new ArrayBuffer(8, { maxByteLength: 12 });
let belowFirstTarget;
belowFirstSource.constructor = {
  [Symbol.species]: function(length) {
    belowFirstTarget = new ArrayBuffer(length);
    let targetView = new DataView(belowFirstTarget);
    for (let i = 0; i < length; i += 1) targetView.setUint8(i, 221);
    belowFirstSource.resize(2);
    return belowFirstTarget;
  }
};
let belowFirstResult = belowFirstSource.slice(5, 8);
if (belowFirstResult !== belowFirstTarget) throw "below first target identity";
if (belowFirstResult.byteLength !== 3) throw "below first requested length";
let belowFirstView = new DataView(belowFirstResult);
if (belowFirstView.getUint8(0) !== 221) throw "below first suffix 0";
if (belowFirstView.getUint8(1) !== 221) throw "below first suffix 1";
if (belowFirstView.getUint8(2) !== 221) throw "below first suffix 2";

let immutableSource = new ArrayBuffer(4);
let immutableOrder = "";
expectTypeError(function() {
  immutableSource.sliceToImmutable({
    valueOf: function() {
      immutableOrder += "start,";
      __lilaDetachArrayBuffer(immutableSource);
      return 0;
    }
  }, {
    valueOf: function() {
      immutableOrder += "end";
      return 4;
    }
  });
}, "sliceToImmutable detach");
if (immutableOrder !== "start,end") throw "sliceToImmutable detach order";

let immutableShrinkSource = new ArrayBuffer(8, { maxByteLength: 12 });
let immutableShrinkOrder = "";
expectRangeError(function() {
  immutableShrinkSource.sliceToImmutable({
    valueOf: function() {
      immutableShrinkOrder += "start,";
      return 2;
    }
  }, {
    valueOf: function() {
      immutableShrinkOrder += "end";
      immutableShrinkSource.resize(4);
      return 7;
    }
  });
}, "sliceToImmutable shrink below final");
if (immutableShrinkOrder !== "start,end") {
  throw "sliceToImmutable shrink order";
}

123;
