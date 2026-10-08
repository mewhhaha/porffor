class ExactPrivateField {
  #field;

  assign(source) {
    ({ ...this.#field } = source);
  }
}

let exactTypeError = false;
try {
  ExactPrivateField.prototype.assign.call({}, {});
} catch (error) {
  exactTypeError = error instanceof TypeError;
}
if (!exactTypeError) throw "privatefieldset-typeerror-11";

class PrivateRestTarget {
  #field = {};

  assign(source) {
    ({ ...this.#field } = source);
    return this.#field;
  }
}

const instance = new PrivateRestTarget();
const copied = instance.assign({ first: 1, second: 2 });
if (copied.first !== 1 || copied.second !== 2) throw "branded private rest target";

let rest;
const source = { kept: 3 };
const assignmentResult = ({ ...rest } = source);
if (assignmentResult !== source || rest.kept !== 3) throw "assignment result and rest copy";

let computedOrder = "";
let defaulted;
function computedKey() {
  computedOrder += "k";
  return "missing";
}
function defaultValue() {
  computedOrder += "d";
  return 9;
}
({ [computedKey()]: defaulted } = {});
({ missing: defaulted = defaultValue() } = {});
if (computedOrder !== "kd" || defaulted !== 9) throw "computed key and default";

let setterValue = 0;
const setterTarget = {
  set value(next) {
    setterValue = next;
  },
};
({ value: setterTarget.value } = { value: 11 });
if (setterValue !== 11) throw "property assignment target";

let exclusionOrder = "";
const exclusionSource = new Proxy(
  { omitted: 1, kept: 2 },
  {
    ownKeys(target) {
      exclusionOrder += "o";
      return ["omitted", "kept"];
    },
    getOwnPropertyDescriptor(target, key) {
      exclusionOrder += "d" + key;
      return { value: target[key], writable: true, enumerable: true, configurable: true };
    },
    get(target, key) {
      exclusionOrder += "g" + key;
      return target[key];
    },
  },
);
let omitted;
({ omitted, ...rest } = exclusionSource);
if (omitted !== 1 || rest.kept !== 2) throw "proxy exclusion values";
if (exclusionOrder !== "gomittedodkeptgkept") throw "proxy exclusion order";

let hiddenGet = false;
const hiddenSource = new Proxy(
  {},
  {
    ownKeys() {
      return ["hidden"];
    },
    getOwnPropertyDescriptor() {
      return { value: 1, writable: true, enumerable: false, configurable: true };
    },
    get() {
      hiddenGet = true;
      return 1;
    },
  },
);
({ ...rest } = hiddenSource);
if (hiddenGet || rest.hidden !== undefined) throw "non-enumerable rest key";

let delayedOrder = "";
const delayedSource = new Proxy(
  { visible: 4 },
  {
    ownKeys() {
      delayedOrder += "o";
      return ["visible"];
    },
    getOwnPropertyDescriptor(target, key) {
      delayedOrder += "d";
      return { value: target[key], writable: true, enumerable: true, configurable: true };
    },
    get(target, key) {
      delayedOrder += "g";
      return target[key];
    },
  },
);
let delayedTypeError = false;
try {
  PrivateRestTarget.prototype.assign.call({}, delayedSource);
} catch (error) {
  delayedTypeError = error instanceof TypeError;
}
if (!delayedTypeError || delayedOrder !== "odg") throw "private brand after copy";

const ownKeysError = {};
const abruptSource = new Proxy(
  {},
  {
    ownKeys() {
      throw ownKeysError;
    },
  },
);
let abruptResult;
try {
  PrivateRestTarget.prototype.assign.call({}, abruptSource);
} catch (error) {
  abruptResult = error;
}
if (abruptResult !== ownKeysError) throw "ownKeys abrupt before private brand";

const symbolKey = Symbol("rest");
const symbolSource = {};
Object.defineProperty(symbolSource, symbolKey, {
  value: 13,
  writable: true,
  enumerable: true,
  configurable: true,
});
({ ...rest } = symbolSource);
if (rest[symbolKey] !== 13) throw "symbol rest key";

function checkSymbolRestExclusion(assignment) {
  const excludedSymbol = Symbol("shared rest description");
  const includedSymbol = Symbol("shared rest description");
  let keyCalls = 0;
  let trace = "";
  function exclusionKey() {
    keyCalls += 1;
    trace += "k";
    return excludedSymbol;
  }
  const symbolProxy = new Proxy({}, {
    ownKeys() {
      trace += "o";
      return [
        excludedSymbol, "excludedString", "0",
        includedSymbol, "includedString", "1",
      ];
    },
    getOwnPropertyDescriptor(target, key) {
      if (key === excludedSymbol || key === "excludedString" || key === "0") {
        throw "excluded rest descriptor trap";
      }
      let value;
      if (key === includedSymbol) {
        trace += "dy";
        value = 21;
      } else if (key === "includedString") {
        trace += "dS";
        value = 22;
      } else if (key === "1") {
        trace += "d1";
        value = 23;
      } else {
        throw "unexpected rest descriptor key";
      }
      return { value, writable: true, enumerable: true, configurable: true };
    },
    get(target, key) {
      if (key === excludedSymbol) {
        trace += "x";
        return 11;
      }
      if (key === "excludedString") {
        trace += "s";
        return 12;
      }
      if (key === "0") {
        trace += "0";
        return 13;
      }
      if (key === includedSymbol) {
        trace += "gy";
        return 21;
      }
      if (key === "includedString") {
        trace += "gS";
        return 22;
      }
      if (key === "1") {
        trace += "g1";
        return 23;
      }
      throw "unexpected rest get key";
    },
  });
  let copiedRest;
  if (assignment) {
    let selectedSymbol;
    let selectedString;
    let selectedIndex;
    ({
      [exclusionKey()]: selectedSymbol,
      excludedString: selectedString,
      0: selectedIndex,
      ...copiedRest
    } = symbolProxy);
    if (selectedSymbol !== 11 || selectedString !== 12 || selectedIndex !== 13) {
      throw "symbol rest assignment binding values";
    }
  } else {
    const {
      [exclusionKey()]: selectedSymbol,
      excludedString: selectedString,
      0: selectedIndex,
      ...boundRest
    } = symbolProxy;
    copiedRest = boundRest;
    if (selectedSymbol !== 11 || selectedString !== 12 || selectedIndex !== 13) {
      throw "symbol rest declaration binding values";
    }
  }
  if (keyCalls !== 1 || trace !== "kxs0odygydSgSd1g1") {
    throw "symbol rest key evaluation and copy order";
  }
  if (
    copiedRest[includedSymbol] !== 21 ||
    copiedRest.includedString !== 22 ||
    copiedRest[1] !== 23
  ) {
    throw "distinct symbol and mixed rest keys";
  }
  if (
    copiedRest[excludedSymbol] !== undefined ||
    copiedRest.excludedString !== undefined ||
    copiedRest[0] !== undefined
  ) {
    throw "excluded rest keys copied";
  }
  if (Reflect.ownKeys(copiedRest).length !== 3) throw "symbol rest own key count";
}

checkSymbolRestExclusion(false);
checkSymbolRestExclusion(true);

let nullThrows = false;
try {
  ({ ...rest } = null);
} catch (error) {
  nullThrows = error instanceof TypeError;
}
let undefinedThrows = false;
try {
  ({ ...rest } = undefined);
} catch (error) {
  undefinedThrows = error instanceof TypeError;
}
if (!nullThrows || !undefinedThrows) throw "RequireObjectCoercible";

({ ...rest } = "ab");
if (rest[0] !== "a" || rest[1] !== "b") throw "string rest source";

true;
