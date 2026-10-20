function check(value, label) {
  if (!value) throw "concat method lookup: " + label;
}
function invoke(receiver, argument) {
  return receiver.concat(argument);
}

check(new String(42).concat(function() {}()) === "42undefined", "boxed undefined");
check(invoke(new String("box"), undefined) === "boxundefined", "dynamic boxed String");
check(invoke("text", undefined) === "textundefined", "dynamic primitive String");
check("a".concat("b") === "ab", "primitive String control");
check([1].concat([2]).join(":") === "1:2", "Array control");

var boxed = new String("box");
boxed.concat = function(argument) {
  "use strict";
  check(this === boxed && argument === 5, "boxed own method this");
  return 7;
};
check(boxed.concat(5) + 1 === 8, "boxed own method result kind");
var array = [1];
array.concat = function(argument) {
  "use strict";
  check(this === array && argument === 5, "Array own method this");
  return 8;
};
check(array.concat(5) + 1 === 9, "Array own method result kind");

var trace = "";
var holder = {};
Object.defineProperty(holder, "concat", {
  configurable: true,
  get: function() {
    "use strict";
    check(this === holder, "own getter this");
    trace += "get;";
    return function(argument) {
      "use strict";
      check(this === holder && argument === 4, "acquired call this");
      trace += "call;";
      return 10;
    };
  }
});
function replaceDuringArgument() {
  trace += "argument;";
  Object.defineProperty(holder, "concat", {value: function() { throw "reacquired method"; }});
  return 4;
}
check(holder.concat(replaceDuringArgument()) === 10, "retain acquired method");
check(trace === "get;argument;call;", "getter before argument, each once");

trace = "";
Object.defineProperty(array, "concat", {
  get: function() {
    "use strict";
    check(this === array, "Array own getter this");
    trace += "get;";
    return function(argument) {
      "use strict";
      check(this === array && argument === 1, "Array accessor call this");
      trace += "call;";
      return 14;
    };
  }
});
function accessorArgument() { trace += "argument;"; return 1; }
check(array.concat(accessorArgument()) === 14, "Array own accessor");
check(trace === "get;argument;call;", "Array own accessor argument order");

var marker;
var caught = false;
trace = "";
var throwing = {};
Object.defineProperty(throwing, "concat", {
  get: function() { trace += "get;"; throw marker; }
});
function observedArgument() { trace += "argument;"; return 1; }
try { throwing.concat(observedArgument()); }
catch (error) { caught = error === marker; }
check(caught && trace === "get;", "getter throw stops arguments and retains undefined");

trace = "";
caught = false;
var noncallable = { get concat() { trace += "get;"; return 3; } };
try { noncallable.concat(observedArgument()); }
catch (error) { caught = error instanceof TypeError; }
check(caught && trace === "get;argument;", "noncallable check follows arguments");

trace = "";
caught = false;
var argumentMarker = {};
var callable = { get concat() {
  trace += "get;";
  return function() { trace += "call;"; };
}};
function abruptArgument() { trace += "argument;"; throw argumentMarker; }
try { callable.concat(abruptArgument()); }
catch (error) { caught = error === argumentMarker; }
check(caught && trace === "get;argument;", "argument throw stops call");

trace = "";
caught = false;
function nullReceiver() { trace += "receiver;"; return null; }
try { nullReceiver().concat(observedArgument()); }
catch (error) { caught = error instanceof TypeError; }
check(caught && trace === "receiver;", "nullish lookup precedes arguments");

trace = "";
caught = false;
function abruptReceiver() { trace += "receiver;"; throw argumentMarker; }
try { abruptReceiver().concat(observedArgument()); }
catch (error) { caught = error === argumentMarker; }
check(caught && trace === "receiver;", "receiver throw stops lookup and arguments");

