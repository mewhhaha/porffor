let ok = true;

let keyCalls = 0;
const missing = null;
ok = ok && missing?.[keyCalls++] === undefined;
ok = ok && keyCalls === 0;

let baseCalls = 0;
function base() {
  baseCalls += 1;
  return { first: { second: 7 } };
}
ok = ok && base()?.first.second === 7;
ok = ok && baseCalls === 1;

ok = ok && null?.first.second === undefined;

let groupedThrew = false;
try {
  (null?.first).second;
} catch (error) {
  groupedThrew = error instanceof TypeError;
}
ok = ok && groupedThrew;

let ordinaryTailThrew = false;
try {
  ({ first: null })?.first.second;
} catch (error) {
  ordinaryTailThrew = error instanceof TypeError;
}
ok = ok && ordinaryTailThrew;

let getterThrowCaught = false;
try {
  ({ get value() { throw 7; } })?.value;
} catch (error) {
  getterThrowCaught = error === 7;
}
ok = ok && getterThrowCaught;

function throwingKey() {
  throw 8;
}
let keyThrowCaught = false;
try {
  ({})?.[throwingKey()];
} catch (error) {
  keyThrowCaught = error === 8;
}
ok = ok && keyThrowCaught;

ok = ok && typeof (42)?.toString === "function";
ok = ok && typeof true?.valueOf === "function";
ok = ok && typeof 1n?.toString === "function";
ok = ok && typeof Symbol("s")?.valueOf === "function";
ok = ok && "hello"?.length === 5;
ok = ok && "hello"?.[1] === "e";

const originalStringToString = String.prototype.toString;
String.prototype.optionalMarker = 7;
String.prototype.toString = 8;
ok = ok && "value"?.optionalMarker === 7;
ok = ok && "value"?.toString === 8;
String.prototype.toString = originalStringToString;

const originalBigIntToString = BigInt.prototype.toString;
BigInt.prototype.optionalMarker = 9;
BigInt.prototype.toString = 10;
ok = ok && 1n?.optionalMarker === 9;
ok = ok && 1n?.toString === 10;
BigInt.prototype.toString = originalBigIntToString;

const values = [39, 42];
values.true = "named";
values[1.1] = "fractional";
ok = ok && values?.[0] === 39;
ok = ok && values?.[0, 1] === 42;
ok = ok && values?.[true] === "named";
ok = ok && values?.[1.1] === "fractional";

function runtimeNamedArrayKey(key, name) {
  const target = [];
  target[key] = 11;
  return target.length === 0 && target[name] === 11;
}
ok = ok && runtimeNamedArrayKey(1.1, "1.1");
ok = ok && runtimeNamedArrayKey(-1, "-1");
ok = ok && runtimeNamedArrayKey(NaN, "NaN");
ok = ok && runtimeNamedArrayKey(Infinity, "Infinity");

const indexed = [1, 2];
let runtimeIndex = 1;
indexed[runtimeIndex] = 12;
ok = ok && indexed.length === 2;
ok = ok && indexed[1] === 12;

let assignmentOrder = "";
function orderedKey() {
  assignmentOrder += "k";
  return 0;
}
function orderedValue() {
  assignmentOrder += "v";
  return 13;
}
const ordered = [];
ordered[orderedKey()] = orderedValue();
ok = ok && assignmentOrder === "kv";
ok = ok && ordered[0] === 13;

ok;
