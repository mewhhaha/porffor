let values = [10, 20, 30, 40, 50, 60, 70, 80];

function readCapturedArray() {
  return values[0] + values[7];
}

function readArrayParameter(array, firstIndex, secondIndex) {
  return array[firstIndex] + array[secondIndex];
}

function readAliasedArguments(index) {
  let argumentValues = arguments;
  return argumentValues[index];
}

function readNumericNamedProperty(array, propertyKey) {
  return array[propertyKey];
}

async function readAwaitedObject() {
  let object = await Promise.resolve({ 0: 7 });
  return object[0];
}

if (readCapturedArray() !== 90) throw "captured array";
if (readArrayParameter(values, 1, 6) !== 90) throw "array parameter";
if (readAliasedArguments(7, 2, 3, 4, 5, 6, 7, 8) !== 8) {
  throw "aliased arguments";
}
values[-1] = 11;
values[1.5] = 12;
if (readNumericNamedProperty(values, -1) !== 11) throw "negative array property";
if (readNumericNamedProperty(values, 1.5) !== 12) {
  throw "fractional array property";
}
readAwaitedObject().then(function (value) {
  if (value !== 7) throw "awaited object";
  print("dynamic-awaited-object:" + value);
});

126;
