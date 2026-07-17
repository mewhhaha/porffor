var ok = true;

function hasFourArguments() {
  return arguments.length === 4
    && arguments[0] === 42
    && arguments[1] === 1
    && arguments[2] === 2
    && arguments[3] === 3;
}

var trailingValues = [2, 3];
ok = ok && hasFourArguments(42, ...[1], ...trailingValues,);

var indirectCall = hasFourArguments;
ok = ok && indirectCall(42, ...[1], ...trailingValues);

var methodHolder = { method: hasFourArguments };
ok = ok && methodHolder.method(42, ...[1], ...trailingValues);

class SpreadMethods {
  method() { return hasFourArguments(...arguments); }
  static method() { return hasFourArguments(...arguments); }
}
ok = ok && new SpreadMethods().method(42, ...[1], ...trailingValues,);
ok = ok && SpreadMethods.method(42, ...[1], ...trailingValues,);

var order = [];
var orderedStep = 0;
var orderedIterator = {
  next: function() {
    orderedStep++;
    if (orderedStep === 1) {
      order.push(6);
      return {
        get done() {
          order.push(7);
          return false;
        },
        get value() {
          order.push(8);
          return 1;
        }
      };
    }
    order.push(9);
    return {
      get done() {
        order.push(10);
        return true;
      },
      get value() {
        ok = false;
        return 0;
      }
    };
  }
};
var orderedIterable = {};
Object.defineProperty(orderedIterable, Symbol.iterator, {
  get: function() {
    order.push(4);
    return function() {
      order.push(5);
      return orderedIterator;
    };
  }
});
function orderedCallee() {
  order.push(12);
  return arguments.length === 3
    && arguments[0] === 0
    && arguments[1] === 1
    && arguments[2] === 2;
}
function getOrderedCallee() {
  order.push(1);
  return orderedCallee;
}
function beforeSpread() {
  order.push(2);
  return 0;
}
function getOrderedIterable() {
  order.push(3);
  return orderedIterable;
}
function afterSpread() {
  order.push(11);
  return 2;
}
ok = ok && getOrderedCallee()(beforeSpread(), ...getOrderedIterable(), afterSpread());
ok = ok && order.length === 12;
for (var orderIndex = 0; orderIndex < order.length; orderIndex++) {
  ok = ok && order[orderIndex] === orderIndex + 1;
}

var overrideCalls = 0;
var overriddenArray = [1, 2];
overriddenArray[Symbol.iterator] = function() {
  overrideCalls++;
  var done = false;
  return {
    next: function() {
      if (done) return { done: true };
      done = true;
      return { done: false, value: 9 };
    }
  };
};
function receivesOverride(value) {
  return arguments.length === 1 && value === 9;
}
ok = ok && receivesOverride(...overriddenArray) && overrideCalls === 1;

var abruptError = {};
var abruptCalls = 0;
var abruptCloses = 0;
var abruptIterable = {};
abruptIterable[Symbol.iterator] = function() {
  return {
    next: function() { throw abruptError; },
    return: function() {
      abruptCloses++;
      return {};
    }
  };
};
function mustNotRun() { abruptCalls++; }
var caughtAbrupt;
try {
  mustNotRun(...abruptIterable);
} catch (error) {
  caughtAbrupt = error;
}
ok = ok && caughtAbrupt === abruptError && abruptCalls === 0 && abruptCloses === 0;

var valueError = {};
var valueCloses = 0;
var valueIterable = {};
valueIterable[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        done: false,
        get value() { throw valueError; }
      };
    },
    return: function() {
      valueCloses++;
      return {};
    }
  };
};
var caughtValue;
try {
  mustNotRun(...valueIterable);
} catch (error) {
  caughtValue = error;
}
ok = ok && caughtValue === valueError && abruptCalls === 0 && valueCloses === 0;

class SpreadBase {
  constructor() {
    this.correct = arguments.length === 3
      && arguments[0] === 1
      && arguments[1] === 2
      && arguments[2] === 3;
  }
}
var constructed = new SpreadBase(1, ...[2, 3]);
ok = ok && constructed.correct;

class SpreadDerived extends SpreadBase {
  constructor(values) {
    super(1, ...values);
  }
}
var derived = new SpreadDerived([2, 3]);
ok = ok && derived.correct;

function receivesString() {
  return arguments.length === 2 && arguments[0] === "a" && arguments[1] === "b";
}
ok = ok && receivesString(..."ab");

ok;
