function throwsTypeError(fn) {
  try {
    fn();
  } catch (error) {
    return error instanceof TypeError;
  }
  return false;
}

var dense = [41];
var denseResult = dense.pop();
dense.length = 1;
var denseDoesNotResurrect = denseResult === 41
  && !(0 in dense)
  && dense[0] === undefined;

var getterCalls = 0;
var accessor = [];
Object.defineProperty(accessor, "0", {
  configurable: true,
  get: function () {
    getterCalls++;
    return 23;
  }
});
var accessorResult = accessor.pop();
var accessorWasReadAndDeleted = accessorResult === 23
  && getterCalls === 1
  && accessor.length === 0
  && !Object.prototype.hasOwnProperty.call(accessor, "0");

var locked = [7];
Object.defineProperty(locked, "0", { configurable: false });
var lockedThrows = throwsTypeError(function () {
  locked.pop();
});
var lockedIsUnchanged = locked.length === 1
  && locked[0] === 7
  && Object.prototype.hasOwnProperty.call(locked, "0");

var partial = [13];
Object.defineProperty(partial, "length", { writable: false });
var partialThrows = throwsTypeError(function () {
  partial.pop();
});
var partialDeletePrecedesLengthThrow = partial.length === 1
  && !(0 in partial)
  && !Object.prototype.hasOwnProperty.call(partial, "0");

var empty = [];
Object.defineProperty(empty, "length", { writable: false });
var emptySameValueWriteThrows = throwsTypeError(function () {
  empty.pop();
});

denseDoesNotResurrect
  && accessorWasReadAndDeleted
  && lockedThrows
  && lockedIsUnchanged
  && partialThrows
  && partialDeletePrecedesLengthThrow
  && emptySameValueWriteThrows;
