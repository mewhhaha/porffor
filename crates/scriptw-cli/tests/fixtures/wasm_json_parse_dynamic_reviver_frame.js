function fail(message) {
  throw message;
}

function parse(text, reviver) {
  return JSON.parse(text, reviver);
}

let calls = [];
let nestedHolder;
let rootHolder;
let sourceChecks = 0;

let result = parse('{"":{"leaf":1e+2},"array":[2,3],"later":4}', function (key, value, context) {
  calls.push(key);

  if (key === "leaf") {
    if (context.source !== "1e+2") fail("nested primitive source");
    sourceChecks = sourceChecks + 1;
    return 101;
  }

  if (key === "" && value !== null && typeof value === "object" && value.leaf === 101) {
    nestedHolder = this;
    this.added = 5;
    return "nested-empty-key";
  }

  if (key === "0") {
    if (context.source !== "2") fail("array primitive source");
    sourceChecks = sourceChecks + 1;
    this[1] = 30;
    this.push(40);
    return value;
  }

  if (key === "1") {
    if (value !== 30) fail("forward array mutation");
    if (context.source !== undefined) fail("mutated value source eligibility");
    sourceChecks = sourceChecks + 1;
    return value;
  }

  if (key === "later") return undefined;

  if (key === "") {
    if (context.source !== undefined) fail("root object source eligibility");
    rootHolder = this;
    return { wrapped: value };
  }

  return value;
});

let wrapped = result.wrapped;
if (calls.length !== 7) fail("postorder call count");
if (calls[0] !== "leaf") fail("postorder leaf");
if (calls[1] !== "") fail("nested empty-string key");
if (calls[2] !== "0" || calls[3] !== "1") fail("array snapshot order");
if (calls[4] !== "array" || calls[5] !== "later" || calls[6] !== "") {
  fail("object snapshot order");
}
if (nestedHolder !== wrapped) fail("nested holder identity");
if (rootHolder === nestedHolder) fail("root holder role");
if (rootHolder[""] !== wrapped) fail("synthetic root holder");
if (wrapped[""] !== "nested-empty-key") fail("nested empty-string replacement");
if (wrapped.added !== 5) fail("object snapshot mutation");
if ("later" in wrapped) fail("nested deletion");
if (wrapped.array.length !== 3 || wrapped.array[0] !== 2 || wrapped.array[1] !== 30 || wrapped.array[2] !== 40) {
  fail("array snapshot mutation");
}
if (sourceChecks !== 3) fail("source checks");

let sentinel = {};
let abruptCalls = 0;
let caught = false;
try {
  parse('{"a":{"b":1},"c":2}', function (key, value) {
    abruptCalls = abruptCalls + 1;
    if (key === "b") throw sentinel;
    return value;
  });
} catch (error) {
  caught = error === sentinel;
}
if (!caught || abruptCalls !== 1) fail("abrupt reviver order");

let rootUndefined = parse("0", function () {
  return undefined;
});
if (rootUndefined !== undefined) fail("root undefined result");

true;
