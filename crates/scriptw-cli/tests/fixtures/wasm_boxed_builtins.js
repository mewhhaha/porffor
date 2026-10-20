function isNumberBox() { return this instanceof Number; }
function isStringBox() { return this instanceof String; }
function isBooleanBox() { return this instanceof Boolean; }

let boxedNumber = Object(1);
let boxedString = new Object("x");
let boxedBoolean = Object(true);

typeof Number === "function"
  && Number === globalThis.Number
  && String === globalThis.String
  && Boolean === globalThis.Boolean
  && boxedNumber instanceof Number
  && boxedString instanceof String
  && Object.getPrototypeOf(boxedBoolean) === Boolean.prototype
  && isNumberBox.call(1)
  && isStringBox.apply("x", [])
  && isBooleanBox.call(false)
  && new Number(1) + 1 === 2
  && new String("x") + "y" === "xy"
  && new Boolean(false) == false;
