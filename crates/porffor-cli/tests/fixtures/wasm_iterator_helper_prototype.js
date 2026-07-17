function assertSameValue(actual, expected, label) {
  if (actual !== expected) throw label;
}

function assertTypeError(callback, label) {
  var threw = false;
  try {
    callback();
  } catch (error) {
    threw = error instanceof TypeError;
  }
  if (!threw) throw label;
}

function iteratorFromOne() {
  return Iterator.from([1]);
}

var helpers = [
  iteratorFromOne().map(function (value) { return value; }),
  iteratorFromOne().filter(function () { return true; }),
  iteratorFromOne().flatMap(function (value) { return [value]; }),
  iteratorFromOne().take(1),
  iteratorFromOne().drop(0),
  Iterator.zip([[1]]),
];
var helperPrototype = Object.getPrototypeOf(helpers[0]);
var next = helperPrototype.next;
var returnMethod = helperPrototype.return;

assertSameValue(Object.getPrototypeOf(helperPrototype), Iterator.prototype, "helper parent");
var wrapper = Iterator.from({
  next: function () { return { done: true, value: undefined }; },
});
var wrapperPrototype = Object.getPrototypeOf(wrapper);
assertSameValue(wrapperPrototype === helperPrototype, false, "wrapper prototype");
assertSameValue(Object.getPrototypeOf(wrapperPrototype), Iterator.prototype, "wrapper parent");

for (var index = 0; index < helpers.length; index = index + 1) {
  var helper = helpers[index];
  assertSameValue(Object.getPrototypeOf(helper), helperPrototype, "shared helper prototype");
  assertSameValue(helper.next, next, "shared next");
  assertSameValue(helper.return, returnMethod, "shared return");
  assertSameValue(Object.prototype.hasOwnProperty.call(helper, "next"), false, "helper own next");
  assertSameValue(Object.prototype.hasOwnProperty.call(helper, "return"), false, "helper own return");
}

var nextDescriptor = Object.getOwnPropertyDescriptor(helperPrototype, "next");
var returnDescriptor = Object.getOwnPropertyDescriptor(helperPrototype, "return");
var tagDescriptor = Object.getOwnPropertyDescriptor(helperPrototype, Symbol.toStringTag);
assertSameValue(nextDescriptor.value, next, "next descriptor value");
assertSameValue(next.name, "next", "next name");
assertSameValue(next.length, 0, "next length");
assertSameValue(nextDescriptor.writable, true, "next writable");
assertSameValue(nextDescriptor.enumerable, false, "next enumerable");
assertSameValue(nextDescriptor.configurable, true, "next configurable");
assertSameValue(returnDescriptor.value, returnMethod, "return descriptor value");
assertSameValue(returnMethod.name, "return", "return name");
assertSameValue(returnMethod.length, 0, "return length");
assertSameValue(returnDescriptor.writable, true, "return writable");
assertSameValue(returnDescriptor.enumerable, false, "return enumerable");
assertSameValue(returnDescriptor.configurable, true, "return configurable");
assertSameValue(tagDescriptor.value, "Iterator Helper", "tag value");
assertSameValue(tagDescriptor.writable, false, "tag writable");
assertSameValue(tagDescriptor.enumerable, false, "tag enumerable");
assertSameValue(tagDescriptor.configurable, true, "tag configurable");
assertSameValue(Object.prototype.toString.call(helpers[0]), "[object Iterator Helper]", "helper toString");

assertTypeError(function () { next.call({}); }, "plain next receiver");
assertTypeError(function () { returnMethod.call({}); }, "plain return receiver");
var forged = Object.create(helperPrototype);
assertTypeError(function () { next.call(forged); }, "forged next receiver");
assertTypeError(function () { returnMethod.call(forged); }, "forged return receiver");

var other = __porfCreateRealm().global;
var otherSource = {
  __proto__: other.Iterator.prototype,
  next: function () { return { done: false, value: 2 }; },
};
var otherMap = other.Iterator.prototype.map.call(otherSource, function (value) { return value + 1; });
var otherZip = other.Iterator.zip([[4]]);
var otherHelperPrototype = Object.getPrototypeOf(otherMap);
assertSameValue(otherHelperPrototype === helperPrototype, false, "other helper prototype");
assertSameValue(Object.getPrototypeOf(otherHelperPrototype), other.Iterator.prototype, "other helper parent");
assertSameValue(otherHelperPrototype.next === next, false, "other next identity");
assertSameValue(otherHelperPrototype.return === returnMethod, false, "other return identity");
assertSameValue(Object.getPrototypeOf(otherHelperPrototype.next), other.Function.prototype, "other next function prototype");
assertSameValue(Object.getPrototypeOf(otherHelperPrototype.next) === Function.prototype, false, "other next not main function prototype");
assertSameValue(Object.getPrototypeOf(otherHelperPrototype.return), other.Function.prototype, "other return function prototype");
assertSameValue(Object.getPrototypeOf(otherHelperPrototype.return) === Function.prototype, false, "other return not main function prototype");
assertSameValue(next.call(otherMap).value, 3, "borrowed next map");
assertSameValue(next.call(otherZip).value[0], 4, "borrowed next zip");
var otherReturningMap = other.Iterator.prototype.map.call(
  {
    __proto__: other.Iterator.prototype,
    next: function () { return { done: false, value: 5 }; },
  },
  function (value) { return value; }
);
assertSameValue(returnMethod.call(otherReturningMap).done, true, "borrowed return map");

