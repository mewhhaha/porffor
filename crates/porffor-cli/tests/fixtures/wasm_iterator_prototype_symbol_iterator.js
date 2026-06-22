const iteratorPrototype = Object.getPrototypeOf(
  Object.getPrototypeOf([][Symbol.iterator]())
);
const method = iteratorPrototype[Symbol.iterator];
const receiver = { value: 42 };

typeof method === "function" &&
  method.name === "[Symbol.iterator]" &&
  method.length === 0 &&
  method.call(receiver) === receiver;
