let otherGlobal = __porfCreateRealm().global;
let otherArray = new otherGlobal.Array();

function setDynamicLength(target, key, value) {
  target[key] = value;
}

let conversions = 0;
setDynamicLength(otherArray, "length", {
  valueOf: function () {
    conversions = conversions + 1;
    return 4;
  }
});
if (conversions !== 2 || otherArray.length !== 4) {
  throw "cross-realm dynamic length set";
}

let overflowThrew = false;
try {
  setDynamicLength(otherArray, "length", 4294967296);
} catch (error) {
  overflowThrew = error instanceof RangeError;
}
if (!overflowThrew || otherArray.length !== 4) {
  throw "cross-realm dynamic length overflow";
}

let foreignObjectRangeError = false;
try {
  otherGlobal.Object.defineProperty([], "length", { value: -1 });
} catch (error) {
  foreignObjectRangeError = error instanceof otherGlobal.RangeError
    && !(error instanceof RangeError);
}
if (!foreignObjectRangeError) {
  throw "foreign Object.defineProperty RangeError realm";
}

let foreignReflectRangeError = false;
try {
  otherGlobal.Reflect.defineProperty([], "length", { value: 4294967296 });
} catch (error) {
  foreignReflectRangeError = error instanceof otherGlobal.RangeError
    && !(error instanceof RangeError);
}
if (!foreignReflectRangeError) {
  throw "foreign Reflect.defineProperty RangeError realm";
}

let foreignReflectSetRangeError = false;
try {
  otherGlobal.Reflect.set([], "length", -1);
} catch (error) {
  foreignReflectSetRangeError = error instanceof otherGlobal.RangeError
    && !(error instanceof RangeError);
}
if (!foreignReflectSetRangeError) {
  throw "foreign Reflect.set RangeError realm";
}

true;
