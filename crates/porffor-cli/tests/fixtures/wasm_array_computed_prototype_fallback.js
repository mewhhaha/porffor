function read(value, key) {
  return value[key];
}

var seen = false;
Object.defineProperty(Array.prototype, "0", {
  configurable: true,
  get: function () {
    seen = this === hole;
    return 99;
  }
});

var hole = new Array(1);
var inheritedString = read(hole, "0");
var inheritedNumeric = read(hole, 0);
Object.defineProperty(hole, "0", { value: 7, configurable: true });
var own = read(hole, "0");
delete Array.prototype[0];

Array.prototype.myproperty = 1;
var named = new Array(0);
var inheritedNamed = named.myproperty;
var inheritedComputedNamed = named["myproperty"];
var arrayPrototypeNamed = Array.prototype.myproperty;
delete Array.prototype.myproperty;

function IntermediateArrayPrototype() {}
IntermediateArrayPrototype.prototype = new Array(0);
var inheritedAccessorReceiver;
Object.defineProperty(IntermediateArrayPrototype.prototype, "namedAccessor", {
  configurable: true,
  get: function () {
    inheritedAccessorReceiver = this;
    return 17;
  }
});
var intermediateReceiver = new IntermediateArrayPrototype();
var inheritedAccessor = read(intermediateReceiver, "namedAccessor");

var originalJoin = Array.prototype.join;
var ordinaryJoin = [1, 2].join(",");
var extractedJoin = originalJoin.call([1, 2], "-");
var borrowedJoin = originalJoin.call({ 0: "a", length: 1 }, "-");

var zeroJoinOrder = "";
var zeroJoinReceiver = {};
Object.defineProperty(zeroJoinReceiver, "length", {
  get: function () {
    zeroJoinOrder += "length;";
    return 0;
  }
});
var zeroJoinSeparator = {
  toString: function () {
    zeroJoinOrder += "separator;";
    return "-";
  }
};
var zeroJoin = originalJoin.call(zeroJoinReceiver, zeroJoinSeparator);

var genericJoinOrder = "";
var genericJoinReceiver = {};
Object.defineProperty(genericJoinReceiver, "length", {
  get: function () {
    genericJoinOrder += "length;";
    return 1;
  }
});
Object.defineProperty(genericJoinReceiver, "0", {
  get: function () {
    genericJoinOrder += "element;";
    return "a";
  }
});
var genericJoinSeparator = {
  toString: function () {
    genericJoinOrder += "separator;";
    return "-";
  }
};
var genericReceiverEvaluations = 0;
function makeGenericJoinReceiver() {
  genericReceiverEvaluations += 1;
  return genericJoinReceiver;
}
var orderedGenericJoin = originalJoin.call(
  makeGenericJoinReceiver(),
  genericJoinSeparator
);
var arrayPrototypeAlias = Array.prototype;
arrayPrototypeAlias.join = function () {
  return "alias";
};
var aliasJoin = named.join(",");
var directPrototypeJoin = Array.prototype.join(",");
arrayPrototypeAlias.join = originalJoin;

var lateAlias;
lateAlias = Array.prototype;
Object.defineProperties(lateAlias, {
  join: {
    configurable: true,
    value: function () {
      return "late-alias";
    }
  }
});
var lateAliasJoin = named.join(",");
Object.defineProperty(Array.prototype, "join", {
  configurable: true,
  writable: true,
  value: originalJoin
});

var computedAlias = Array["prototype"];
computedAlias.join = function () {
  return "computed-alias";
};
var computedAliasJoin = named.join(",");
Object.defineProperty(Array.prototype, "join", {
  configurable: true,
  writable: true,
  value: originalJoin
});

function replaceArrayJoin(prototype, result) {
  prototype.join = function () {
    return result;
  };
}
replaceArrayJoin(Array.prototype, "helper-alias");
var helperAliasJoin = named.join(",");
Object.defineProperty(Array.prototype, "join", {
  configurable: true,
  writable: true,
  value: originalJoin
});

