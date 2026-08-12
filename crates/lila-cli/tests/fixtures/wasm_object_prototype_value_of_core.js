let valueOf = Object.prototype.valueOf;
let object = {};
let array = [];
let callable = function () {};
let boxedBoolean = valueOf.call(true);
let boxedNumber = valueOf.call(13);
let boxedString = valueOf.call("lila");
let symbol = Symbol("lila");
let boxedSymbol = valueOf.call(symbol);
let bigint = 13n;
let boxedBigInt = valueOf.call(bigint);
let other = __lilaCreateRealm().global;
let otherValueOf = other.Object.prototype.valueOf;
let otherBoxedBoolean = otherValueOf.call(true);
let otherBoxedNumber = otherValueOf.call(13);
let otherBoxedString = otherValueOf.call("lila");
let otherBoxedSymbol = otherValueOf.call(symbol);
let otherBoxedBigInt = otherValueOf.call(bigint);
let otherNullThrows = false;
try {
  otherValueOf.call(null);
} catch (error) {
  otherNullThrows = error instanceof other.TypeError
    && !(error instanceof TypeError);
}

let nullThrows = false;
let undefinedThrows = false;
try {
  valueOf.call(null);
} catch (error) {
  nullThrows = error instanceof TypeError;
}
try {
  valueOf.call(undefined);
} catch (error) {
  undefinedThrows = error instanceof TypeError;
}

let hadLength = valueOf.hasOwnProperty("length");
let deletedLength = delete valueOf.length;
let deletionPersists = Object.prototype.valueOf === valueOf
  && !Object.prototype.valueOf.hasOwnProperty("length");

Object.prototype.valueOf = function () {
  return 73;
};
let overrideObserved = object.valueOf() === 73;
Object.prototype.valueOf = valueOf;

valueOf.call(object) === object
  && valueOf.call(array) === array
  && valueOf.call(callable) === callable
  && typeof boxedBoolean === "object"
  && Object.getPrototypeOf(boxedBoolean) === Boolean.prototype
  && boxedBoolean.valueOf() === true
  && typeof boxedNumber === "object"
  && Object.getPrototypeOf(boxedNumber) === Number.prototype
  && boxedNumber.valueOf() === 13
  && typeof boxedString === "object"
  && Object.getPrototypeOf(boxedString) === String.prototype
  && boxedString.valueOf() === "lila"
  && typeof boxedSymbol === "object"
  && Object.getPrototypeOf(boxedSymbol) === Symbol.prototype
  && boxedSymbol.valueOf() === symbol
  && typeof boxedBigInt === "object"
  && Object.getPrototypeOf(boxedBigInt) === BigInt.prototype
  && boxedBigInt.valueOf() === bigint
  && Object.getPrototypeOf(otherBoxedBoolean) === other.Boolean.prototype
  && otherBoxedBoolean.valueOf() === true
  && Object.getPrototypeOf(otherBoxedNumber) === other.Number.prototype
  && otherBoxedNumber.valueOf() === 13
  && Object.getPrototypeOf(otherBoxedString) === other.String.prototype
  && otherBoxedString.valueOf() === "lila"
  && Object.getPrototypeOf(otherBoxedSymbol) === other.Symbol.prototype
  && otherBoxedSymbol.valueOf() === symbol
  && Object.getPrototypeOf(otherBoxedBigInt) === other.BigInt.prototype
  && otherBoxedBigInt.valueOf() === bigint
  && otherNullThrows
  && nullThrows
  && undefinedThrows
  && hadLength
  && deletedLength
  && deletionPersists
  && overrideObserved;