var stringDescriptor = Object.getOwnPropertyDescriptor(String.prototype, "concat");
var arrayDescriptor = Object.getOwnPropertyDescriptor(Array.prototype, "concat");
try {
  String.prototype.concat = function() { return 11; };
  check("x".concat() + 1 === 12, "primitive prototype override result kind");
  check(new String("x").concat() + 1 === 12, "boxed prototype override result kind");
  Array.prototype.concat = function() { return 12; };
  check([1].concat() + 1 === 13, "Array prototype override result kind");

  trace = "";
  var boxedForGetter = new String("wrapped");
  Object.defineProperty(String.prototype, "concat", {
    configurable: true,
    get: function() {
      "use strict";
      check(this === "primitive" || this === boxedForGetter, "String getter original this");
      trace += "get;";
      return function(argument) {
        "use strict";
        check((this === "primitive" || this === boxedForGetter) && argument === 1, "String call original this");
        trace += "call;";
        return 13;
      };
    }
  });
  check("primitive".concat(observedArgument()) === 13, "primitive prototype accessor");
  check(trace === "get;argument;call;", "primitive accessor argument order");
  trace = "";
  check(boxedForGetter.concat(observedArgument()) === 13, "boxed prototype accessor");
  check(trace === "get;argument;call;", "boxed accessor argument order");
  trace = "";
  var inheritedArray = [];
  Object.defineProperty(Array.prototype, "concat", {
    configurable: true,
    get: function() {
      "use strict";
      check(this === inheritedArray, "Array prototype getter original this");
      trace += "get;";
      return function(argument) {
        "use strict";
        check(this === inheritedArray && argument === 1, "Array prototype accessor call this");
        trace += "call;";
        return 15;
      };
    }
  });
  check(inheritedArray.concat(observedArgument()) === 15, "Array prototype accessor");
  check(trace === "get;argument;call;", "Array prototype accessor argument order");
} finally {
  Object.defineProperty(String.prototype, "concat", stringDescriptor);
  Object.defineProperty(Array.prototype, "concat", arrayDescriptor);
}

trace = "";
var convertedReceiver = {
  concat: String.prototype.concat,
  toString: function() { trace += "receiver;"; return "base"; }
};
function firstArgument() {
  trace += "eval1;";
  return { toString: function() { trace += "first;"; return "L"; } };
}
function secondArgument() {
  trace += "eval2;";
  return { toString: function() { trace += "second;"; return "R"; } };
}
check(convertedReceiver.concat(firstArgument(), secondArgument()) === "baseLR", "builtin ToString values");
check(trace === "eval1;eval2;receiver;first;second;", "argument evaluation before ordered ToString");

true;

// Transferred builtin aliases retain the authored property reference.
var transferredArray = [1];
var unrelatedArrayCalls = 0;
transferredArray.alias = Array.prototype.concat;
transferredArray.concat = function() { unrelatedArrayCalls++; return 99; };
var transferredArrayResult = transferredArray.alias([2]);
check(transferredArrayResult.length === 2 && transferredArrayResult[0] === 1 && transferredArrayResult[1] === 2 && unrelatedArrayCalls === 0, "Array transferred alias ignores own concat override");

var transferredString = new String("s");
var unrelatedStringCalls = 0;
transferredString.alias = String.prototype.concat;
transferredString.concat = function() { unrelatedStringCalls++; return 99; };
check(transferredString.alias(42) === "s42" && unrelatedStringCalls === 0, "String transferred alias ignores own concat override");

trace = "";
var arrayWithCanonicalGetter = [3];
arrayWithCanonicalGetter.alias = Array.prototype.concat;
Object.defineProperty(arrayWithCanonicalGetter, "concat", {
  get: function() { trace += "wrong-get;"; throw "canonical Array concat getter"; }
});
var arrayCanonicalGetterResult = arrayWithCanonicalGetter.alias([4]);
check(arrayCanonicalGetterResult.length === 2 && arrayCanonicalGetterResult[0] === 3 && arrayCanonicalGetterResult[1] === 4 && trace === "", "Array alias never reads throwing concat getter");

var stringWithCanonicalGetter = new String("t");
stringWithCanonicalGetter.alias = String.prototype.concat;
Object.defineProperty(stringWithCanonicalGetter, "concat", {
  get: function() { trace += "wrong-get;"; throw "canonical String concat getter"; }
});
check(stringWithCanonicalGetter.alias("u") === "tu" && trace === "", "String alias never reads throwing concat getter");

var borrowedArrayPrototype = {
  alias: Array.prototype.concat,
  concat: function() { throw "canonical borrowed Array concat"; }
};
var borrowedArrayReceiver = Object.create(borrowedArrayPrototype);
var borrowedArrayResult = borrowedArrayReceiver.alias(7);
check(borrowedArrayResult.length === 2 && borrowedArrayResult[0] === borrowedArrayReceiver && borrowedArrayResult[1] === 7, "Array prototype alias preserves receiver identity");

var borrowedStringPrototype = {
  alias: String.prototype.concat,
  concat: function() { throw "canonical borrowed String concat"; }
};
var borrowedStringReceiver = Object.create(borrowedStringPrototype);
borrowedStringReceiver.toString = function() {
  "use strict";
  check(this === borrowedStringReceiver, "String prototype alias preserves receiver this");
  return "proto";
};
check(borrowedStringReceiver.alias("type") === "prototype", "String inherited transferred alias result");