var reflectedAlias = Object.getPrototypeOf([]);
reflectedAlias.join = function () {
  return "reflected-alias";
};
var reflectedAliasJoin = named.join(",");
Object.defineProperty(Array.prototype, "join", {
  configurable: true,
  writable: true,
  value: originalJoin
});

var hiddenConstructor = globalThis["Ar" + "ray"];
var hiddenPrototype = hiddenConstructor["proto" + "type"];
hiddenPrototype.join = function () {
  return "hidden-alias";
};
var hiddenAliasJoin = named.join(",");
Object.defineProperty(Array.prototype, "join", {
  configurable: true,
  writable: true,
  value: originalJoin
});

Object.defineProperty(Array.prototype, "join", {
  configurable: true,
  value: function () {
    return "define";
  }
});
var definePropertyJoin = named.join(",");
Object.defineProperty(Array.prototype, "join", {
  configurable: true,
  writable: true,
  value: originalJoin
});

Reflect.defineProperty(Array.prototype, "join", {
  configurable: true,
  value: function () {
    return "reflect";
  }
});
var reflectDefinePropertyJoin = named.join(",");
var reflectDeleteJoin = Reflect.deleteProperty(Array.prototype, "join");
var reflectDeletedJoinThrew = false;
try {
  named.join(",");
} catch (error) {
  reflectDeletedJoinThrew = error instanceof TypeError;
}
Object.defineProperty(Array.prototype, "join", {
  configurable: true,
  writable: true,
  value: originalJoin
});



delete Array.prototype.join;
var deletedJoinThrew = false;
try {
  named.join(",");
} catch (error) {
  deletedJoinThrew = error instanceof TypeError;
}
Object.defineProperty(Array.prototype, "join", {
  configurable: true,
  writable: true,
  value: originalJoin
});

Reflect.set(Array.prototype, "join", function () {
  return "reflect-set";
});
var reflectSetPropertyJoin = named.join(",");
Object.defineProperty(Array.prototype, "join", {
  configurable: true,
  writable: true,
  value: originalJoin
});

Object.defineProperties(Array.prototype, {
  join: {
    configurable: true,
    value: function () {
      return "define-properties";
    }
  }
});
var definePropertiesJoin = named.join(",");
Object.defineProperty(Array.prototype, "join", {
  configurable: true,
  writable: true,
  value: originalJoin
});

var originalToString = Array.prototype.toString;
Array.prototype.toString = Object.prototype.toString;
var inheritedToString = named.toString();
Array.prototype.toString = originalToString;

!!(inheritedString === 99
  && inheritedNumeric === 99
  && own === 7
  && seen
  && arrayPrototypeNamed === 1
  && inheritedNamed === 1
  && inheritedComputedNamed === 1
  && inheritedAccessor === 17
  && inheritedAccessorReceiver === intermediateReceiver
  && ordinaryJoin === "1,2"
  && extractedJoin === "1-2"
  && borrowedJoin === "a"
  && zeroJoin === ""
  && zeroJoinOrder === "length;separator;"
  && orderedGenericJoin === "a"
  && genericJoinOrder === "length;separator;element;"
  && genericReceiverEvaluations === 1
  && aliasJoin === "alias"
  && directPrototypeJoin === "alias"
  && lateAliasJoin === "late-alias"
  && computedAliasJoin === "computed-alias"
  && helperAliasJoin === "helper-alias"
  && reflectedAliasJoin === "reflected-alias"
  && hiddenAliasJoin === "hidden-alias"
  && definePropertyJoin === "define"
  && reflectDefinePropertyJoin === "reflect"
  && reflectSetPropertyJoin === "reflect-set"
  && definePropertiesJoin === "define-properties"
  && reflectDeleteJoin
  && reflectDeletedJoinThrew
  && deletedJoinThrew
  && inheritedToString === "[object Array]");
