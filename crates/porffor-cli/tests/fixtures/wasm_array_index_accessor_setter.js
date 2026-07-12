let denseSeen = -1;
let denseReceiver = null;
let sparseSeen = -1;
let sparseReceiver = null;
let array = [];

Object.defineProperty(array, "0", {
  get: function () { return 11; },
  set: function (value) {
    denseSeen = value;
    denseReceiver = this;
  },
  configurable: true
});

let sparseSetter = new Proxy(function (value) {
  sparseSeen = value;
  sparseReceiver = this;
}, {});
Object.defineProperty(array, "10000", {
  get: function () { return 22; },
  set: sparseSetter,
  configurable: true
});

array[0] = 7;
array[10000] = 9;

Object.defineProperty(array, "1", { get: function () { return 33; } });
array[1] = 12;

Object.defineProperty(array, "2", {
  set: function () { throw 44; }
});
let setterAbrupt = false;
try {
  array[2] = 14;
} catch (error) {
  setterAbrupt = error === 44;
}

let strictAbsentSetterThrows = false;
try {
  (function () {
    "use strict";
    array[1] = 13;
  })();
} catch (error) {
  strictAbsentSetterThrows = error instanceof TypeError;
}

let denseGrowthSeen = -1;
let denseGrowth = [];
Object.defineProperty(denseGrowth, "0", {
  get: function () { return 55; },
  set: function (value) { denseGrowthSeen = value; },
  configurable: true
});
for (let i = 1; i < 40; i += 1) {
  denseGrowth[i] = i;
}
denseGrowth[0] = 71;

let sparseGrowthSeen = -1;
let sparseGrowth = [];
Object.defineProperty(sparseGrowth, "10000", {
  get: function () { return 66; },
  set: function (value) { sparseGrowthSeen = value; },
  configurable: true
});
for (let i = 0; i < 12; i += 1) {
  Object.defineProperty(sparseGrowth, String(20000 + i * 1000), {
    value: i,
    configurable: true
  });
}
sparseGrowth[10000] = 81;

denseSeen === 7
  && denseReceiver === array
  && sparseSeen === 9
  && sparseReceiver === array
  && array[0] === 11
  && array[10000] === 22
  && array[1] === 33
  && setterAbrupt
  && strictAbsentSetterThrows
  && denseGrowthSeen === 71
  && denseGrowth[0] === 55
  && sparseGrowthSeen === 81
  && sparseGrowth[10000] === 66;
