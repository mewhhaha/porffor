function throwsTypeError(thunk) {
  try {
    thunk();
  } catch (error) {
    return error.name === "TypeError";
  }
  return false;
}

let nonCallableLengthOrder = "";
let nonCallableReceiver = {
  get length() {
    nonCallableLengthOrder = nonCallableLengthOrder + "length";
    return 0;
  }
};
let nonCallableThrew = throwsTypeError(function () {
  Array.prototype.sort.call(nonCallableReceiver, null);
});

let undefinedComparatorLengthOrder = "";
let undefinedComparatorReceiver = {
  get length() {
    undefinedComparatorLengthOrder = undefinedComparatorLengthOrder + "length";
    return 0;
  }
};
Array.prototype.sort.call(undefinedComparatorReceiver, undefined);

let comparatorMarker = {};
let beforeWrite = [2, 1];
let comparatorThrew = false;
try {
  beforeWrite.sort(function () {
    throw comparatorMarker;
  });
} catch (error) {
  comparatorThrew = error === comparatorMarker;
}

let numberMarker = {};
let beforeNumber = [2, 1];
let numberThrew = false;
try {
  beforeNumber.sort(function () {
    return {
      valueOf: function () {
        throw numberMarker;
      }
    };
  });
} catch (error) {
  numberThrew = error === numberMarker;
}

let readonly = [2, 1];
Object.defineProperty(readonly, "0", { writable: false });
let strictSetThrew = throwsTypeError(function () {
  readonly.sort();
});

let nonConfigurableTrailing = [2, , 1];
Object.defineProperty(nonConfigurableTrailing, "2", { configurable: false });
let strictDeleteThrew = throwsTypeError(function () {
  nonConfigurableTrailing.sort();
});

throwsTypeError(function () {
  Array.prototype.sort.call(null, undefined);
})
  && throwsTypeError(function () {
    Array.prototype.sort.call(undefined);
  })
  && nonCallableThrew
  && nonCallableLengthOrder === ""
  && undefinedComparatorLengthOrder === "length"
  && comparatorThrew
  && beforeWrite[0] === 2
  && beforeWrite[1] === 1
  && numberThrew
  && beforeNumber[0] === 2
  && beforeNumber[1] === 1
  && strictSetThrew
  && readonly[0] === 2
  && readonly[1] === 1
  && strictDeleteThrew
  && nonConfigurableTrailing[0] === 1
  && nonConfigurableTrailing[1] === 2
  && nonConfigurableTrailing[2] === 1;
