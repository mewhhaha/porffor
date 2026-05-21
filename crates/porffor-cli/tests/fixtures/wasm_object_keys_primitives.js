let ok = true;

function expectEmptyArray(value) {
  let keys = Object.keys(value);
  if (!Array.isArray(keys)) ok = false;
  if (keys.length !== 0) ok = false;
}

expectEmptyArray(0);
expectEmptyArray(true);

let stringKeys = Object.keys("abc");
if (!Array.isArray(stringKeys)) ok = false;
if (stringKeys.length !== 3) ok = false;
if (stringKeys[0] !== "0") ok = false;
if (stringKeys[1] !== "1") ok = false;
if (stringKeys[2] !== "2") ok = false;

let boxedString = new String("abc");
let boxedStringKeys = Object.keys(boxedString);
if (!Array.isArray(boxedStringKeys)) ok = false;
if (boxedStringKeys.length !== 3) ok = false;
if (boxedStringKeys[0] !== "0") ok = false;
if (boxedStringKeys[1] !== "1") ok = false;
if (boxedStringKeys[2] !== "2") ok = false;

let boxedStringSeen = [];
for (var boxedKey in boxedString) {
  if (boxedString.hasOwnProperty(boxedKey)) {
    boxedStringSeen.push(boxedKey);
  }
}
if (boxedStringSeen.length !== 3) ok = false;
if (boxedStringSeen[0] !== "0") ok = false;
if (boxedStringSeen[1] !== "1") ok = false;
if (boxedStringSeen[2] !== "2") ok = false;

function captureArguments(a, b, c) {
  return arguments;
}

let args = captureArguments(1, "b", false);
let argsKeys = Object.keys(args);
if (argsKeys.length !== 3) ok = false;
if (argsKeys[0] !== "0") ok = false;
if (argsKeys[1] !== "1") ok = false;
if (argsKeys[2] !== "2") ok = false;

let argsSeen = [];
for (var argsKey in args) {
  if (args.hasOwnProperty(argsKey)) {
    argsSeen.push(argsKey);
  }
}
if (argsSeen.length !== 3) ok = false;
if (argsSeen[0] !== "0") ok = false;
if (argsSeen[1] !== "1") ok = false;
if (argsSeen[2] !== "2") ok = false;

let objectWithKeys = new Date(0);
objectWithKeys.prop1 = 100;
objectWithKeys.prop2 = "prop2";
let objectSeen = [];
for (var objectKey in objectWithKeys) {
  if (objectWithKeys.hasOwnProperty(objectKey)) {
    objectSeen.push(objectKey);
  }
}
let objectKeys = Object.keys(objectWithKeys);
if (objectSeen.length !== 2) ok = false;
if (objectSeen[0] !== "prop1") ok = false;
if (objectSeen[1] !== "prop2") ok = false;
if (objectKeys.length !== 2) ok = false;
if (objectKeys[0] !== objectSeen[0]) ok = false;
if (objectKeys[1] !== objectSeen[1]) ok = false;

let arrayKeys = Object.keys([1, 2]);
if (!Array.isArray(arrayKeys)) ok = false;
if (arrayKeys.length !== 2) ok = false;
if (Object.isSealed(arrayKeys) !== false) ok = false;
if (Object.isFrozen(arrayKeys) !== false) ok = false;
if (arrayKeys.hasOwnProperty(0) !== true) ok = false;
if (arrayKeys[0] !== "0") ok = false;
if (arrayKeys[1] !== "1") ok = false;

let iteratedArrayKeys = Object.keys([1, 2, 3, 4, 5]);
let iteratedIndex = 0;
for (var key in iteratedArrayKeys) {
  if (iteratedArrayKeys.hasOwnProperty(key)) {
    if (iteratedArrayKeys[key] !== iteratedIndex.toString()) ok = false;
    iteratedIndex++;
  }
}
if (iteratedIndex !== 5) ok = false;

let nullRejected = false;
try {
  Object.keys(null);
} catch (error) {
  nullRejected = error instanceof TypeError;
}

let undefinedRejected = false;
try {
  Object.keys(undefined);
} catch (error) {
  undefinedRejected = error instanceof TypeError;
}

if (nullRejected !== true) ok = false;
if (undefinedRejected !== true) ok = false;

ok;
