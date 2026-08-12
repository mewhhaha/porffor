function throwsTypeError(fn) {
  try {
    fn();
  } catch (err) {
    return err instanceof TypeError;
  }
  return false;
}

function keepThis(value) {
  return this.keep === value;
}

let find = Array.prototype.find;
let findIndex = Array.prototype.findIndex;
let findDesc = Object.getOwnPropertyDescriptor(Array.prototype, "find");
let findIndexDesc = Object.getOwnPropertyDescriptor(Array.prototype, "findIndex");

let sparse = [undefined, , , "foo"];
let sparseCount = 0;
sparse.find(function () {
  sparseCount = sparseCount + 1;
  return false;
});

let argArray = ["Mike", "Rick", "Leo"];
let argOk = true;
let argCount = 0;
argArray.find(function (value, index, source) {
  argOk = argOk
    && source === argArray
    && value === argArray[index]
    && index === argCount;
  argCount = argCount + 1;
  return false;
});

let spliceArray = ["Shoes", "Car", "Bike"];
let spliceCount = 0;
let spliceFirst;
let spliceSecond;
let spliceThird;
let spliceIndexResult = spliceArray.findIndex(function (value) {
  if (spliceCount === 0) {
    spliceArray.splice(1, 1);
  }
  if (spliceCount === 0) spliceFirst = value;
  if (spliceCount === 1) spliceSecond = value;
  if (spliceCount === 2) spliceThird = value;
  spliceCount = spliceCount + 1;
  return false;
});

let pushArray = ["Skateboard", "Barefoot"];
let pushCount = 0;
let pushFirst;
let pushSecond;
let pushIndexResult = pushArray.findIndex(function (value) {
  if (pushCount === 0) {
    pushArray.push("Motorcycle");
    pushArray[1] = "Magic Carpet";
  }
  if (pushCount === 0) pushFirst = value;
  if (pushCount === 1) pushSecond = value;
  pushCount = pushCount + 1;
  return false;
});

let rab = new ArrayBuffer(4, { maxByteLength: 5 });
let fixedBytes = new Uint8Array(rab, 0, 4);
for (let i = 0; i < 4; i++) {
  fixedBytes[i] = i * 2;
}
let fixedShrinkValues = [];
let fixedShrinkUndefinedCount = 0;
let fixedShrinkZeroAfterShrink = false;
let fixedShrinkResult = Array.prototype.find.call(fixedBytes, function (value) {
  if (fixedShrinkValues.length >= 2 && value === undefined) {
    fixedShrinkUndefinedCount = fixedShrinkUndefinedCount + 1;
  }
  if (fixedShrinkValues.length >= 2 && value === 0) {
    fixedShrinkZeroAfterShrink = true;
  }
  fixedShrinkValues[fixedShrinkValues.length] = value;
  if (fixedShrinkValues.length === 2) {
    rab.resize(3);
  }
  return false;
});

typeof find === "function"
  && typeof findIndex === "function"
  && find.name === "find"
  && find.length === 1
  && findDesc.value === find
  && findDesc.writable === true
  && findDesc.enumerable === false
  && findDesc.configurable === true
  && findIndex.name === "findIndex"
  && findIndex.length === 1
  && findIndexDesc.value === findIndex
  && findIndexDesc.writable === true
  && findIndexDesc.enumerable === false
  && findIndexDesc.configurable === true
  && [1, 2, 3].find(function (value) { return value > 1; }) === 2
  && [1, 2, 3].find(function (value) { return value > 3; }) === undefined
  && [1, 2, 3].findIndex(function (value) { return value > 1; }) === 1
  && [1, 2, 3].findIndex(function (value) { return value > 3; }) === -1
  && [2].find(keepThis, { keep: 2 }) === 2
  && [2].findIndex(keepThis, { keep: 2 }) === 0
  && sparseCount === 4
  && argOk === true
  && argCount === 3
  && spliceIndexResult === -1
  && spliceCount === 3
  && spliceFirst === "Shoes"
  && spliceSecond === "Bike"
  && spliceThird === undefined
  && pushIndexResult === -1
  && pushCount === 2
  && pushFirst === "Skateboard"
  && pushSecond === "Magic Carpet"
  && fixedShrinkResult === undefined
  && fixedShrinkValues.length === 4
  && fixedShrinkValues[0] === 0
  && fixedShrinkValues[1] === 2
  && fixedShrinkUndefinedCount === 2
  && fixedShrinkZeroAfterShrink === false
  && throwsTypeError(function () { [1].find(); })
  && throwsTypeError(function () { [1].find(null); })
  && throwsTypeError(function () { [1].findIndex(); })
  && throwsTypeError(function () { [1].findIndex(null); });
