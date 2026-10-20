function argumentsSpecialAccessorsShareTheirSetterBindings(a) {
  let lengthValue = 7;
  let calleeValue = 9;
  Object.defineProperty(arguments, "length", {
    get: function () { return lengthValue; },
    set: function (next) { lengthValue = next; },
    configurable: true
  });
  Object.defineProperty(arguments, "callee", {
    get: function () { return calleeValue; },
    set: function (next) { calleeValue = next; },
    configurable: true
  });
  arguments.length = 11;
  arguments.callee = 13;
  return arguments.length === 11 && arguments.callee === 13;
}

argumentsSpecialAccessorsShareTheirSetterBindings(1);
