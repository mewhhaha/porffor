let everyConstructorReads = 0;
let everyConstructorReceiver = [1, 2];
Object.defineProperty(everyConstructorReceiver, "constructor", {
  get() {
    everyConstructorReads = everyConstructorReads + 1;
    throw "every must not read constructor";
  }
});
let everyConstructorCalls = 0;
let everyConstructorResult = everyConstructorReceiver.every(function (value) {
  everyConstructorCalls = everyConstructorCalls + 1;
  return value > 0;
});

let everySpeciesReads = 0;
let everySpeciesReceiver = [1, 0, 2];
everySpeciesReceiver.constructor = {
  get [Symbol.species]() {
    everySpeciesReads = everySpeciesReads + 1;
    throw "every must not read Symbol.species";
  }
};
let everySpeciesCalls = 0;
let everySpeciesResult = everySpeciesReceiver.every(function (value) {
  everySpeciesCalls = everySpeciesCalls + 1;
  return value > 0;
});

let someConstructorReads = 0;
let someConstructorReceiver = [0, 1];
Object.defineProperty(someConstructorReceiver, "constructor", {
  get() {
    someConstructorReads = someConstructorReads + 1;
    throw "some must not read constructor";
  }
});
let someConstructorCalls = 0;
let someConstructorResult = someConstructorReceiver.some(function (value) {
  someConstructorCalls = someConstructorCalls + 1;
  return value === 1;
});

let someSpeciesReads = 0;
let someSpeciesReceiver = [0, 2, 3];
someSpeciesReceiver.constructor = {
  get [Symbol.species]() {
    someSpeciesReads = someSpeciesReads + 1;
    throw "some must not read Symbol.species";
  }
};
let someSpeciesCalls = 0;
let someSpeciesResult = someSpeciesReceiver.some(function (value) {
  someSpeciesCalls = someSpeciesCalls + 1;
  return value === 2;
});

everyConstructorResult === true
  && everyConstructorCalls === 2
  && everyConstructorReads === 0
  && everySpeciesResult === false
  && everySpeciesCalls === 2
  && everySpeciesReads === 0
  && someConstructorResult === true
  && someConstructorCalls === 2
  && someConstructorReads === 0
  && someSpeciesResult === true
  && someSpeciesCalls === 2
  && someSpeciesReads === 0;
