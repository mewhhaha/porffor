let ok = true;
let replacement = {};

let array = [1];
array.constructor = replacement;
let descriptor = Object.getOwnPropertyDescriptor(array, "constructor");
ok = ok && array.constructor === replacement;
ok = ok && Object.hasOwn(array, "constructor");
ok = ok && array.hasOwnProperty("constructor");
ok = ok && descriptor.value === replacement;
ok = ok && descriptor.writable && descriptor.enumerable && descriptor.configurable;
ok = ok && Object.keys(array).join(",") === "0,constructor";
ok = ok && delete array.constructor;
ok = ok && array.constructor === Array;

let nullPrototype = [];
Object.setPrototypeOf(nullPrototype, null);
ok = ok && nullPrototype.constructor === undefined;

let getterReceiver;
let getterPrototype = {};
Object.defineProperty(getterPrototype, "constructor", {
  get() { getterReceiver = this; return replacement; },
  configurable: true
});
let getterArray = [];
Object.setPrototypeOf(getterArray, getterPrototype);
ok = ok && getterArray.constructor === replacement;
ok = ok && getterReceiver === getterArray;

let setterReceiver;
let setterValue;
let setterPrototype = {};
Object.defineProperty(setterPrototype, "constructor", {
  set(value) { setterReceiver = this; setterValue = value; },
  configurable: true
});
let setterArray = [];
Object.setPrototypeOf(setterArray, setterPrototype);
setterArray.constructor = replacement;
ok = ok && setterReceiver === setterArray && setterValue === replacement;
ok = ok && !Object.hasOwn(setterArray, "constructor");

let ordered = [];
ordered.first = 1;
ordered.constructor = 2;
ordered.last = 3;
delete ordered.constructor;
ordered.constructor = 4;
ok = ok && Object.keys(ordered).join(",") === "first,last,constructor";

let sealed = [];
Object.preventExtensions(sealed);
ok = ok && Reflect.set(sealed, "constructor", replacement) === false;
ok = ok && !Object.hasOwn(sealed, "constructor");

ok;
