let log = "";
let spreadableReads = 0;

let receiver = [1];
receiver.constructor = {
  get [Symbol.species]() {
    log = log + "species,";
    return undefined;
  }
};

let arg = { length: 1, 0: 2 };
Object.defineProperty(arg, Symbol.isConcatSpreadable, {
  get() {
    log = log + "arg-spread,";
    spreadableReads = spreadableReads + 1;
    return true;
  }
});

let orderResult = receiver.concat(arg);

let abrupt = { length: 1, 0: 3 };
Object.defineProperty(abrupt, Symbol.isConcatSpreadable, {
  get() {
    throw "spreadable boom";
  }
});

let propagated = false;
try {
  [0].concat(abrupt);
} catch (err) {
  propagated = err === "spreadable boom";
}

orderResult.length === 2
  && orderResult[0] === 1
  && orderResult[1] === 2
  && log === "species,arg-spread,"
  && spreadableReads === 1
  && propagated;
