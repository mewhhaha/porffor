function loopClosures() {
  let closures = [];
  for (var index = 0; index < 3; index++) {
    let value = index;
    closures.push(function () { return value; });
  }
  return closures[0]() * 100 + closures[1]() * 10 + closures[2]();
}

function whileClosures() {
  let closures = [];
  let index = 0;
  while (index < 3) {
    let value = index;
    closures.push(function () { return value; });
    index++;
  }
  return closures[0]() * 100 + closures[1]() * 10 + closures[2]();
}

function sharedEntryMutation() {
  let closures = [];
  for (var index = 0; index < 2; index++) {
    let value = index;
    closures.push(function () {
      value = value + 1;
      return value;
    });
    closures.push(function () { return value; });
  }
  let firstIncrement = closures[0]();
  let firstRead = closures[1]();
  let secondIncrement = closures[2]();
  let secondRead = closures[3]();
  let firstIncrementAgain = closures[0]();
  return firstIncrement === 1
    && firstRead === 1
    && secondIncrement === 2
    && secondRead === 2
    && firstIncrementAgain === 2;
}

function nestedBlocks() {
  let read;
  {
    let outer = 4;
    {
      let inner = 5;
      read = function () { return outer * 10 + inner; };
    }
  }
  return read();
}

function nestedActivation(argument) {
  {
    let outer = 2;
    {
      let inner = 3;
      return function () { return argument + outer + inner; };
    }
  }
}

function classBlockCapture() {
  {
    let value = 6;
    class CapturingClass {
      read() { return value; }
    }
    return new CapturingClass().read();
  }
}

function lexicalTdzAndClosureReads() {
  let directLetTdz = false;
  try {
    {
      let value = value;
    }
  } catch (error) {
    directLetTdz = error instanceof ReferenceError;
  }

  let directConstTdz = false;
  try {
    {
      const value = value;
    }
  } catch (error) {
    directConstTdz = error instanceof ReferenceError;
  }

  let closureTdz = false;
  let afterInitialization = false;
  {
    let read = () => value;
    try {
      read();
    } catch (error) {
      closureTdz = error instanceof ReferenceError;
    }
    let value = 8;
    afterInitialization = read() === 8;
  }

  return directLetTdz && directConstTdz && closureTdz && afterInitialization;
}

function nestedShadowAliases() {
  {
    let value = 2;
    let readOuter = () => value;
    {
      let value = 3;
      let readInner = () => value;
      return () => readOuter() * 10 + readInner();
    }
  }
}

function sharedArrowBlockCell() {
  let increment;
  let read;
  {
    let value = 4;
    increment = () => {
      value = value + 1;
      return value;
    };
    read = () => value;
  }
  return () => increment() === 5 && read() === 5 && increment() === 6 && read() === 6;
}

function materializedNeighborBlocks() {
  let readBelow;
  {
    let ignoredAbove = 1;
    {
      let captured = 2;
      readBelow = () => captured;
    }
  }

  let readAbove;
  {
    let captured = 3;
    readAbove = () => captured;
    {
      let ignoredBelow = 4;
    }
  }

  return readBelow() === 2 && readAbove() === 3;
}

function bothIfBranches() {
  function readBranch(condition) {
    if (condition) {
      let value = 4;
      return () => value;
    } else {
      let value = 6;
      return () => value;
    }
  }

  return readBranch(true)() === 4 && readBranch(false)() === 6;
}

function conditionalExpressionTdzUnwinds() {
  try {
    {
      let value = true ? value : 1;
    }
  } catch (error) {
    return error instanceof ReferenceError;
  }
  return false;
}

function labelledVarLoopPaths() {
  let takenContinue = 0;
  continueLoop: for (var index = 0; index < 3; index++) {
    {
      let value = index;
      let read = () => value;
      if (index === 1) {
        takenContinue += read();
        continue continueLoop;
      }
      takenContinue += read() * 10;
    }
  }

  let takenBreak = 0;
  breakLoop: for (var index = 0; index < 3; index++) {
    {
      let value = index;
      let read = () => value;
      if (index === 1) {
        takenBreak += read();
        break breakLoop;
      }
      takenBreak += read() * 10;
    }
  }

  let notTakenContinue = 0;
  skippedContinue: for (var index = 0; index < 2; index++) {
    {
      let value = index;
      let read = () => value;
      if (index === -1) {
        continue skippedContinue;
      }
      notTakenContinue += read();
    }
  }

  let notTakenBreak = 0;
  skippedBreak: for (var index = 0; index < 2; index++) {
    {
      let value = index;
      let read = () => value;
      if (index === -1) {
        break skippedBreak;
      }
      notTakenBreak += read();
    }
  }

  return takenContinue === 21
    && takenBreak === 1
    && notTakenContinue === 1
    && notTakenBreak === 1;
}

let scriptBlockRead;
{
  let value = 9;
  scriptBlockRead = () => value;
}

function abruptBlocks() {
  let score = 0;
  outer: for (var index = 0; index < 3; index++) {
    let retained = index;
    {
      let extra = 0;
      let read = function () { return retained + extra; };
      if (index === 0) {
        score += read();
        continue outer;
      }
      score += read();
      break outer;
    }
  }

  try {
    {
      let explicit = 4;
      let read = function () { return explicit; };
      throw read();
    }
  } catch (thrown) {
    score = score * 10 + thrown;
  }

  try {
    {
      let captured = 5;
      let read = function () { return captured; };
      score = score * 10 + read();
      const uninitialized = uninitialized;
    }
  } catch (error) {
    score = score * 10 + 6;
  }

  try {
    try {
      {
        let finalValue = 7;
        let read = function () { return finalValue; };
        score = score * 10 + read();
        return function () { return score; };
      }
    } finally {
      score = score * 10 + 8;
    }
  } finally {
    score = score * 10 + 9;
  }
}

let readFinalScore = abruptBlocks();
let checkSharedArrowBlockCell = sharedArrowBlockCell();
loopClosures() === 12
  && whileClosures() === 12
  && sharedEntryMutation()
  && nestedBlocks() === 45
  && nestedActivation(4)() === 9
  && classBlockCapture() === 6
  && lexicalTdzAndClosureReads()
  && nestedShadowAliases()() === 23
  && checkSharedArrowBlockCell()
  && materializedNeighborBlocks()
  && bothIfBranches()
  && conditionalExpressionTdzUnwinds()
  && scriptBlockRead() === 9
  && labelledVarLoopPaths()
  && readFinalScore() === 1456789;
