let descriptor = Object.getOwnPropertyDescriptor(Iterator, "zip");
if (typeof Iterator.zip !== "function") throw "zip type";
if (Iterator.zip.name !== "zip") throw "zip name";
if (Iterator.zip.length !== 1) throw "zip length";
if (descriptor.value !== Iterator.zip) throw "zip descriptor value";
if (!descriptor.writable || descriptor.enumerable || !descriptor.configurable) {
  throw "zip descriptor flags";
}

let first = [1, 2, 3];
let second = ["a", "b"];
let zipped = Iterator.zip([first, second], { mode: "shortest" });
if (!(zipped instanceof Iterator)) throw "zip prototype";
let firstRow = zipped.next();
if (firstRow.done || firstRow.value.length !== 2) throw "first row length";
if (firstRow.value[0] !== 1 || firstRow.value[1] !== "a") throw "first row";
let secondRow = zipped.next();
if (secondRow.done || secondRow.value[0] !== 2 || secondRow.value[1] !== "b") {
  throw "second row";
}
let end = zipped.next();
if (!end.done || end.value !== undefined) throw "zip end";
if (!zipped.next().done) throw "zip stable end";

let defaultMode = Iterator.zip([[7], [8]]).next();
if (defaultMode.done || defaultMode.value[0] !== 7 || defaultMode.value[1] !== 8) {
  throw "default mode";
}
let emptyOptions = Iterator.zip([[9], [10]], {}).next();
if (emptyOptions.done || emptyOptions.value[0] !== 9 || emptyOptions.value[1] !== 10) {
  throw "empty options";
}
if (!Iterator.zip([]).next().done) throw "empty iterables";

let directIterator = {
  next: function() { return { done: false, value: 13 }; }
};
let directRow = Iterator.zip([directIterator]).next();
if (directRow.done || directRow.value[0] !== 13) throw "direct iterator source";

let acquisitionCloseOrder = 0;
let firstAcquisitionClose = 0;
let secondAcquisitionClose = 0;
let inputAcquisitionClose = 0;
let firstAcquiredIterator = {};
firstAcquiredIterator.next = function() { return { done: false, value: 1 }; };
Object.defineProperty(firstAcquiredIterator, "return", {
  get: function() {
    acquisitionCloseOrder += 1;
    firstAcquisitionClose = acquisitionCloseOrder;
    throw "first acquisition close";
  }
});
let secondAcquiredIterator = {};
secondAcquiredIterator.next = function() { return { done: false, value: 2 }; };
secondAcquiredIterator.return = new Proxy(function() {
  acquisitionCloseOrder += 1;
  secondAcquisitionClose = acquisitionCloseOrder;
  throw "second acquisition close";
}, {});
let badAcquisitionSource = {};
badAcquisitionSource[Symbol.iterator] = function() { throw "source acquisition"; };
let inputIndex = 0;
let acquisitionInput = {};
acquisitionInput.next = function() {
  if (inputIndex === 0) {
    inputIndex += 1;
    return { done: false, value: firstAcquiredIterator };
  }
  if (inputIndex === 1) {
    inputIndex += 1;
    return { done: false, value: secondAcquiredIterator };
  }
  if (inputIndex === 2) {
    inputIndex += 1;
    return { done: false, value: badAcquisitionSource };
  }
  return { done: true };
};
acquisitionInput.return = function() {
  acquisitionCloseOrder += 1;
  inputAcquisitionClose = acquisitionCloseOrder;
  throw "input acquisition close";
};
acquisitionInput[Symbol.iterator] = function() { return acquisitionInput; };
try {
  Iterator.zip(acquisitionInput);
  throw "missing acquisition error";
} catch (error) {
  if (error !== "source acquisition") throw error;
}
if (secondAcquisitionClose !== 1 || firstAcquisitionClose !== 2 || inputAcquisitionClose !== 3) {
  throw "acquisition cleanup";
}

