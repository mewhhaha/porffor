var trace = "";
var token = Symbol("to-length");

function symbolLength(label) {
  return {
    valueOf: function () {
      trace += label + "-coerce;";
      return token;
    },
    toString: function () {
      throw label + " toString reached";
    },
  };
}

function expectTypeError(label, operation) {
  try {
    operation();
    throw label + " did not throw";
  } catch (error) {
    if (!(error instanceof TypeError)) throw label + " wrong error";
    trace += label + "-caught;";
  }
}

// Keep the source runtime-computed so this RegExp misses the AOT program table
// and reaches the bounded simple fallback matcher.
var simple = new RegExp(String.fromCharCode(97), "g");
var simpleLastIndex = symbolLength("simple");
simple.lastIndex = simpleLastIndex;
expectTypeError("simple", function () {
  simple.exec("ba");
});
if (simple.lastIndex !== simpleLastIndex) throw "simple lastIndex changed";

var program = /[a-f]d/g;
var programLastIndex = symbolLength("program");
program.lastIndex = programLastIndex;
expectTypeError("program", function () {
  program.exec("ad");
});
if (program.lastIndex !== programLastIndex) throw "program lastIndex changed";

var indexRead = false;
var source = {
  length: symbolLength("fromAsync"),
  get 0() {
    indexRead = true;
    throw "array-like index reached";
  },
};

var promise;
try {
  promise = Array.fromAsync(source);
  trace += "fromAsync-returned;";
} catch (error) {
  throw "Array.fromAsync threw synchronously";
}

promise.then(
  function () {
    print("to-length-routes:unexpected-fulfillment");
  },
  function (error) {
    if (!(error instanceof TypeError)) throw "Array.fromAsync wrong rejection";
    if (indexRead) throw "Array.fromAsync read an index after ToLength threw";
    trace += "fromAsync-rejected;";
    if (
      trace !==
      "simple-coerce;simple-caught;program-coerce;program-caught;" +
        "fromAsync-coerce;fromAsync-returned;fromAsync-rejected;"
    ) {
      throw trace;
    }
    print("to-length-routes:ok");
  },
);

"ok";
