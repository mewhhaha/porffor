let other = __lilaCreateRealm().global;

let constructors = [
  ["EvalError", EvalError, other.EvalError, other.EvalError.prototype],
  ["RangeError", RangeError, other.RangeError, other.RangeError.prototype],
  ["ReferenceError", ReferenceError, other.ReferenceError, other.ReferenceError.prototype],
  ["SyntaxError", SyntaxError, other.SyntaxError, other.SyntaxError.prototype],
  ["TypeError", TypeError, other.TypeError, other.TypeError.prototype],
  ["URIError", URIError, other.URIError, other.URIError.prototype],
];

let primitivePrototypes = [undefined, null, false, "str", Symbol(), 0];

for (let i = 0; i < constructors.length; i = i + 1) {
  let name = constructors[i][0];
  let localConstructor = constructors[i][1];
  let otherConstructor = constructors[i][2];
  let otherPrototype = constructors[i][3];

  for (let j = 0; j < primitivePrototypes.length; j = j + 1) {
    let newTarget = new other.Function();
    newTarget.prototype = primitivePrototypes[j];
    let cause = {};
    let value = Reflect.construct(
      localConstructor,
      [name + " message", { cause: cause }],
      newTarget,
    );
    if (Object.getPrototypeOf(value) !== otherPrototype) {
      throw name + " primitive fallback realm";
    }
    if (!(Error.isError(value) &&
          value.message === name + " message" &&
          value.cause === cause)) {
      throw name + " primitive fallback message/options";
    }
  }

  let activeCause = {};
  let active = otherConstructor("active", { cause: activeCause });
  if (!(Object.getPrototypeOf(active) === otherPrototype &&
        Error.isError(active) &&
        active.message === "active" &&
        active.cause === activeCause)) {
    throw name + " active function realm/options";
  }

  let localActiveCause = {};
  let localActive = localConstructor("local active", { cause: localActiveCause });
  if (!(Object.getPrototypeOf(localActive) === localConstructor.prototype &&
        Error.isError(localActive) &&
        localActive.message === "local active" &&
        localActive.cause === localActiveCause)) {
    throw name + " entry active function/options";
  }

  let customPrototypes = [
    { custom: true },
    function () {},
    [],
    (function () { return arguments; })(),
  ];
  for (let j = 0; j < customPrototypes.length; j = j + 1) {
    let newTarget = new other.Function();
    let customPrototype = customPrototypes[j];
    newTarget.prototype = customPrototype;
    let cause = {};
    let value = Reflect.construct(
      localConstructor,
      ["custom", { cause: cause }],
      newTarget,
    );
    let actualPrototype = Object.getPrototypeOf(value);
    if (!(actualPrototype === customPrototype &&
          Error.isError(value) &&
          value.message === "custom" &&
          value.cause === cause)) {
      throw name + " custom prototype tag/identity";
    }
    if (j === 0 &&
        (typeof actualPrototype !== "object" || Array.isArray(actualPrototype))) {
      throw name + " Object custom prototype tag";
    }
    if (j === 1 && typeof actualPrototype !== "function") {
      throw name + " Function custom prototype tag";
    }
    if (j === 2 && !Array.isArray(actualPrototype)) {
      throw name + " Array custom prototype tag";
    }
    if (j === 3 &&
        Object.prototype.toString.call(actualPrototype) !== "[object Arguments]") {
      throw name + " Arguments custom prototype tag";
    }
  }

  let order = [];
  let orderedPrototype = {};
  let orderedNewTarget = new Proxy(new other.Function(), {
    get: function (target, key, receiver) {
      if (key === "prototype") {
        order.push("prototype");
        return orderedPrototype;
      }
      return Reflect.get(target, key, receiver);
    },
  });
  let orderedCause = {};
  let orderedMessage = {
    toString: function () {
      order.push("message");
      return "ordered";
    },
  };
  let orderedOptions = {
    get cause() {
      order.push("cause");
      return orderedCause;
    },
  };
  let ordered = Reflect.construct(
    localConstructor,
    [orderedMessage, orderedOptions],
    orderedNewTarget,
  );
  if (!(order.length === 3 &&
        order[0] === "prototype" &&
        order[1] === "message" &&
        order[2] === "cause" &&
        Object.getPrototypeOf(ordered) === orderedPrototype &&
        ordered.message === "ordered" &&
        ordered.cause === orderedCause)) {
    throw name + " prototype/message/options order";
  }

  let reads = 0;
  let observedNewTarget = new Proxy(new other.Function(), {
    get: function (target, key, receiver) {
      if (key === "prototype") {
        reads = reads + 1;
        return null;
      }
      return Reflect.get(target, key, receiver);
    },
  });
  let observed = Reflect.construct(localConstructor, [], observedNewTarget);
  if (!(reads === 1 && Object.getPrototypeOf(observed) === otherPrototype)) {
    throw name + " sole prototype Get";
  }

  let abruptMarker = {};
  let abruptReads = 0;
  let abruptNewTarget = new Proxy(new other.Function(), {
    get: function (target, key, receiver) {
      if (key === "prototype") {
        abruptReads = abruptReads + 1;
        throw abruptMarker;
      }
      return Reflect.get(target, key, receiver);
    },
  });
  let abruptThrew = false;
  try {
    Reflect.construct(localConstructor, [], abruptNewTarget);
  } catch (error) {
    abruptThrew = error === abruptMarker;
  }
  if (!(abruptThrew && abruptReads === 1)) {
    throw name + " prototype Get abrupt completion";
  }

  let objectPrototype = [];
  let objectRevocable;
  objectRevocable = Proxy.revocable(new other.Function(), {
    get: function (target, key, receiver) {
      if (key === "prototype") {
        objectRevocable.revoke();
        return objectPrototype;
      }
      return Reflect.get(target, key, receiver);
    },
  });
  let objectResult = Reflect.construct(
    localConstructor,
    [],
    objectRevocable.proxy,
  );
  if (Object.getPrototypeOf(objectResult) !== objectPrototype) {
    throw name + " object prototype must not resolve revoked realm";
  }

  let primitiveRevocable;
  primitiveRevocable = Proxy.revocable(new other.Function(), {
    get: function (target, key, receiver) {
      if (key === "prototype") {
        primitiveRevocable.revoke();
        return null;
      }
      return Reflect.get(target, key, receiver);
    },
  });
  let revocationThrew = false;
  try {
    Reflect.construct(localConstructor, [], primitiveRevocable.proxy);
  } catch (error) {
    revocationThrew = error instanceof TypeError;
  }
  if (!revocationThrew) throw name + " revoked function realm fallback";
}

262;
