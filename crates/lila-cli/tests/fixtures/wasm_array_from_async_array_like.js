const events = [];
let iteratorKeyCount = 0;
let iteratorKeysWereSymbols = true;
const sourceTarget = {
  get [Symbol.asyncIterator]() {
    events.push("asyncIterator");
    return null;
  },
  get [Symbol.iterator]() {
    events.push("iterator");
    return undefined;
  },
  get length() {
    events.push("length");
    return 2;
  },
  get 0() {
    events.push("get:0");
    return {
      then(resolve) {
        events.push("then:0");
        resolve(3);
      },
    };
  },
  get 1() {
    events.push("get:1");
    return 4;
  },
};
const source = new Proxy(sourceTarget, {
  get(target, key, receiver) {
    if (key === Symbol.asyncIterator || key === Symbol.iterator) {
      iteratorKeyCount = iteratorKeyCount + 1;
    }
    if (key === "Symbol.asyncIterator" || key === "Symbol.iterator") {
      iteratorKeyCount = iteratorKeyCount + 1;
      iteratorKeysWereSymbols = false;
    }
    return Reflect.get(target, key, receiver);
  },
});

Array.fromAsync(source, function (value, index) {
  events.push(`map:${index}:${value}`);
  return index === 0 ? Promise.resolve(value * 2 + index) : value * 2 + index;
}).then(function (result) {
  let inputWasRead = false;
  const rejected = Array.fromAsync(
    {
      get [Symbol.asyncIterator]() {
        inputWasRead = true;
        return undefined;
      },
    },
    0,
  );

  rejected.then(
    function () {
      print("array-from-async-array-like:unexpected-fulfillment");
    },
    function (error) {
      print(
        "array-from-async-array-like:" +
          result.join(",") +
          ":" +
          events.join(",") +
          ":" +
          (error instanceof TypeError) +
          ":" +
          inputWasRead +
          ":" +
          (iteratorKeysWereSymbols && iteratorKeyCount === 2),
      );
    },
  );
});

0;
