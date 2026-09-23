// Iterator and iterator-result allocations take their prototypes from the
// running execution context's Realm. Exactly fourteen top-level lexical
// bindings captured by a closure make the script's environment record the
// main export's current environment, and that record ends at byte 272 (a
// 48-byte header plus 14 slots of 16 bytes). 272 is also the offset of a
// function object's defining Realm, so an async-generator completion in the
// promise-job drain that read the environment as a function object loaded the
// first word past the record and trapped out of bounds.
let c0 = { n: 0 }, c1 = { n: 1 }, c2 = { n: 2 }, c3 = { n: 3 }, c4 = { n: 4 };
let c5 = { n: 5 }, c6 = { n: 6 }, c7 = { n: 7 }, c8 = { n: 8 }, c9 = { n: 9 };
let c10 = { n: 10 }, c11 = { n: 11 }, c12 = { n: 12 }, c13 = { n: 13 };
function capturedSum() {
  return c0.n + c1.n + c2.n + c3.n + c4.n + c5.n + c6.n + c7.n + c8.n + c9.n +
    c10.n + c11.n + c12.n + c13.n;
}

function check(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}

let iteratorPrototype = Object.getPrototypeOf(
  Object.getPrototypeOf([][Symbol.iterator]())
);
let arrayIteratorPrototype = Object.getPrototypeOf([][Symbol.iterator]());
let text = String("aba");
let stringIterator = text[Symbol.iterator]();
let regExpStringIterator = text.matchAll(/a/g);
let stringIteratorPrototype = Object.getPrototypeOf(stringIterator);
let regExpStringIteratorPrototype = Object.getPrototypeOf(regExpStringIterator);
let stringStep = stringIterator.next();
let regExpStep = regExpStringIterator.next();
check(Object.getPrototypeOf(arrayIteratorPrototype), iteratorPrototype, "%ArrayIteratorPrototype% parent");
check(Object.getPrototypeOf(stringIteratorPrototype), iteratorPrototype, "%StringIteratorPrototype% parent");
check(Object.getPrototypeOf(regExpStringIteratorPrototype), iteratorPrototype, "%RegExpStringIteratorPrototype% parent");
check(arrayIteratorPrototype[Symbol.toStringTag], "Array Iterator", "array iterator tag");
check(stringIteratorPrototype[Symbol.toStringTag], "String Iterator", "string iterator tag");
check(regExpStringIteratorPrototype[Symbol.toStringTag], "RegExp String Iterator", "regexp string iterator tag");

// CreateIteratorResultObject from the String and RegExp String iterator
// next methods.
check(Object.getPrototypeOf(stringStep), Object.prototype, "string iterator result prototype");
check(stringStep.value, "a", "string iterator result value");
check(Object.getPrototypeOf(regExpStep), Object.prototype, "regexp string iterator result prototype");
check(regExpStep.value[0], "a", "regexp string iterator result value");

// CreateArrayIterator (keys / values / entries / @@iterator).
let array = [7, 8];
for (let iterator of [array.keys(), array.values(), array.entries(), array[Symbol.iterator]()]) {
  check(Object.getPrototypeOf(iterator), arrayIteratorPrototype, "array iterator prototype");
}
let typed = new Uint8Array([5, 6]);
check(Object.getPrototypeOf(typed.values()), arrayIteratorPrototype, "typed array iterator prototype");

// CreateIteratorResultObject from the Array iterator next method.
let arrayStep = array.values().next();
check(Object.getPrototypeOf(arrayStep), Object.prototype, "array iterator result prototype");
check(arrayStep.value, 7, "array iterator result value");
check(arrayStep.done, false, "array iterator result done");

// AsyncGeneratorCompleteStep runs in the main export's promise-job drain.
let settled = [];
async function* generator() {
  await 0;
  yield "first";
  await 0;
  return "last";
}
let running = generator();
running.next().then(function (step) {
  settled.push(Object.getPrototypeOf(step) === Object.prototype);
  settled.push(step.value + ":" + step.done);
  return running.next();
}).then(function (step) {
  settled.push(Object.getPrototypeOf(step) === Object.prototype);
  settled.push(step.value + ":" + step.done);
  check(settled.join(","), "true,first:false,true,last:true", "async generator results");
  check(capturedSum(), 91, "captured bindings intact");
  print("iterator-allocation-running-realm:true");
});

true;
