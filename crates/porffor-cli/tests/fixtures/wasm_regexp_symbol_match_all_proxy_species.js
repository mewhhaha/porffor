const calls = [];
const matcher = {
  lastIndex: 0,
  exec(value) {
    calls.push("exec:" + value);
    return null;
  },
};
const speciesTarget = function () {};
const species = new Proxy(speciesTarget, {
  construct(target, argumentsList, newTarget) {
    calls.push(
      "construct:" +
        (argumentsList[0] === receiver) +
        ":" +
        argumentsList[1] +
        ":" +
        (newTarget === species),
    );
    return matcher;
  },
});
const constructor = {};
Object.defineProperty(constructor, Symbol.species, {
  get() {
    calls.push("species");
    return species;
  },
});
const receiver = {
  get constructor() {
    calls.push("constructor");
    return constructor;
  },
  get flags() {
    calls.push("flags");
    return {
      toString() {
        calls.push("flags:string");
        return "g";
      },
    };
  },
  get lastIndex() {
    calls.push("lastIndex");
    return 2;
  },
};
const input = {
  toString() {
    calls.push("input:string");
    return "abc";
  },
};

const iterator = RegExp.prototype[Symbol.matchAll].call(receiver, input);
const result = iterator.next();

print(
  "regexp-symbol-match-all-proxy-species:" +
    result.done +
    ":" +
    matcher.lastIndex +
    ":" +
    calls.join(","),
);

0;
