let getterOnly = [1];
let getterCalls = 0;
Object.defineProperty(getterOnly, Symbol.isConcatSpreadable, {
  get() {
    getterCalls += 1;
    return false;
  }
});

getterOnly[Symbol.isConcatSpreadable] = true;
let getterOnlyStrictThrow = false;
try {
  (function() {
    "use strict";
    getterOnly[Symbol.isConcatSpreadable] = true;
  })();
} catch (error) {
  getterOnlyStrictThrow = error instanceof TypeError;
}
let getterOnlyRead = getterOnly[Symbol.isConcatSpreadable];
let getterOnlyResult = [].concat(getterOnly);

let setterArray = [2];
let setterReceiver = false;
let setterValue = false;
Object.defineProperty(setterArray, Symbol.isConcatSpreadable, {
  get() {
    return false;
  },
  set(value) {
    setterReceiver = this === setterArray;
    setterValue = value;
  }
});
setterArray[Symbol.isConcatSpreadable] = true;
let setterResult = [].concat(setterArray);

let nonWritable = [3];
Object.defineProperty(nonWritable, Symbol.isConcatSpreadable, {
  value: false,
  writable: false
});
nonWritable[Symbol.isConcatSpreadable] = true;
let nonWritableStrictThrow = false;
try {
  (function() {
    "use strict";
    nonWritable[Symbol.isConcatSpreadable] = true;
  })();
} catch (error) {
  nonWritableStrictThrow = error instanceof TypeError;
}
let nonWritableResult = [].concat(nonWritable);

getterOnlyStrictThrow
  && getterOnlyRead === false
  && getterCalls === 2
  && getterOnlyResult.length === 1
  && getterOnlyResult[0] === getterOnly
  && setterReceiver
  && setterValue === true
  && setterResult.length === 1
  && setterResult[0] === setterArray
  && nonWritableStrictThrow
  && nonWritable[Symbol.isConcatSpreadable] === false
  && nonWritableResult.length === 1
  && nonWritableResult[0] === nonWritable;
