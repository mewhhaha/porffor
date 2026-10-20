function retainedCellsAcrossRepeatedSites() {
  let first;
  let second;

  for (var seed = 1; seed <= 2; seed++) {
    let readTried;
    let readError;
    let readHandled;
    let readFinalized;
    let readSwitched;

    try {
      let tried = seed;
      readTried = () => tried;
      throw seed + 10;
    } catch (error) {
      let handled = error + 10;
      readError = () => error;
      readHandled = () => handled;
    } finally {
      let finalized = seed + 30;
      readFinalized = () => finalized;
    }

    switch (0) {
      case 0:
        let switched = seed + 40;
        readSwitched = () => switched;
        break;
    }

    let cells = [readTried, readError, readHandled, readFinalized, readSwitched];
    if (seed === 1) first = cells;
    else second = cells;
  }

  return [first, second];
}

function switchDiscriminantPrecedesEnvironment() {
  let steps = 0;
  let value = 1;
  try {
    switch ((steps = 1, value)) {
      case (steps = steps * 10 + 2, value):
        let value = 2;
        return false;
    }
  } catch (error) {
    return error instanceof ReferenceError && steps === 12;
  }
  return false;
}

function switchCrossCaseTdz() {
  try {
    switch (0) {
      case 0:
        value;
        break;
      default:
        let value = 1;
    }
  } catch (error) {
    return error instanceof ReferenceError;
  }
  return false;
}

function switchNestedShadowingHidesAnOuterTdzBinding() {
  switch (0) {
    case 0: {
      let value = 5;
      return value;
    }
    default:
      let value;
  }
}

function switchInitializesFunctionsBeforeSelectorsAndBodies() {
  "use strict";
  switch (0) {
    case 0:
      return read();
    default:
      function read() {
        return 7;
      }
  }
}

function switchAbruptExitRunsFinally() {
  let sequence = "";
  let read;
  try {
    try {
      switch (0) {
        case 0:
          let value = 5;
          read = () => value;
          sequence += "S";
          throw read();
      }
    } finally {
      sequence += "F";
    }
  } catch (error) {
    sequence += error;
  }
  return sequence === "SF5" && read() === 5;
}

function nestedParentLinkedHops() {
  try {
    throw 2;
  } catch (error) {
    let outer = 3;
    switch (0) {
      case 0:
        let inner = 4;
        return () => error * 100 + outer * 10 + inner;
    }
  }
}

function optionalCatchBindingRetainsClosures() {
  let first;
  let second;

  for (var seed = 1; seed <= 2; seed++) {
    let read;
    try {
      throw seed;
    } catch {
      let value = seed + 10;
      read = () => value;
    }

    if (seed === 1) first = read;
    else second = read;
  }

  return first() === 11 && second() === 12;
}

function classMethodCapturesSwitchEnvironment() {
  let read;
  switch (0) {
    case 0:
      let value = 7;
      class CapturingClass {
        read() { return value; }
      }
      read = new CapturingClass().read;
      break;
  }
  return read() === 7;
}

function switchSelectorCallRunsFinallyBeforeCatch() {
  let marker = 0;
  let caught;

  function throwFromSelector() {
    marker = 1;
    throw 7;
  }

  try {
    try {
      switch (0) {
        case throwFromSelector():
          marker = 9;
          break;
        default:
          let selectorScoped = 0;
      }
      marker = 8;
    } finally {
      marker = marker * 10 + 2;
    }
  } catch (error) {
    caught = error;
    marker = marker * 10 + 3;
  }

  return caught === 7 && marker === 123;
}

let retained = retainedCellsAcrossRepeatedSites();
let first = retained[0];
let second = retained[1];

first[0]() === 1
  && second[0]() === 2
  && first[1]() === 11
  && second[1]() === 12
  && first[2]() === 21
  && second[2]() === 22
  && first[3]() === 31
  && second[3]() === 32
  && first[4]() === 41
  && second[4]() === 42
  && switchDiscriminantPrecedesEnvironment()
  && switchCrossCaseTdz()
  && switchNestedShadowingHidesAnOuterTdzBinding() === 5
  && switchInitializesFunctionsBeforeSelectorsAndBodies() === 7
  && switchAbruptExitRunsFinally()
  && nestedParentLinkedHops()() === 234
  && optionalCatchBindingRetainsClosures()
  && classMethodCapturesSwitchEnvironment()
  && switchSelectorCallRunsFinallyBeforeCatch();
