function assertTypeError(callback, label) {
  try {
    callback();
  } catch (error) {
    if (error instanceof TypeError) return;
    throw label + " wrong error";
  }
  throw label + " missing error";
}

let longest = Iterator.zip([[1, 2], ["a"]], { mode: "longest" });
let longestFirst = longest.next();
if (longestFirst.done || longestFirst.value[0] !== 1 || longestFirst.value[1] !== "a") {
  throw "longest first row";
}
let longestSecond = longest.next();
if (longestSecond.done || longestSecond.value[0] !== 2 || longestSecond.value[1] !== undefined) {
  throw "longest undefined padding";
}
if (!longest.next().done || !longest.next().done) throw "longest no phantom row";

let customPadding = Iterator.zip([[1, 2], [3]], {
  mode: "longest",
  padding: [0, 9]
});
if (customPadding.next().value[0] !== 1) throw "custom padding first";
let customPaddingSecond = customPadding.next();
if (customPaddingSecond.done
  || customPaddingSecond.value[0] !== 2
  || customPaddingSecond.value[1] !== 9) {
  throw "custom padding second";
}
if (!customPadding.next().done) throw "custom padding end";

let exhaustedPulls = 0;
let exhaustedSource = {
  next: function() {
    exhaustedPulls += 1;
    if (exhaustedPulls === 1) return { done: false, value: 1 };
    if (exhaustedPulls === 2) return { done: true };
    throw "repeated exhausted next";
  }
};
let steadySource = [2, 3];
let exhaustedLongest = Iterator.zip([exhaustedSource, steadySource], { mode: "longest" });
if (exhaustedLongest.next().done || exhaustedLongest.next().done) throw "exhausted longest rows";
if (!exhaustedLongest.next().done || exhaustedPulls !== 2) throw "longest repeated stepping";

let paddingPulls = 0;
let paddingCloses = 0;
let eagerPadding = {
  next: function() {
    paddingPulls += 1;
    return { done: false, value: paddingPulls + 20 };
  },
  return: function() {
    paddingCloses += 1;
    return {};
  }
};
let eagerPaddingSource = {};
eagerPaddingSource[Symbol.iterator] = function() { return eagerPadding; };
let eagerPaddingZip = Iterator.zip([[1], [2]], {
  mode: "longest",
  padding: eagerPaddingSource
});
if (paddingPulls !== 2 || paddingCloses !== 1) throw "eager padding surplus close";
if (eagerPaddingZip.next().done) throw "eager padding row";

let longestReturnOrder = "";
function longestReturnIterator(label) {
  return {
    next: function() { return { done: false, value: label }; },
    return: function() {
      longestReturnOrder += label;
      return {};
    }
  };
}
let longestReturn = Iterator.zip([
  longestReturnIterator("A"),
  longestReturnIterator("B"),
  longestReturnIterator("C")
], { mode: "longest" });
if (longestReturn.next().done || !longestReturn.return().done) throw "longest return";
if (longestReturnOrder !== "CBA") throw "longest return order";

let strictEqual = Iterator.zip([[1], [2]], { mode: "strict" });
let strictEqualRow = strictEqual.next();
if (strictEqualRow.done || strictEqualRow.value[0] !== 1 || strictEqualRow.value[1] !== 2) {
  throw "strict equal row";
}
if (!strictEqual.next().done || !strictEqual.next().done) throw "strict equal end";

let strictFirstDone = { next: function() { return { done: true }; } };
let strictLiveMismatchValueRead = false;
let strictLiveMismatch = {
  next: function() {
    return {
      done: false,
      get value() {
        strictLiveMismatchValueRead = true;
        throw "strict mismatch value";
      }
    };
  },
  return: function() { strictFirstDone.closeOrder += "B"; return {}; }
};
let strictUnprobed = {
  next: function() { strictFirstDone.probedLater = true; return { done: false, value: 3 }; },
  return: function() { strictFirstDone.closeOrder += "C"; return {}; }
};
strictFirstDone.closeOrder = "";
strictFirstDone.probedLater = false;
let strictFirstMismatch = Iterator.zip([
  strictFirstDone,
  strictLiveMismatch,
  strictUnprobed
], { mode: "strict" });
assertTypeError(function() { strictFirstMismatch.next(); }, "strict first mismatch");
if (strictLiveMismatchValueRead || strictFirstDone.probedLater || strictFirstDone.closeOrder !== "CB") {
  throw "strict first mismatch behavior";
}

let strictLaterProbe = false;
let strictLaterCloseOrder = "";
let strictLaterMismatch = Iterator.zip([
  {
    next: function() { return { done: false, value: 1 }; },
    return: function() { strictLaterCloseOrder += "A"; return {}; }
  },
  { next: function() { return { done: true }; } },
  {
    next: function() { strictLaterProbe = true; return { done: false, value: 3 }; },
    return: function() { strictLaterCloseOrder += "C"; return {}; }
  }
], { mode: "strict" });
assertTypeError(function() { strictLaterMismatch.next(); }, "strict later mismatch");
if (strictLaterProbe || strictLaterCloseOrder !== "CA") throw "strict later mismatch behavior";

let strictAbruptClose = 0;
let strictAbrupt = Iterator.zip([
  { next: function() { return { done: true }; } },
  { next: function() { throw "strict abrupt"; } },
  {
    next: function() { return { done: false, value: 3 }; },
    return: function() { strictAbruptClose += 1; throw "strict close"; }
  }
], { mode: "strict" });
try {
  strictAbrupt.next();
  throw "strict abrupt missing";
} catch (error) {
  if (error !== "strict abrupt") throw error;
}
if (strictAbruptClose !== 1) throw "strict abrupt close";

true;
