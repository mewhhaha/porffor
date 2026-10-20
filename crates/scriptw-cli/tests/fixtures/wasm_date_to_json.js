function expectError(errorConstructor, callback) {
  let thrown = false;
  try {
    callback();
  } catch (error) {
    if (!(error instanceof errorConstructor)) throw error;
    thrown = true;
  }
  if (!thrown) throw errorConstructor.name;
}

if (new Date(0).toJSON() !== "1970-01-01T00:00:00.000Z") throw "date";
if (new Date(NaN).toJSON() !== null) throw "invalid date";

const effects = [];
let callReceiver;
let callArgumentCount;
const expectedResult = {};
const receiver = {
  valueOf: function() {
    effects.push("valueOf");
    return 1;
  },
  get toISOString() {
    effects.push("get");
    return function() {
      effects.push("call");
      callReceiver = this;
      callArgumentCount = arguments.length;
      return expectedResult;
    };
  },
};
if (Date.prototype.toJSON.call(receiver, "ignored") !== expectedResult) throw "result";
if (effects.join(",") !== "valueOf,get,call") throw "order";
if (callReceiver !== receiver || callArgumentCount !== 0) throw "call";

const exoticReceiver = {
  toISOString: function() {
    return "exotic";
  },
};
let primitiveHint;
exoticReceiver[Symbol.toPrimitive] = function(hint) {
  primitiveHint = hint;
  return 1;
};
if (Date.prototype.toJSON.call(exoticReceiver) !== "exotic") throw "exotic";
if (primitiveHint !== "number") throw "hint";

const nonFiniteReceiver = {
  valueOf: function() {
    return NaN;
  },
  get toISOString() {
    throw "non-finite lookup";
  },
};
if (Date.prototype.toJSON.call(nonFiniteReceiver) !== null) throw "non-finite";

Number.prototype.toISOString = function() {
  return this.valueOf() + 1;
};
if (Date.prototype.toJSON.call(10) !== 11) throw "boxed number";

expectError(TypeError, function() {
  Date.prototype.toJSON.call(null);
});
expectError(TypeError, function() {
  Date.prototype.toJSON.call(undefined);
});
expectError(TypeError, function() {
  Date.prototype.toJSON.call({
    valueOf: function() {
      return 1;
    },
    toISOString: 0,
  });
});

const abrupt = {};
try {
  Date.prototype.toJSON.call({
    get valueOf() {
      throw abrupt;
    },
  });
  throw "missing primitive throw";
} catch (error) {
  if (error !== abrupt) throw error;
}
try {
  Date.prototype.toJSON.call({
    valueOf: function() {
      return 1;
    },
    get toISOString() {
      throw abrupt;
    },
  });
  throw "missing get throw";
} catch (error) {
  if (error !== abrupt) throw error;
}
try {
  Date.prototype.toJSON.call({
    valueOf: function() {
      return 1;
    },
    toISOString: function() {
      throw abrupt;
    },
  });
  throw "missing call throw";
} catch (error) {
  if (error !== abrupt) throw error;
}

if (Date.prototype.toJSON.length !== 1) throw "length";

262;
