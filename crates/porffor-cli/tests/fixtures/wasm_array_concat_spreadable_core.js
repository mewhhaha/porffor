function sameArray(actual, expected) {
  if (actual.length !== expected.length) return false;
  for (let i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) return false;
  }
  return true;
}

let spreadableObject = {
  length: 3,
  0: "a",
  2: "c"
};
spreadableObject[Symbol.isConcatSpreadable] = true;

let falseyArray = [1, 2];
falseyArray[Symbol.isConcatSpreadable] = false;

let fallbackArray = [3, 4];
fallbackArray[Symbol.isConcatSpreadable] = undefined;

let truthyStringObject = {
  length: 2,
  0: "x",
  1: "y"
};
truthyStringObject[Symbol.isConcatSpreadable] = "yes";

function spreadableFunction(a) {
  return a;
}
spreadableFunction[0] = "fn";
spreadableFunction[Symbol.isConcatSpreadable] = true;

let mixed = [0].concat(spreadableObject, falseyArray, fallbackArray, 5, truthyStringObject, spreadableFunction);
let receiverObject = { length: 1, 0: "receiver" };
let nonSpreadReceiverResult = Array.prototype.concat.call(receiverObject, "tail");

sameArray(mixed, [0, "a", undefined, "c", falseyArray, 3, 4, 5, "x", "y", "fn"])
  && Object.prototype.hasOwnProperty.call(mixed, "2") === false
  && mixed[4] === falseyArray
  && sameArray(nonSpreadReceiverResult, [receiverObject, "tail"]);
