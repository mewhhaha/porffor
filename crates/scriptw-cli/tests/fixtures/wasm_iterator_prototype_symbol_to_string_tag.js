const iteratorPrototype = Iterator.prototype;
const desc = Object.getOwnPropertyDescriptor(
  iteratorPrototype,
  Symbol.toStringTag
);
const receiver = Object.create(iteratorPrototype);
const fakeGeneratorPrototype = Object.create(iteratorPrototype);
const objectWithOwnTag = { [Symbol.toStringTag]: "aa" };
let homeSetterThrew = false;
let homeAssignmentThrew = false;

try {
  desc.set.call(iteratorPrototype, "Changed");
} catch (error) {
  homeSetterThrew = error instanceof TypeError;
}

try {
  iteratorPrototype[Symbol.toStringTag] = "Changed";
} catch (error) {
  homeAssignmentThrew = error instanceof TypeError;
}

desc.set.call(receiver, "Custom Iterator");
Object.freeze(iteratorPrototype);
fakeGeneratorPrototype[Symbol.toStringTag] = "Fake Iterator";
desc.set.call(objectWithOwnTag, "Object Iterator");

typeof desc.get === "function" &&
  typeof desc.set === "function" &&
  desc.enumerable === false &&
  desc.configurable === true &&
  desc.get.call() === "Iterator" &&
  iteratorPrototype[Symbol.toStringTag] === "Iterator" &&
  homeSetterThrew &&
  homeAssignmentThrew &&
  receiver[Symbol.toStringTag] === "Custom Iterator" &&
  fakeGeneratorPrototype[Symbol.toStringTag] === "Fake Iterator" &&
  objectWithOwnTag[Symbol.toStringTag] === "Object Iterator";
