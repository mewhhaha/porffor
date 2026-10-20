// A synchronous iterator used by `for-of` inside a plain async function must
// survive the body's await without being reacquired. Array and String sources
// are both exact literals so this also catches either old source-kind shortcut.

function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

const savedArrayIterator = Object.getOwnPropertyDescriptor(
  Array.prototype,
  Symbol.iterator
);
const savedStringIterator = Object.getOwnPropertyDescriptor(
  String.prototype,
  Symbol.iterator
);

let arrayIteratorMethodReads = 0;
let arrayIteratorMethodCalls = 0;
let arrayNextReads = 0;
let arrayNextCalls = 0;
let arrayReturnCalls = 0;
let arrayMethodReceiver;
let arrayTrace = "";
let arrayIndex = 0;
const arrayIterator = {
  get next() {
    arrayNextReads++;
    arrayTrace += "get-next>";
    return function () {
      if (this !== arrayIterator) throw "array next receiver";
      arrayTrace += "next:" + arrayIndex + ">";
      arrayNextCalls++;
      if (arrayIndex === 0) {
        arrayIndex++;
        return { value: "4", done: false };
      }
      if (arrayIndex === 1) {
        arrayIndex++;
        return { value: "8", done: false };
      }
      return { value: undefined, done: true };
    };
  },
  return() {
    arrayReturnCalls++;
    return { value: undefined, done: true };
  },
};

let stringIteratorMethodCalls = 0;
let stringNextCalls = 0;
let stringValue;
let assignedValue = "initial";
let assignmentTrace = "";
let assignmentIteratorCalls = 0;
let assignmentNextCalls = 0;
const customIterable = {
  [Symbol.iterator]: function () {
    assignmentIteratorCalls++;
    let index = 0;
    return {
      next: function () {
        assignmentNextCalls++;
        if (index === 0) {
          index++;
          return { value: "assigned-1", done: false };
        }
        if (index === 1) {
          index++;
          return { value: "assigned-2", done: false };
        }
        return { value: undefined, done: true };
      },
    };
  },
};

async function consumeArray() {
  let values = "";
  for (const value of [1]) {
    values += value;
    arrayTrace += "body:" + value + ">";
    await 0;
    arrayTrace += "resume:" + value + ">";
  }
  return values;
}

async function consumeString() {
  for (const value of "native") {
    stringValue = value;
    await 0;
  }
}

async function consumeBareAssignment() {
  for (assignedValue of customIterable) {
    assignmentTrace += "body:" + assignedValue + ">";
    await 0;
    assignmentTrace += "resume:" + assignedValue + ">";
  }
}

async function main() {
  try {
    Object.defineProperty(Array.prototype, Symbol.iterator, {
      configurable: true,
      get: function () {
        arrayIteratorMethodReads++;
        arrayTrace += "get-iterator>";
        return function () {
          arrayIteratorMethodCalls++;
          if (arrayMethodReceiver === undefined) arrayMethodReceiver = this;
          if (this !== arrayMethodReceiver) throw "array iterator receiver";
          arrayTrace += "call-iterator>";
          return arrayIterator;
        };
      },
    });

    const values = await consumeArray();
    same(values, "48", "custom Array iterator values");
    same(arrayIteratorMethodReads, 1, "Array @@iterator reads");
    same(arrayIteratorMethodCalls, 1, "Array @@iterator calls");
    same(arrayNextReads, 1, "Array next reads");
    same(arrayNextCalls, 3, "Array next calls");
    same(arrayReturnCalls, 0, "Array return on natural exhaustion");
    same(
      arrayTrace,
      "get-iterator>call-iterator>get-next>next:0>body:4>resume:4>" +
        "next:1>body:8>resume:8>next:2>",
      "Array iterator trace"
    );
  } finally {
    Object.defineProperty(
      Array.prototype,
      Symbol.iterator,
      savedArrayIterator
    );
  }

  await consumeBareAssignment();
  same(assignedValue, "assigned-2", "bare assignment final value");
  same(assignmentIteratorCalls, 1, "bare assignment iterator calls");
  same(assignmentNextCalls, 3, "bare assignment next calls");
  same(
    assignmentTrace,
    "body:assigned-1>resume:assigned-1>" +
      "body:assigned-2>resume:assigned-2>",
    "bare assignment trace"
  );

  try {
    Object.defineProperty(String.prototype, Symbol.iterator, {
      configurable: true,
      value: function () {
        "use strict";
        if (this !== "native") throw "String iterator receiver";
        stringIteratorMethodCalls++;
        let done = false;
        return {
          next: function () {
            stringNextCalls++;
            if (done) return { value: undefined, done: true };
            done = true;
            return { value: 9, done: false };
          },
        };
      },
    });

    await consumeString();
    same(stringValue, 9, "custom String iterator value");
    same(stringIteratorMethodCalls, 1, "String @@iterator calls");
    same(stringNextCalls, 2, "String next calls");
  } finally {
    Object.defineProperty(
      String.prototype,
      Symbol.iterator,
      savedStringIterator
    );
  }

  print("plain-async-sync-for-of:protocol=ok");
}

main().then(undefined, function (error) {
  print("plain-async-sync-for-of:protocol=FAILED:" + error);
});

0;
