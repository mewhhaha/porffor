function expectTypeError(label, fn) {
  try {
    fn();
  } catch (e) {
    if (e instanceof TypeError) {
      return label;
    }
    return "wrong " + label;
  }
  return "missing " + label;
}

let ordinary = {};
let callResult = expectTypeError("call", function() {
  ordinary();
});
let constructResult = expectTypeError("construct", function() {
  new ordinary();
});
let errorPrototypeCall = expectTypeError("error-call", function() {
  Error.prototype();
});
let errorPrototypeConstruct = expectTypeError("error-construct", function() {
  new Error.prototype();
});

callResult + "," + constructResult + "," + errorPrototypeCall + "," + errorPrototypeConstruct;
