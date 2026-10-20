function catchesReferenceError(callback) {
  try {
    callback();
  } catch (error) {
    return error instanceof ReferenceError && error.name === "ReferenceError";
  }
  return false;
}

if (!catchesReferenceError(function () {
  for (let read = () => later, observed = read(), later = 2; false;) {}
})) {
  throw "classic for initializer closure";
}

if (!catchesReferenceError(function () {
  let { value } = { value: (() => value)() };
})) {
  throw "object pattern initializer closure";
}

if (!catchesReferenceError(function () {
  let [value] = [(() => value)()];
})) {
  throw "array pattern initializer closure";
}

if (!catchesReferenceError(function () {
  let value = 1;
  for (let { value } of [{ value }]) {}
})) {
  throw "for-of object pattern head";
}

var readInitialized;
for (let capture = readInitialized = () => later, later = 2; false;) {}
if (readInitialized() !== 2) {
  throw "initialized eventual binding";
}

var turn = 0;
var secondEntryCaught = false;
while (turn < 2) {
  try {
    let read = () => later;
    let observed = turn === 0 ? undefined : read();
    let later = turn + 1;
  } catch (error) {
    if (turn !== 1 || !(error instanceof ReferenceError)) {
      throw error;
    }
    secondEntryCaught = true;
  }
  turn++;
}
if (!secondEntryCaught) {
  throw "re-entered lexical scope reset";
}

if (!catchesReferenceError(function () {
  let value = 1;
  {
    value;
    let value = 2;
  }
})) {
  throw "block shadow read before declaration";
}

if (!catchesReferenceError(function () {
  let value = 1;
  {
    let value = value;
  }
})) {
  throw "block self initializer";
}

if (!catchesReferenceError(function () {
  switch (1) {
    case 0:
      let value = 1;
      break;
    case value:
  }
})) {
  throw "later switch selector";
}

if ((function () {
  {
    {
      let value = 2;
      {
        let value = 3;
      }
      return value;
    }
    let value = 1;
  }
})() !== 2) {
  throw "initialized nested shadow";
}

true;
