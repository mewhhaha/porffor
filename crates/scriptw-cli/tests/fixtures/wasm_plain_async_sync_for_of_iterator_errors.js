// IteratorNext, IteratorComplete, and IteratorValue failures happen before the
// loop body owns a completion, so none of them performs IteratorClose.

function same(actual, expected, label) {
  if (actual !== expected) throw label;
}

async function rejectionOf(promise, label) {
  try {
    await promise;
  } catch (error) {
    return error;
  }
  throw label + " fulfilled";
}

const savedArrayIterator = Object.getOwnPropertyDescriptor(
  Array.prototype,
  Symbol.iterator
);
let activeIterator;
let nextCloseCalls = 0;
let doneCloseCalls = 0;
let valueCloseCalls = 0;
const nextError = { label: "next" };
const doneError = { label: "done" };
const valueError = { label: "value" };

const nextIterator = {
  next() {
    throw nextError;
  },
  return() {
    nextCloseCalls++;
    return { done: true };
  },
};

const doneIterator = {
  next() {
    return {
      get done() {
        throw doneError;
      },
    };
  },
  return() {
    doneCloseCalls++;
    return { done: true };
  },
};

const valueIterator = {
  next() {
    return {
      done: false,
      get value() {
        throw valueError;
      },
    };
  },
  return() {
    valueCloseCalls++;
    return { done: true };
  },
};

async function consumeActiveIterator() {
  for (const value of [0]) {
    if (value === "unreachable") throw "unreachable body";
    await 0;
  }
}

async function main() {
  try {
    Object.defineProperty(Array.prototype, Symbol.iterator, {
      configurable: true,
      value: function () {
        return activeIterator;
      },
    });

    activeIterator = nextIterator;
    same(
      await rejectionOf(consumeActiveIterator(), "next error loop"),
      nextError,
      "next error identity"
    );
    same(nextCloseCalls, 0, "next error close count");

    activeIterator = doneIterator;
    same(
      await rejectionOf(consumeActiveIterator(), "done error loop"),
      doneError,
      "done error identity"
    );
    same(doneCloseCalls, 0, "done error close count");

    activeIterator = valueIterator;
    same(
      await rejectionOf(consumeActiveIterator(), "value error loop"),
      valueError,
      "value error identity"
    );
    same(valueCloseCalls, 0, "value error close count");
  } finally {
    Object.defineProperty(
      Array.prototype,
      Symbol.iterator,
      savedArrayIterator
    );
  }

  print("plain-async-sync-for-of:protocol-errors=ok");
}

main().then(undefined, function (error) {
  print("plain-async-sync-for-of:protocol-errors=FAILED:" + error);
});

0;
