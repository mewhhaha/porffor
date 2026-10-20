// Unicode before callable spans: 🟣 café
function declared() { return import('./target.js'); }
const expression = function named() { return import('./target.js'); };
const arrow = () => import('./target.js');
const asyncArrow = async () => import('./target.js');
function* generator() { return import('./target.js'); }
async function asynchronous() { return import('./target.js'); }
async function* asyncGenerator() { return import('./target.js'); }
const object = { load() { return import('./target.js'); } };
class Witness {
  load() { return import('./target.js'); }
  static load() { return import('./target.js'); }
  #private() { return import('./target.js'); }
  expose() { return this.#private; }
}
const expectedClass = "class Witness {\n  load() { return import('./target.js'); }\n  static load() { return import('./target.js'); }\n  #private() { return import('./target.js'); }\n  expose() { return this.#private; }\n}";
const pairs = [
  [declared, "function declared() { return import('./target.js'); }"],
  [expression, "function named() { return import('./target.js'); }"],
  [arrow, "() => import('./target.js')"],
  [asyncArrow, "async () => import('./target.js')"],
  [generator, "function* generator() { return import('./target.js'); }"],
  [asynchronous, "async function asynchronous() { return import('./target.js'); }"],
  [asyncGenerator, "async function* asyncGenerator() { return import('./target.js'); }"],
  [object.load, "load() { return import('./target.js'); }"],
  [Witness, expectedClass],
  [Witness.prototype.load, "load() { return import('./target.js'); }"],
  [Witness.load, "load() { return import('./target.js'); }"],
  [new Witness().expose(), "#private() { return import('./target.js'); }"],
];
for (const [callable, original] of pairs) {
  const actual = Function.prototype.toString.call(callable);
  if (actual !== original) throw 'Script callable source mismatch: ' + actual;
}
import('./target.js').then(namespace => {
  if (namespace.read.toString() !== "function read() { void import.meta; return import('./other.js'); }")
    throw 'Module callable source mismatch: ' + namespace.read.toString();
  if (namespace.default.toString() !== "function () { return import('./other.js'); }")
    throw 'Default callable source mismatch: ' + namespace.default.toString();
  if (namespace.Class.toString() !== "class Class { load() { return import('./other.js'); } }")
    throw 'Module class source mismatch: ' + namespace.Class.toString();
  print('original callable sources');
});
true;
