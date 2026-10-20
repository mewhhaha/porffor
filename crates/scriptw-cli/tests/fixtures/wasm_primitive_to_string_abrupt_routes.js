var trace = "";
var token = Symbol("primitive-to-string");

function expectTypeError(label, operation) {
  try {
    operation();
    trace += label + "-missing;";
  } catch (error) {
    if (!(error instanceof TypeError)) throw label + " wrong error";
    trace += label + ";";
  }
}

expectTypeError("symbol-description", function () {
  Symbol(token);
});

var stringObject = {
  toString: function () {
    trace += "string-hook;";
    return token;
  },
  valueOf: function () {
    throw "string valueOf reached";
  }
};

function stringify(value) {
  return String(value);
}

expectTypeError("String", function () {
  stringify(stringObject);
});

var arrayObject = {
  toString: function () {
    trace += "array-hook;";
    return token;
  },
  valueOf: function () {
    throw "array valueOf reached";
  }
};

expectTypeError("array", function () {
  [arrayObject].toString();
});

if (trace !== "symbol-description;string-hook;String;array-hook;array;") {
  throw trace;
}

"ok";