var mainError = null;
try {
  next.call({});
} catch (error) {
  mainError = error;
}
var otherError = null;
try {
  otherHelperPrototype.next.call({});
} catch (error) {
  otherError = error;
}
assertSameValue(mainError instanceof TypeError, true, "main error realm");
assertSameValue(mainError instanceof other.TypeError, false, "main error not other realm");
assertSameValue(otherError instanceof TypeError, false, "other error not main realm");
assertSameValue(otherError instanceof other.TypeError, true, "other error realm");

assertSameValue(other.Iterator.prototype === Iterator.prototype, false, "other iterator prototype");
var mainIterator = iteratorFromOne();
assertSameValue(Iterator.from(mainIterator), mainIterator, "main Iterator.from keeps iterator");
function functionIterator() {}
functionIterator.next = function () { return { done: false, value: 12 }; };
var functionIteratorWrapper = Iterator.from(functionIterator);
assertSameValue(functionIteratorWrapper === functionIterator, false, "function iterator wrapped");
assertSameValue(functionIteratorWrapper.next().value, 12, "function iterator wrapper next");
var proxyGetPrototypeOfObserved = false;
var proxyIterator = new Proxy(mainIterator, {
  getPrototypeOf: function (target) {
    proxyGetPrototypeOfObserved = true;
    return Object.getPrototypeOf(target);
  },
});
assertSameValue(Iterator.from(proxyIterator), proxyIterator, "proxy iterator not wrapped");
assertSameValue(proxyGetPrototypeOfObserved, true, "proxy iterator getPrototypeOf observed");
var otherWrapper = other.Iterator.from({
  next: function () { return { done: false, value: 7 }; },
  return: function () { return { done: true, value: 8 }; },
});
var otherWrapperPrototype = Object.getPrototypeOf(otherWrapper);
assertSameValue(otherWrapperPrototype === wrapperPrototype, false, "other wrapper prototype");
assertSameValue(Object.getPrototypeOf(otherWrapperPrototype), other.Iterator.prototype, "other wrapper parent");
assertSameValue(otherWrapperPrototype.next === wrapperPrototype.next, false, "other wrapper next identity");
assertSameValue(otherWrapperPrototype.return === wrapperPrototype.return, false, "other wrapper return identity");
assertSameValue(Object.getPrototypeOf(otherWrapperPrototype.next), other.Function.prototype, "other wrapper next function prototype");
assertSameValue(Object.getPrototypeOf(otherWrapperPrototype.return), other.Function.prototype, "other wrapper return function prototype");
assertSameValue(other.Iterator.from(otherWrapper), otherWrapper, "other Iterator.from keeps iterator");

var foreignWrapper = other.Iterator.from(mainIterator);
assertSameValue(foreignWrapper === mainIterator, false, "foreign iterator rewrapped");
assertSameValue(Object.getPrototypeOf(foreignWrapper), otherWrapperPrototype, "foreign wrapper realm prototype");
assertSameValue(Object.prototype.hasOwnProperty.call(foreignWrapper, "next"), false, "foreign wrapper own next");
assertSameValue(Object.prototype.hasOwnProperty.call(foreignWrapper, "return"), false, "foreign wrapper own return");
assertSameValue(foreignWrapper.next().value, 1, "foreign wrapper iteration");
var foreignReturn = foreignWrapper.return();
assertSameValue(foreignReturn.done, true, "foreign wrapper return");
assertSameValue(Object.getPrototypeOf(foreignReturn), other.Object.prototype, "foreign wrapper return realm");

var mainNoReturnWrapper = Iterator.from({
  next: function () { return { done: true, value: undefined }; },
});
var otherNoReturnWrapper = other.Iterator.from({
  next: function () { return { done: true, value: undefined }; },
});
var mainBorrowedReturn = wrapperPrototype.return.call(otherNoReturnWrapper);
assertSameValue(Object.getPrototypeOf(mainBorrowedReturn), Object.prototype, "main borrowed return realm");
var otherBorrowedReturn = otherWrapperPrototype.return.call(mainNoReturnWrapper);
assertSameValue(Object.getPrototypeOf(otherBorrowedReturn), other.Object.prototype, "other borrowed return realm");

var mainStringIterator = String.prototype[Symbol.iterator];
var otherStringIterator = other.String.prototype[Symbol.iterator];
var otherStringIteratorObserved = false;
var customOtherStringIterator = function () {
  otherStringIteratorObserved = true;
  return [9][Symbol.iterator]();
};
other.String.prototype[Symbol.iterator] = customOtherStringIterator;
assertSameValue(other.String.prototype[Symbol.iterator], customOtherStringIterator, "other String iterator installed");
var otherStringIteratorFrom = other.Iterator.from("x");
assertSameValue(otherStringIteratorObserved, true, "other String iterator observed");
assertSameValue(otherStringIteratorFrom.next().value, 9, "other String iterator result");
assertSameValue(String.prototype[Symbol.iterator], mainStringIterator, "main String iterator unchanged");
other.String.prototype[Symbol.iterator] = otherStringIterator;

var otherWrapperError = null;
try {
  otherWrapperPrototype.next.call({});
} catch (error) {
  otherWrapperError = error;
}
assertSameValue(otherWrapperError instanceof TypeError, false, "other wrapper error not main realm");
assertSameValue(otherWrapperError instanceof other.TypeError, true, "other wrapper error realm");

true;
