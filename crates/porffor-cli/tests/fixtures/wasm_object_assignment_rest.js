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
