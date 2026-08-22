var keyTrace = "";
var parameterTrace = "";

function computedKey(label, name) {
  return {
    toString: function() {
      keyTrace += label;
      return name;
    }
  };
}

function makePrototype(label) {
  var prototype = { label: label };
  prototype.invoke = function(suffix) {
    return this.marker + ":" + label + ":" + suffix;
  };
  Object.defineProperty(prototype, "observed", {
    get: function() {
      return label + ":" + this.marker;
    },
    set: function(value) {
      this.setterResult = label + ":" + value;
    },
    configurable: true
  });
  Object.defineProperty(prototype, "parameterValue", {
    get: function() {
      parameterTrace += "get,";
      return label + "-parameter:" + this.marker;
    },
    configurable: true
  });
  return prototype;
}

var literal = {
  marker: "literal",

  method(suffix) {
    return super.invoke(suffix);
  },

  parameterMethod(value = super.parameterValue) {
    parameterTrace += "body";
    return value;
  },

  get namedAccessor() {
    return super.observed;
  },

  set namedAccessor(value) {
    super.observed = value;
  },

  [computedKey("m", "computedMethod")](suffix) {
    return super.invoke(suffix);
  },

  staticBetween() {
    return super.observed;
  },

  get [computedKey("g", "computedAccessor")]() {
    return super.observed;
  },

  set [computedKey("s", "computedAccessor")](value) {
    super.observed = value;
  }
};

var prototypeA = makePrototype("A");
var prototypeB = makePrototype("B");
Object.setPrototypeOf(literal, prototypeA);

var method = literal.method;
var parameterMethod = literal.parameterMethod;
var computedMethod = literal.computedMethod;
var staticBetween = literal.staticBetween;
var namedAccessor = Object.getOwnPropertyDescriptor(literal, "namedAccessor");
var computedAccessor = Object.getOwnPropertyDescriptor(literal, "computedAccessor");
var alien = { marker: "alien" };

var firstMethod = method.call(alien, "first");
parameterTrace = "";
var firstParameter = parameterMethod.call(alien);
var firstParameterTrace = parameterTrace;
var firstNamedGet = namedAccessor.get.call(alien);
namedAccessor.set.call(alien, 1);
var firstNamedSet = alien.setterResult;
var firstComputedMethod = computedMethod.call(alien, "computed");
var firstComputedGet = computedAccessor.get.call(alien);
computedAccessor.set.call(alien, 2);
var firstComputedSet = alien.setterResult;
var firstStatic = staticBetween.call(alien);

Object.setPrototypeOf(literal, prototypeB);

var secondMethod = method.call(alien, "second");
parameterTrace = "";
var secondParameter = parameterMethod.call(alien);
var secondParameterTrace = parameterTrace;
var secondNamedGet = namedAccessor.get.call(alien);
namedAccessor.set.call(alien, 3);
var secondNamedSet = alien.setterResult;
var secondComputedMethod = computedMethod.call(alien, "computed");
var secondComputedGet = computedAccessor.get.call(alien);
computedAccessor.set.call(alien, 4);
var secondComputedSet = alien.setterResult;
var secondStatic = staticBetween.call(alien);

function isNonConstructable(fn) {
  try {
    new fn();
  } catch (error) {
    return error instanceof TypeError;
  }
  return false;
}

keyTrace === "mgs"
  && firstMethod === "alien:A:first"
  && firstParameter === "A-parameter:alien"
  && firstParameterTrace === "get,body"
  && firstNamedGet === "A:alien"
  && firstNamedSet === "A:1"
  && firstComputedMethod === "alien:A:computed"
  && firstComputedGet === "A:alien"
  && firstComputedSet === "A:2"
  && firstStatic === "A:alien"
  && secondMethod === "alien:B:second"
  && secondParameter === "B-parameter:alien"
  && secondParameterTrace === "get,body"
  && secondNamedGet === "B:alien"
  && secondNamedSet === "B:3"
  && secondComputedMethod === "alien:B:computed"
  && secondComputedGet === "B:alien"
  && secondComputedSet === "B:4"
  && secondStatic === "B:alien"
  && Object.getPrototypeOf(literal) === prototypeB
  && isNonConstructable(method)
  && isNonConstructable(namedAccessor.get)
  && isNonConstructable(namedAccessor.set)
  && isNonConstructable(computedMethod)
  && isNonConstructable(computedAccessor.get)
  && isNonConstructable(computedAccessor.set);