var changedAlias = function() { throw "reacquired changed alias"; };
var arrayAliasDuringArgument = [10];
arrayAliasDuringArgument.alias = Array.prototype.concat;
arrayAliasDuringArgument.concat = function() { throw "canonical Array concat during argument"; };
var arrayAliasArgumentCalls = 0;
function replaceArrayAliasArgument() {
  arrayAliasArgumentCalls++;
  arrayAliasDuringArgument.alias = changedAlias;
  return [11];
}
var arrayAliasDuringArgumentResult = arrayAliasDuringArgument.alias(replaceArrayAliasArgument());
check(arrayAliasDuringArgumentResult.length === 2 && arrayAliasDuringArgumentResult[0] === 10 && arrayAliasDuringArgumentResult[1] === 11 && arrayAliasDuringArgument.alias === changedAlias && arrayAliasArgumentCalls === 1, "Array alias acquisition precedes argument mutation");

var stringAliasDuringArgument = new String("before");
stringAliasDuringArgument.alias = String.prototype.concat;
stringAliasDuringArgument.concat = function() { throw "canonical String concat during argument"; };
var stringAliasArgumentCalls = 0;
function replaceStringAliasArgument() {
  stringAliasArgumentCalls++;
  stringAliasDuringArgument.alias = changedAlias;
  return "after";
}
check(stringAliasDuringArgument.alias(replaceStringAliasArgument()) === "beforeafter" && stringAliasDuringArgument.alias === changedAlias && stringAliasArgumentCalls === 1, "String alias acquisition precedes argument mutation");

var arrayPrototypeBeforeArgument = {
  alias: Array.prototype.concat,
  concat: function() { throw "canonical Array prototype before argument"; }
};
var arrayPrototypeAfterArgument = {
  alias: changedAlias,
  concat: function() { throw "canonical Array prototype after argument"; }
};
var arrayPrototypeDuringArgument = Object.create(arrayPrototypeBeforeArgument);
trace = "";
function replaceArrayPrototypeArgument() {
  trace += "argument;";
  Object.setPrototypeOf(arrayPrototypeDuringArgument, arrayPrototypeAfterArgument);
  return 13;
}
var arrayPrototypeDuringArgumentResult = arrayPrototypeDuringArgument.alias(replaceArrayPrototypeArgument());
check(arrayPrototypeDuringArgumentResult.length === 2 && arrayPrototypeDuringArgumentResult[0] === arrayPrototypeDuringArgument && arrayPrototypeDuringArgumentResult[1] === 13 && arrayPrototypeDuringArgument.alias === changedAlias && trace === "argument;", "Array alias acquisition precedes prototype replacement");

var stringPrototypeBeforeArgument = {
  alias: String.prototype.concat,
  concat: function() { throw "canonical String prototype before argument"; }
};
var stringPrototypeAfterArgument = {
  alias: changedAlias,
  concat: function() { throw "canonical String prototype after argument"; }
};
var stringPrototypeDuringArgument = Object.create(stringPrototypeBeforeArgument);
stringPrototypeDuringArgument.toString = function() { trace += "receiver;"; return "before"; };
trace = "";
function replaceStringPrototypeArgument() {
  trace += "argument;";
  Object.setPrototypeOf(stringPrototypeDuringArgument, stringPrototypeAfterArgument);
  return "after";
}
check(stringPrototypeDuringArgument.alias(replaceStringPrototypeArgument()) === "beforeafter" && stringPrototypeDuringArgument.alias === changedAlias && trace === "argument;receiver;", "String alias acquisition precedes prototype replacement and coercion");

var aliasGetterHolder = {};
trace = "";
Object.defineProperty(aliasGetterHolder, "alias", {
  configurable: true,
  get: function() {
    "use strict";
    check(this === aliasGetterHolder, "alias getter preserves original receiver");
    trace += "get;";
    return String.prototype.concat;
  }
});
Object.defineProperty(aliasGetterHolder, "concat", {
  get: function() { throw "canonical concat getter during alias call"; }
});
aliasGetterHolder.toString = function() {
  "use strict";
  check(this === aliasGetterHolder, "acquired alias builtin preserves call receiver");
  trace += "receiver;";
  return "get";
};
function replaceGetterAliasArgument() {
  trace += "argument;";
  Object.defineProperty(aliasGetterHolder, "alias", {value: changedAlias});
  return "x";
}
check(aliasGetterHolder.alias(replaceGetterAliasArgument()) === "getx", "retain getter-acquired alias across argument mutation");
check(trace === "get;argument;receiver;", "alias getter precedes argument and builtin receiver coercion");

true;
