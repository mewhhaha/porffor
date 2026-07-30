const descriptor = Object.getOwnPropertyDescriptor(Iterator, "concat");
if (typeof Iterator.concat !== "function") throw "concat type";
if (Iterator.concat.name !== "concat") throw "concat name";
if (Iterator.concat.length !== 0) throw "concat length";
if (Object.getPrototypeOf(Iterator.concat) !== Function.prototype) throw "concat prototype";
if (!descriptor.writable || descriptor.enumerable || !descriptor.configurable) {
  throw "concat descriptor";
}
__porfAssertThrows(TypeError, function () { new Iterator.concat(); });

let events = [];
function iterable(name, values) {
  return {
    get [Symbol.iterator]() {
      events.push("get " + name);
      return function () {
        events.push("open " + name + " " + arguments.length);
        let index = 0;
        return {
          next() {
            events.push("next " + name + " " + arguments.length);
            if (index === values.length) return { done: true };
            return { done: false, value: values[index++] };
          }
        };
      };
    }
  };
}

let concatenated = Iterator.concat(iterable("a", [1, 2]), iterable("b", []), iterable("c", [3]));
if (events.join(",") !== "get a,get b,get c") throw "eager method lookup";
let first = concatenated.next("ignored");
let second = concatenated.next();
let third = concatenated.next();
let completed = concatenated.next();
let completedAgain = concatenated.next();
if (first.value !== 1 || first.done || second.value !== 2 || second.done) throw "first source";
if (third.value !== 3 || third.done || !completed.done || !completedAgain.done) {
  throw "later sources";
}
if (first === second || completed === completedAgain) throw "fresh result objects";
if (events.join(",") !== [
  "get a", "get b", "get c",
  "open a 0", "next a 0", "next a 0", "next a 0",
  "open b 0", "next b 0",
  "open c 0", "next c 0", "next c 0"
].join(",")) throw "lazy ordered iteration";

let valueRead = false;
let doneBeforeValue = Iterator.concat({
  [Symbol.iterator]() {
    return {
      next() {
        return {
          get done() { return true; },
          get value() { valueRead = true; throw "value"; }
        };
      }
    };
  }
});
if (!doneBeforeValue.next().done || valueRead) throw "done before value";

let openedBeforeReturn = false;
let suspendedStart = Iterator.concat({
  get [Symbol.iterator]() {
    return function () {
      openedBeforeReturn = true;
      return { next() { return { done: true }; } };
    };
  }
});
if (!suspendedStart.return().done || openedBeforeReturn) throw "return before start";

let returnCalls = 0;
let returnArguments = -1;
let active = Iterator.concat({
  [Symbol.iterator]() {
    return {
      next() { return { done: false, value: 4 }; },
      return() {
        returnCalls++;
        returnArguments = arguments.length;
        return {};
      }
    };
  }
});
active.next();
if (!active.return("ignored").done || returnCalls !== 1 || returnArguments !== 0) {
  throw "active return";
}
active.return();
if (returnCalls !== 1) throw "repeated return";

let reentrant;
reentrant = Iterator.concat({
  [Symbol.iterator]() {
    return {
      next() {
        reentrant.next();
        return { done: false };
      }
    };
  }
});
__porfAssertThrows(TypeError, function () { reentrant.next(); });

__porfAssertThrows(TypeError, function () { Iterator.concat("primitive"); });
__porfAssertThrows(TypeError, function () { Iterator.concat({ [Symbol.iterator]: null }); });
if (!Iterator.concat().next().done) throw "zero arguments";
true;