let pulls = 0;
let closes = 0;
let cachedIterator = {};
cachedIterator.next = function() {
  pulls += 1;
  if (pulls === 1) return { done: false, value: 11 };
  return { done: true };
};
cachedIterator.return = function() {
  closes += 1;
  return {};
};
let cachedSource = {};
cachedSource[Symbol.iterator] = function() { return cachedIterator; };
let cachedZip = Iterator.zip([cachedSource, [12, 13]]);
if (pulls !== 0) throw "eager stepping";
cachedIterator.next = function() { throw "uncached next"; };
let cachedRow = cachedZip.next();
if (cachedRow.done || cachedRow.value[0] !== 11 || cachedRow.value[1] !== 12) {
  throw "cached next";
}
cachedZip.return();
if (closes !== 1) throw "return close";
cachedZip.return();
if (closes !== 1) throw "return close once";

function assertTypeError(callback, label) {
  try {
    callback();
  } catch (error) {
    if (error instanceof TypeError) return;
    throw label + " wrong error";
  }
  throw label + " missing error";
}

let badResultSource = {};
badResultSource[Symbol.iterator] = function() {
  return { next: function() { return 1; } };
};
let badResultZip = Iterator.zip([badResultSource]);
assertTypeError(function() { badResultZip.next(); }, "bad next result");

let reentrantZip;
let reentrantSource = {};
reentrantSource[Symbol.iterator] = function() {
  return {
    next: function() {
      reentrantZip.next();
      return { done: true };
    }
  };
};
reentrantZip = Iterator.zip([reentrantSource]);
assertTypeError(function() { reentrantZip.next(); }, "reentrant next");

let delayedBadNextIterator = {};
delayedBadNextIterator.next = 1;
let delayedBadNextZip = Iterator.zip([delayedBadNextIterator]);
assertTypeError(function() { delayedBadNextZip.next(); }, "deferred bad next");
if (!delayedBadNextZip.next().done || !delayedBadNextZip.return().done) {
  throw "deferred bad next state";
}

let proxyResultIterator = {};
proxyResultIterator.next = function() {
  return new Proxy({}, {
    get: function(_target, key) {
      if (key === "done") return false;
      if (key === "value") return 21;
      return undefined;
    }
  });
};
let proxyResultRow = Iterator.zip([proxyResultIterator]).next();
if (proxyResultRow.done || proxyResultRow.value[0] !== 21) {
  throw "proxy next result";
}

let callableProxyCloseCount = 0;
let callableProxyIterator = {};
callableProxyIterator.next = new Proxy(function() {
  return { done: false, value: 22 };
}, {});
callableProxyIterator.return = new Proxy(function() {
  callableProxyCloseCount += 1;
  return {};
}, {});
let callableProxyZip = Iterator.zip([callableProxyIterator]);
let callableProxyRow = callableProxyZip.next();
if (callableProxyRow.done || callableProxyRow.value[0] !== 22) {
  throw "callable proxy next";
}
callableProxyZip.return();
if (callableProxyCloseCount !== 1) throw "callable proxy return";

let stepCloseOrder = "";
let stepFailingCloseCount = 0;
let stepFirstIterator = {};
stepFirstIterator.next = function() { return { done: false, value: 1 }; };
stepFirstIterator.return = function() {
  stepCloseOrder += "A";
  return {};
};
let stepSecondIterator = {};
stepSecondIterator.next = function() { return { done: false, value: 2 }; };
stepSecondIterator.return = function() {
  stepCloseOrder += "B";
  return {};
};
let stepFailingIterator = {};
stepFailingIterator.next = function() { throw "step abrupt"; };
stepFailingIterator.return = function() {
  stepFailingCloseCount += 1;
  return {};
};
let stepLastIterator = {};
stepLastIterator.next = function() { return { done: false, value: 4 }; };
stepLastIterator.return = new Proxy(function() {
  stepCloseOrder += "D";
  return {};
}, {});
let stepAbruptZip = Iterator.zip([
  stepFirstIterator,
  stepSecondIterator,
  stepFailingIterator,
  stepLastIterator
]);
try {
  stepAbruptZip.next();
  throw "missing step abrupt";
} catch (error) {
  if (error !== "step abrupt") throw error;
}
if (stepCloseOrder !== "DBA" || stepFailingCloseCount !== 0) {
  throw "step abrupt cleanup";
}
if (!stepAbruptZip.next().done || !stepAbruptZip.return().done) {
  throw "step abrupt state";
}

