var keyTrace = "";

function makePrototype(label) {
  return {
    get named() {
      return label + ":" + this.marker;
    },
    get computed() {
      return label + ":computed:" + this.marker;
    },
    invoke(suffix) {
      return label + ":" + this.marker + ":" + suffix;
    }
  };
}

function observeKey() {
  keyTrace += "key";
  return "computed";
}

var object = {
  namedArrow() {
    return () => super.named;
  },

  computedArrow() {
    return () => super[observeKey()];
  },

  nestedArrow() {
    return () => () => super.named;
  },

  callArrow() {
    return () => super.invoke("call");
  },

  parameterArrow(factory = () => super.named) {
    return factory;
  }
};

var prototypeA = makePrototype("A");
var prototypeB = makePrototype("B");
Object.setPrototypeOf(object, prototypeA);

var alien = { marker: "alien" };
var named = object.namedArrow.call(alien);
var computed = object.computedArrow.call(alien);
var nested = object.nestedArrow.call(alien)();
var call = object.callArrow.call(alien);
var parameter = object.parameterArrow.call(alien);

var firstNamed = named.call({ marker: "wrong" });
var firstComputed = computed.call({ marker: "wrong" });
var firstNested = nested.call({ marker: "wrong" });
var firstCall = call.call({ marker: "wrong" });
var firstParameter = parameter.call({ marker: "wrong" });

Object.setPrototypeOf(object, prototypeB);

var secondNamed = named();
var secondComputed = computed();
var secondNested = nested();
var secondCall = call();
var secondParameter = parameter();

firstNamed === "A:alien"
  && firstComputed === "A:computed:alien"
  && firstNested === "A:alien"
  && firstCall === "A:alien:call"
  && firstParameter === "A:alien"
  && secondNamed === "B:alien"
  && secondComputed === "B:computed:alien"
  && secondNested === "B:alien"
  && secondCall === "B:alien:call"
  && secondParameter === "B:alien"
  && keyTrace === "keykey";
