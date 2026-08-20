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

let exactObject = { marker: 1 };
let exactSymbol = Symbol("spreadable");
let exactArray = [6];
exactArray[Symbol.isConcatSpreadable] = exactObject;
let objectIdentity = exactArray[Symbol.isConcatSpreadable] === exactObject;
let truthyArrayResult = [].concat(exactArray);

exactArray[Symbol.isConcatSpreadable] = exactSymbol;
let symbolIdentity = exactArray[Symbol.isConcatSpreadable] === exactSymbol;

exactArray[Symbol.isConcatSpreadable] = "yes";
let stringIdentity = exactArray[Symbol.isConcatSpreadable] === "yes"
  && typeof exactArray[Symbol.isConcatSpreadable] === "string";

exactArray[Symbol.isConcatSpreadable] = -0;
let zeroIdentity = Object.is(exactArray[Symbol.isConcatSpreadable], -0)
  && typeof exactArray[Symbol.isConcatSpreadable] === "number";
let falseyArrayResult = [].concat(exactArray);

exactArray[Symbol.isConcatSpreadable] = NaN;
let nanIdentity = Number.isNaN(exactArray[Symbol.isConcatSpreadable]);

exactArray[Symbol.isConcatSpreadable] = undefined;
let undefinedIdentity = exactArray[Symbol.isConcatSpreadable] === undefined;
let fallbackArrayResult = [].concat(exactArray);

let getterArray = [7];
let getterValue = { marker: 2 };
let getterReceiver = false;
Object.defineProperty(getterArray, Symbol.isConcatSpreadable, {
  get() {
    getterReceiver = this === getterArray;
    return getterValue;
  }
});
let getterIdentity = getterArray[Symbol.isConcatSpreadable] === getterValue;
let getterTruthyResult = [].concat(getterArray);

let proxyGetterArray = [8];
let proxyGetterValue = { marker: 3 };
let proxyGetterReceiver = false;
let proxyGetterCalls = 0;
let proxyGetter = new Proxy(function() {}, {
  apply(target, receiver, args) {
    proxyGetterCalls += 1;
    proxyGetterReceiver = receiver === proxyGetterArray && args.length === 0;
    return proxyGetterValue;
  }
});
Object.defineProperty(proxyGetterArray, Symbol.isConcatSpreadable, {
  get: proxyGetter
});
let proxyGetterIdentity = proxyGetterArray[Symbol.isConcatSpreadable] === proxyGetterValue;
let proxyGetterTruthyResult = [].concat(proxyGetterArray);

let throwingGetterArray = [9];
let getterSentinel = { marker: 4 };
Object.defineProperty(throwingGetterArray, Symbol.isConcatSpreadable, {
  get() {
    throw getterSentinel;
  }
});
let getterThrowIdentity = false;
try {
  [].concat(throwingGetterArray);
} catch (error) {
  getterThrowIdentity = error === getterSentinel;
}

sameArray(mixed, [0, "a", undefined, "c", falseyArray, 3, 4, 5, "x", "y", "fn"])
  && Object.prototype.hasOwnProperty.call(mixed, "2") === false
  && mixed[4] === falseyArray
  && sameArray(nonSpreadReceiverResult, [receiverObject, "tail"])
  && objectIdentity
  && symbolIdentity
  && stringIdentity
  && zeroIdentity
  && nanIdentity
  && undefinedIdentity
  && sameArray(truthyArrayResult, [6])
  && falseyArrayResult.length === 1
  && falseyArrayResult[0] === exactArray
  && sameArray(fallbackArrayResult, [6])
  && getterReceiver
  && getterIdentity
  && sameArray(getterTruthyResult, [7])
  && proxyGetterReceiver
  && proxyGetterCalls === 2
  && proxyGetterIdentity
  && sameArray(proxyGetterTruthyResult, [8])
  && getterThrowIdentity;