let shortestCloseOrder = "";
let shortestFirstIterator = {};
shortestFirstIterator.next = function() { return { done: false, value: 1 }; };
shortestFirstIterator.return = function() {
  shortestCloseOrder += "A";
  return {};
};
let shortestDoneIterator = {};
shortestDoneIterator.next = function() { return { done: true }; };
let shortestLastIterator = {};
shortestLastIterator.next = function() { return { done: false, value: 3 }; };
shortestLastIterator.return = function() {
  shortestCloseOrder += "C";
  throw "shortest close C";
};
let shortestCloseZip = Iterator.zip([
  shortestFirstIterator,
  shortestDoneIterator,
  shortestLastIterator
]);
try {
  shortestCloseZip.next();
  throw "missing shortest close";
} catch (error) {
  if (error !== "shortest close C") throw error;
}
if (shortestCloseOrder !== "CA") throw "shortest cleanup order";
if (!shortestCloseZip.next().done || !shortestCloseZip.return().done) {
  throw "shortest cleanup state";
}

let returnCloseOrder = "";
function returnCloseIterator(label, closeError) {
  return {
    next: function() { return { done: false, value: label }; },
    return: function() {
      returnCloseOrder += label;
      if (closeError) throw closeError;
      return {};
    }
  };
}
let explicitReturnZip = Iterator.zip([
  returnCloseIterator("A"),
  returnCloseIterator("B", "return close B"),
  returnCloseIterator("C", "return close C")
]);
try {
  explicitReturnZip.return();
  throw "missing return close";
} catch (error) {
  if (error !== "return close C") throw error;
}
if (returnCloseOrder !== "CBA") throw "return cleanup order";
if (!explicitReturnZip.next().done || !explicitReturnZip.return().done) {
  throw "return cleanup state";
}

let suspendedStartZip;
let suspendedStartNextDone = false;
let suspendedStartReturnDone = false;
let suspendedStartIterator = {};
suspendedStartIterator.next = function() { return { done: false, value: 1 }; };
suspendedStartIterator.return = function() {
  suspendedStartNextDone = suspendedStartZip.next().done;
  suspendedStartReturnDone = suspendedStartZip.return().done;
  return {};
};
suspendedStartZip = Iterator.zip([suspendedStartIterator]);
if (!suspendedStartZip.return().done
  || !suspendedStartNextDone
  || !suspendedStartReturnDone) {
  throw "suspended start reentry";
}

let suspendedYieldZip;
let suspendedYieldNextThrew = false;
let suspendedYieldReturnThrew = false;
let suspendedYieldIterator = {};
suspendedYieldIterator.next = function() { return { done: false, value: 1 }; };
suspendedYieldIterator.return = function() {
  try {
    suspendedYieldZip.next();
  } catch (error) {
    suspendedYieldNextThrew = error instanceof TypeError;
  }
  try {
    suspendedYieldZip.return();
  } catch (error) {
    suspendedYieldReturnThrew = error instanceof TypeError;
  }
  return {};
};
suspendedYieldZip = Iterator.zip([suspendedYieldIterator]);
if (suspendedYieldZip.next().done) throw "suspended yield setup";
if (!suspendedYieldZip.return().done
  || !suspendedYieldNextThrew
  || !suspendedYieldReturnThrew) {
  throw "suspended yield reentry";
}

let helperProbe = Iterator.zip([[1]]);
function hasOwnKey(object, key) {
  let names = Object.getOwnPropertyNames(object);
  for (let index = 0; index < names.length; index += 1) {
    if (names[index] === key) return true;
  }
  return false;
}
for (let stateKey of [
  "$IteratorZipIterators",
  "$IteratorZipNextMethods",
  "$IteratorZipOpen",
  "$IteratorZipMode",
  "$IteratorZipPadding",
  "$IteratorZipDone",
  "$IteratorZipExecuting",
  "$IteratorZipStarted"
]) {
  if (hasOwnKey(helperProbe, stateKey)) throw "visible zip state key";
}

let forgedZipHelper = {
  $IteratorZipDone: false,
  $IteratorZipExecuting: false,
  $IteratorZipStarted: false,
  $IteratorZipIterators: [],
  $IteratorZipNextMethods: [],
  $IteratorZipOpen: [],
  $IteratorZipMode: 0,
  $IteratorZipPadding: []
};
assertTypeError(function() { helperProbe.next.call(forgedZipHelper); }, "forged zip next");
assertTypeError(function() { helperProbe.return.call(forgedZipHelper); }, "forged zip return");

