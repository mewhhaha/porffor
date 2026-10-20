const originalExec = RegExp.prototype.exec;
const thrown = {};
const throwingIterator = /./g[Symbol.matchAll]("x");

RegExp.prototype.exec = function () {
  throw thrown;
};

let preservedThrow = false;
try {
  throwingIterator.next();
} catch (error) {
  preservedThrow = error === thrown;
}

RegExp.prototype.exec = 5;
const fallbackIterator = /\w/g[Symbol.matchAll]("a*b");
const fallbackFirst = fallbackIterator.next();
const fallbackSecond = fallbackIterator.next();
if (fallbackFirst.value[0] !== "a" || fallbackFirst.value.index !== 0) {
  throw "non-callable exec first match";
}
if (fallbackSecond.value[0] !== "b" || fallbackSecond.value.index !== 2) {
  throw "non-callable exec second match";
}

let callCount = 0;
let receiverWasRegExp = false;
let inputWasForwarded = false;
const firstMatch = ["x"];
const proxiedExec = new Proxy(
  function () {},
  {
    apply(target, receiver, argumentsList) {
      callCount = callCount + 1;
      receiverWasRegExp = receiver instanceof RegExp;
      inputWasForwarded = argumentsList[0] === "x";
      return callCount === 1 ? firstMatch : null;
    },
  },
);
const proxiedIterator = /./g[Symbol.matchAll]("x");
RegExp.prototype.exec = proxiedExec;

const first = proxiedIterator.next();
const second = proxiedIterator.next();
RegExp.prototype.exec = originalExec;

print(
  "regexp-string-iterator-exec-abrupt-and-proxy:" +
    preservedThrow +
    ":" +
    receiverWasRegExp +
    ":" +
    inputWasForwarded +
    ":" +
    (first.value === firstMatch) +
    ":" +
    first.done +
    ":" +
    second.done +
    ":" +
    callCount,
);

0;
