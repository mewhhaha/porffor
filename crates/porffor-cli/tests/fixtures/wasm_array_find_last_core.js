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

let findLast = Array.prototype.findLast;
let findLastIndex = Array.prototype.findLastIndex;
let findLastDesc = Object.getOwnPropertyDescriptor(Array.prototype, "findLast");
let findLastIndexDesc = Object.getOwnPropertyDescriptor(Array.prototype, "findLastIndex");

let order = [];
let orderResult = [1, 2, 3].findLast(function (value, index, source) {
  order[order.length] = value;
  return source[index] === value && value < 3;
});

let sparse = ["first", , , undefined];
let sparseCount = 0;
sparse.findLast(function () {
  sparseCount = sparseCount + 1;
  return false;
});

let spliceArray = ["Shoes", "Car", "Bike"];
let spliceCount = 0;
let spliceFirst;
let spliceSecond;
let spliceThird;
let spliceIndexResult = spliceArray.findLastIndex(function (value) {
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
let pushIndexResult = pushArray.findLastIndex(function (value) {
  if (pushCount === 0) {
    pushArray.push("Motorcycle");
    pushArray[0] = "Magic Carpet";
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
let fixedShrinkUndefinedCount = 0;
let fixedShrinkZeroAfterShrink = false;
let fixedShrinkCount = 0;
let fixedShrinkResult = Array.prototype.findLast.call(fixedBytes, function (value) {
  if (fixedShrinkCount >= 2 && value === undefined) {
    fixedShrinkUndefinedCount = fixedShrinkUndefinedCount + 1;
  }
  if (fixedShrinkCount >= 2 && value === 0) {
    fixedShrinkZeroAfterShrink = true;
  }
  fixedShrinkCount = fixedShrinkCount + 1;
  if (fixedShrinkCount === 2) {
    rab.resize(3);
  }
  return false;
});

typeof findLast === "function"
  && typeof findLastIndex === "function"
  && findLast.name === "findLast"
  && findLast.length === 1
  && findLastDesc.value === findLast
  && findLastDesc.writable === true
  && findLastDesc.enumerable === false
  && findLastDesc.configurable === true
  && findLastIndex.name === "findLastIndex"
  && findLastIndex.length === 1
  && findLastIndexDesc.value === findLastIndex
  && findLastIndexDesc.writable === true
  && findLastIndexDesc.enumerable === false
  && findLastIndexDesc.configurable === true
  && orderResult === 2
  && order.length === 2
  && order[0] === 3
  && order[1] === 2
  && [1, 2, 3].findLast(function (value) { return value > 1; }) === 3
  && [1, 2, 3].findLast(function (value) { return value > 3; }) === undefined
  && [1, 2, 3].findLastIndex(function (value) { return value > 1; }) === 2
  && [1, 2, 3].findLastIndex(function (value) { return value > 3; }) === -1
  && [2].findLast(keepThis, { keep: 2 }) === 2
  && [2].findLastIndex(keepThis, { keep: 2 }) === 0
  && sparseCount === 4
  && spliceIndexResult === -1
  && spliceCount === 3
  && spliceFirst === "Bike"
  && spliceSecond === "Bike"
  && spliceThird === "Shoes"
  && pushIndexResult === -1
  && pushCount === 2
  && pushFirst === "Barefoot"
  && pushSecond === "Magic Carpet"
  && fixedShrinkResult === undefined
  && fixedShrinkCount === 4
  && fixedShrinkUndefinedCount === 2
  && fixedShrinkZeroAfterShrink === false
  && throwsTypeError(function () { [1].findLast(); })
  && throwsTypeError(function () { [1].findLast(null); })
  && throwsTypeError(function () { [1].findLastIndex(); })
  && throwsTypeError(function () { [1].findLastIndex(null); });
