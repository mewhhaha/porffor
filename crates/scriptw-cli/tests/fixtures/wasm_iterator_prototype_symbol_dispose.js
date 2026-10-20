const iteratorPrototype = Object.getPrototypeOf(
  Object.getPrototypeOf([][Symbol.iterator]())
);
const dispose = iteratorPrototype[Symbol.dispose];
const receiver = Object.create(iteratorPrototype);
let called = false;

receiver.return = function () {
  called = this === receiver;
  return { done: true };
};

typeof dispose === "function" &&
  dispose.name === "[Symbol.dispose]" &&
  dispose.length === 0 &&
  dispose.call(receiver) === undefined &&
  called &&
  dispose.call({}) === undefined;
