const iteratorPrototype = Iterator.prototype;
const desc = Object.getOwnPropertyDescriptor(iteratorPrototype, "constructor");
const receiver = Object.create(iteratorPrototype);
const fakeGeneratorPrototype = Object.create(iteratorPrototype);
const objectWithOwnConstructor = { constructor: "aa" };
let homeSetterThrew = false;
let homeAssignmentThrew = false;
let primitiveSetterThrew = false;

try {
  desc.set.call(iteratorPrototype, "Changed");
} catch (error) {
  homeSetterThrew = error instanceof TypeError;
}

try {
  iteratorPrototype.constructor = "Changed";
} catch (error) {
  homeAssignmentThrew = error instanceof TypeError;
}

try {
  desc.set.call(true, "Changed");
} catch (error) {
  primitiveSetterThrew = error instanceof TypeError;
}

desc.set.call(receiver, Array);
Object.freeze(iteratorPrototype);
fakeGeneratorPrototype.constructor = Array;
desc.set.call(objectWithOwnConstructor, Array);

typeof desc.get === "function" &&
  typeof desc.set === "function" &&
  desc.enumerable === false &&
  desc.configurable === true &&
  desc.value === undefined &&
  desc.writable === undefined &&
  desc.get.call() === Iterator &&
  iteratorPrototype.constructor === Iterator &&
  homeSetterThrew &&
  homeAssignmentThrew &&
  primitiveSetterThrew &&
  receiver.constructor === Array &&
  fakeGeneratorPrototype.constructor === Array &&
  objectWithOwnConstructor.constructor === Array;
