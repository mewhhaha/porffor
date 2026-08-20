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

let messageResult = expectTypeError("message", function() {
  Error.prototype.toString.call({
    message: { valueOf: undefined, toString: undefined }
  });
});

let nameResult = expectTypeError("name", function() {
  Error.prototype.toString.call({
    name: { valueOf: undefined, toString: undefined }
  });
});

messageResult + "," + nameResult;