function forgedZipFunction() {}
forgedZipFunction.prototype = Symbol();
forgedZipFunction.$IteratorZipDone = false;
forgedZipFunction.$IteratorZipExecuting = false;
forgedZipFunction.$IteratorZipStarted = false;
forgedZipFunction.$IteratorZipIterators = [];
forgedZipFunction.$IteratorZipNextMethods = [];
forgedZipFunction.$IteratorZipOpen = [];
forgedZipFunction.$IteratorZipMode = 0;
forgedZipFunction.$IteratorZipPadding = [];
assertTypeError(function() { helperProbe.next.call(forgedZipFunction); }, "function zip next");
assertTypeError(function() { helperProbe.return.call(forgedZipFunction); }, "function zip return");

let nullReturnIterator = {};
nullReturnIterator.next = function() { return { done: false, value: 1 }; };
nullReturnIterator.return = null;
let nullReturnZip = Iterator.zip([nullReturnIterator]);
if (nullReturnZip.next().done || !nullReturnZip.return().done) {
  throw "null return close";
}

let hostilePulls = 0;
let hostileCloses = 0;
let hostileIterator = {};
hostileIterator.next = function() {
  hostilePulls += 1;
  return hostilePulls === 1 ? { done: false, value: 91 } : { done: true };
};
hostileIterator.return = function() {
  hostileCloses += 1;
  return {};
};
let hostileZip = Iterator.zip([hostileIterator]);
hostileZip.$IteratorZipIterators = [];
delete hostileZip.$IteratorZipNextMethods;
Object.defineProperty(hostileZip, "$IteratorZipOpen", { value: [] });
Object.defineProperty(hostileZip, "$IteratorZipMode", { value: 0 });
Object.defineProperty(hostileZip, "$IteratorZipPadding", { value: [] });
Object.defineProperty(hostileZip, "$IteratorZipDone", { value: true });
Object.defineProperty(hostileZip, "$IteratorZipExecuting", { value: true });
Object.defineProperty(hostileZip, "$IteratorZipStarted", { value: true });
hostileIterator.next = function() { throw "uncached hostile next"; };
let hostileRow = hostileZip.next();
if (hostileRow.done || hostileRow.value[0] !== 91) throw "hidden cached next";
if (!hostileZip.return().done || hostileCloses !== 1) throw "hidden close";
hostileZip.$IteratorZipDone = false;
if (!hostileZip.next().done) throw "hidden completion";

let hiddenReentrantZip;
let hiddenReentrantSource = {};
hiddenReentrantSource.next = function() {
  hiddenReentrantZip.next();
  return { done: true };
};
hiddenReentrantZip = Iterator.zip([hiddenReentrantSource]);
hiddenReentrantZip.$IteratorZipExecuting = false;
delete hiddenReentrantZip.$IteratorZipDone;
Object.defineProperty(hiddenReentrantZip, "$IteratorZipStarted", { value: false });
assertTypeError(function() { hiddenReentrantZip.next(); }, "hidden reentrant next");

let nonExtensibleZip = Iterator.zip([[31, 32]]);
Object.preventExtensions(nonExtensibleZip);
let nonExtensibleRow = nonExtensibleZip.next();
if (nonExtensibleRow.done || nonExtensibleRow.value[0] !== 31
  || !nonExtensibleZip.return().done) {
  throw "nonextensible hidden state";
}

let frozenZip = Iterator.zip([[41, 42]]);
Object.freeze(frozenZip);
let frozenRow = frozenZip.next();
if (frozenRow.done || frozenRow.value[0] !== 41 || !frozenZip.return().done) {
  throw "frozen hidden state";
}

let corruptedZipHelper = Iterator.zip([[51]]);
Object.defineProperty(corruptedZipHelper, "$IteratorZipIterators", { value: 1 });
let corruptedZipRow = corruptedZipHelper.next();
if (corruptedZipRow.done || corruptedZipRow.value[0] !== 51) {
  throw "inaccessible corrupted zip state";
}

true;
