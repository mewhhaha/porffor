var trace = [];
var coercions = 0;

var baseA = {
  get p() {
    trace.push("getA");
    return 1;
  },
  set p(value) {
    trace.push("setA:" + value + ":" + (this === alien));
  }
};

var baseB = {
  get p() {
    trace.push("getB");
    return -1;
  },
  set p(value) {
    trace.push("setB:" + value);
  }
};

var object = {
  compound(key) {
    return super[key] += (trace.push("rhs"), 2);
  },
  prefix(key) {
    return ++super[key];
  }
};

var key = {
  toString() {
    coercions += 1;
    trace.push("key");
    Object.setPrototypeOf(object, baseB);
    return "p";
  }
};

var alien = { marker: "alien" };
Object.setPrototypeOf(object, baseA);
var compound = object.compound;
var compoundResult = compound.call(alien, key);
var compoundTrace = trace.join(",");

trace = [];
Object.setPrototypeOf(object, baseA);
var prefix = object.prefix;
var prefixResult = prefix.call(alien, key);
var prefixTrace = trace.join(",");

var numericBase = {
  get number() {
    return this.numberState;
  },
  set number(value) {
    this.numberState = value;
  },
  get bigint() {
    return this.bigintState;
  },
  set bigint(value) {
    this.bigintState = value;
  }
};

var numericMethods = {
  numberPostIncrement() {
    return super.number++;
  },
  numberPrefixIncrement() {
    return ++super.number;
  },
  numberPostDecrement() {
    return super.number--;
  },
  numberPrefixDecrement() {
    return --super.number;
  },
  bigintPostIncrement() {
    return super.bigint++;
  },
  bigintPrefixIncrement() {
    return ++super.bigint;
  },
  bigintPostDecrement() {
    return super.bigint--;
  },
  bigintPrefixDecrement() {
    return --super.bigint;
  }
};
Object.setPrototypeOf(numericMethods, numericBase);

var numericReceiver = { numberState: 5, bigintState: 5n };
var numberPostIncrement = numericMethods.numberPostIncrement.call(numericReceiver);
var numberPrefixIncrement = numericMethods.numberPrefixIncrement.call(numericReceiver);
var numberPostDecrement = numericMethods.numberPostDecrement.call(numericReceiver);
var numberPrefixDecrement = numericMethods.numberPrefixDecrement.call(numericReceiver);
var bigintPostIncrement = numericMethods.bigintPostIncrement.call(numericReceiver);
var bigintPrefixIncrement = numericMethods.bigintPrefixIncrement.call(numericReceiver);
var bigintPostDecrement = numericMethods.bigintPostDecrement.call(numericReceiver);
var bigintPrefixDecrement = numericMethods.bigintPrefixDecrement.call(numericReceiver);

var lockedBase = {};
Object.defineProperty(lockedBase, "locked", {
  value: 1,
  writable: false,
  configurable: true
});
var strictFailureMethod = {
  update() {
    "use strict";
    return super.locked++;
  }
};
Object.setPrototypeOf(strictFailureMethod, lockedBase);
var strictFailureReceiver = {};
var strictFailure = false;
var strictFailureResult = "not published";
try {
  strictFailureResult = strictFailureMethod.update.call(strictFailureReceiver);
} catch (error) {
  strictFailure = error instanceof TypeError;
}

var uninitializedTrace = "";
class UninitializedBase {
  constructor() {
    uninitializedTrace += "base";
    throw new Error("base constructor was evaluated");
  }
}

class UninitializedUpdate extends UninitializedBase {
  constructor() {
    super[(uninitializedTrace += "update-key", super())]++;
  }
}

var uninitializedUpdate = false;
try {
  new UninitializedUpdate();
} catch (error) {
  uninitializedUpdate = error instanceof ReferenceError;
}
var uninitializedUpdateTrace = uninitializedTrace;

uninitializedTrace = "";
class UninitializedCompound extends UninitializedBase {
  constructor() {
    super[(uninitializedTrace += "compound-key", super())] +=
      (uninitializedTrace += "rhs", 1);
  }
}

var uninitializedCompound = false;
try {
  new UninitializedCompound();
} catch (error) {
  uninitializedCompound = error instanceof ReferenceError;
}

compoundResult === 3
  && compoundTrace === "key,getA,rhs,setA:3:true"
  && prefixResult === 2
  && prefixTrace === "key,getA,setA:2:true"
  && coercions === 2
  && numberPostIncrement === 5
  && numberPrefixIncrement === 7
  && numberPostDecrement === 7
  && numberPrefixDecrement === 5
  && numericReceiver.numberState === 5
  && bigintPostIncrement === 5n
  && bigintPrefixIncrement === 7n
  && bigintPostDecrement === 7n
  && bigintPrefixDecrement === 5n
  && numericReceiver.bigintState === 5n
  && strictFailure === true
  && strictFailureResult === "not published"
  && !Object.prototype.hasOwnProperty.call(strictFailureReceiver, "locked")
  && uninitializedUpdate === true
  && uninitializedUpdateTrace === ""
  && uninitializedCompound === true
  && uninitializedTrace === "";
