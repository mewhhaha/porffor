var originalNumberToString = Object.getOwnPropertyDescriptor(
  Number.prototype,
  "toString"
);
var objectGetterThis;
var objectCallThis;

Object.defineProperty(Number.prototype, "toString", {
  configurable: true,
  get: function() {
    objectGetterThis = this;
    return function() {
      "use strict";
      objectCallThis = this;
      return "object";
    };
  }
});

var objectResult = Object.prototype.toLocaleString.call(1);
Object.defineProperty(Number.prototype, "toString", originalNumberToString);

var originalNumberToLocaleString = Object.getOwnPropertyDescriptor(
  Number.prototype,
  "toLocaleString"
);
var arrayGetterThis;
var arrayCallThis;

Object.defineProperty(Number.prototype, "toLocaleString", {
  configurable: true,
  get: function() {
    arrayGetterThis = this;
    return function() {
      "use strict";
      arrayCallThis = this;
      return "array";
    };
  }
});

var arrayResult = [2].toLocaleString();
Object.defineProperty(
  Number.prototype,
  "toLocaleString",
  originalNumberToLocaleString
);

objectResult === "object" &&
  typeof objectGetterThis === "object" &&
  objectCallThis === objectGetterThis &&
  arrayResult === "array" &&
  typeof arrayGetterThis === "object" &&
  arrayCallThis === 2;
