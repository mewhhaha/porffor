// IteratorClose after a resumed synchronous `for-of` keeps the original Throw
// completion, but a close error replaces a Return completion.

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
let iteratorMethodCalls = 0;

const bodyError = { label: "body rejection" };
const bodyCloseError = { label: "body close error" };
let bodyCloseCalls = 0;
const bodyIterator = {
  next() {
    return { value: 1, done: false };
  },
  return() {
    bodyCloseCalls++;
    throw bodyCloseError;
  },
};

const returnCloseError = { label: "return close error" };
let returnCloseCalls = 0;
const returnIterator = {
  next() {
    return { value: 2, done: false };
  },
  return() {
    returnCloseCalls++;
    throw returnCloseError;
  },
};

async function rejectAfterAwait() {
  for (const value of [0]) {
    same(value, 1, "body rejection value");
    await Promise.reject(bodyError);
  }
}

async function returnAfterAwait() {
  for (const value of [0]) {
    same(value, 2, "return value");
    await 0;
    return "unobservable return";
  }
  return "unreachable exhaustion";
}

async function main() {
  try {
    Object.defineProperty(Array.prototype, Symbol.iterator, {
      configurable: true,
      value: function () {
        iteratorMethodCalls++;
        return activeIterator;
      },
    });

    activeIterator = bodyIterator;
    same(
      await rejectionOf(rejectAfterAwait(), "body rejection loop"),
      bodyError,
      "body rejection identity"
    );
    same(bodyCloseCalls, 1, "body rejection close count");

    activeIterator = returnIterator;
    same(
      await rejectionOf(returnAfterAwait(), "return loop"),
      returnCloseError,
      "return close error identity"
    );
    same(returnCloseCalls, 1, "return close count");
    same(iteratorMethodCalls, 2, "iterator method calls");
  } finally {
    Object.defineProperty(
      Array.prototype,
      Symbol.iterator,
      savedArrayIterator
    );
  }

  print("plain-async-sync-for-of:close=ok");
}

main().then(undefined, function (error) {
  print("plain-async-sync-for-of:close=FAILED:" + error);
});

0;
