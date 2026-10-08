function check(value, message) { if (!value) throw message; }
var pending = [];
var trace = [];
var stack = new AsyncDisposableStack();
var resource = {};
var asyncReads = 0;
var syncReads = 0;
var method = new Proxy(function () {}, {
  apply: function (target, receiver, argumentsList) {
    check(receiver === resource && argumentsList.length === 0, "use Call operands");
    trace.push("use");
    return undefined;
  }
});
Object.defineProperty(resource, Symbol.asyncDispose, {
  get: function () { asyncReads++; return null; }
});
Object.defineProperty(resource, Symbol.dispose, {
  get: function () { syncReads++; return method; }
});
check(stack.use(resource) === resource, "use returns resource");
var adopted = Symbol("adopted");
var adopt = new Proxy(function () {}, {
  apply: function (target, receiver, argumentsList) {
    check(receiver === undefined && argumentsList.length === 1 && argumentsList[0] === adopted, "adopt Call operands");
    trace.push("adopt");
  }
});
check(stack.adopt(adopted, adopt) === adopted, "adopt returns resource");
var defer = new Proxy(function () {}, {
  apply: function (target, receiver, argumentsList) {
    check(receiver === undefined && argumentsList.length === 0, "defer Call operands");
    trace.push("defer");
  }
});
check(stack.defer(defer) === undefined, "defer returns undefined");
check(asyncReads === 1 && syncReads === 1, "optional GetMethod exactly once");
pending.push(stack.disposeAsync().then(function (value) {
  check(value === undefined && trace.join(",") === "defer,adopt,use", "reverse disposal order");
}));
check(stack.disposed, "disposed before callbacks");
var count = 0;
var moved;
var original = new AsyncDisposableStack();
original.defer(function () { count++; });
var movedResource = {
  get [Symbol.asyncDispose]() {
    moved = original.move();
    return function () { count += 10; };
  }
};
check(original.use(movedResource) === movedResource, "registration survives getter move");
check(original.disposed && !moved.disposed, "move lifecycle");
pending.push(moved.disposeAsync().then(function () { check(count === 11, "captured List transfers before Get"); }));
var values = [undefined, null, false, 42, -0, NaN, 1n, Symbol("value"), "string", function () {}, {}];
var valuesStack = new AsyncDisposableStack();
var received = [];
for (let i = 0; i < values.length; i++) {
  let saved = values[i];
  check(Object.is(valuesStack.adopt(saved, function (value) {
    check(Object.is(value, saved), "complete stored resource value");
    received.push(i);
  }), saved), "complete adopt return value");
}
pending.push(valuesStack.disposeAsync().then(function () {
  check(received.join(",") === "10,9,8,7,6,5,4,3,2,1,0", "all resource tags in reverse order");
}));
// A captured List may be disposed by a getter before that same registration appends.
// Growing its table must preserve nullable consumed entries without a cast trap.
var reentrant = new AsyncDisposableStack();
var calls = 0;
var disposal;
for (let i = 0; i < 4; i++) reentrant.defer(function () { calls++; });
var unusedCalls = 0;
var late = { get [Symbol.asyncDispose]() {
  disposal = reentrant.disposeAsync();
  return function () { unusedCalls++; };
} };
check(reentrant.use(late) === late, "reentrant disposal during registration");
pending.push(disposal.then(function () {
  check(calls === 4 && unusedCalls === 0, "already-started walk retains its entries");
}));
var marker = {};
var failed = new AsyncDisposableStack();
var fallback = 0;
try {
  failed.use({ get [Symbol.asyncDispose]() { throw marker; }, get [Symbol.dispose]() { fallback++; } });
  throw "getter accepted";
} catch (error) { check(error === marker && fallback === 0, "original Get abrupt and cutoff"); }
check(!failed.disposed, "failed registration leaves pending stack");
Promise.all(pending).then(function () { print("async-stack-registrations:ok"); });
262;
