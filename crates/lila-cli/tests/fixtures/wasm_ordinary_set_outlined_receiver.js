let inheritedValue = 0;
let inheritedReceiver;
let prototype = {};
Object.defineProperty(prototype, "inherited", {
  set(value) {
    inheritedValue = value;
    inheritedReceiver = this;
  },
});
let receiver = Object.create(prototype);
receiver.inherited = 7;
if (inheritedValue !== 7 || inheritedReceiver !== receiver) {
  throw "inherited setter receiver";
}

let symbol = Symbol("receiver key");
if (!Reflect.set({}, symbol, 8, receiver) || receiver[symbol] !== 8) {
  throw "symbol receiver write";
}

function updatesMappedArgument(value) {
  Object.defineProperty(arguments, "0", {
    configurable: false,
    writable: true,
  });
  arguments[0] = 9;
  let descriptor = Object.getOwnPropertyDescriptor(arguments, "0");
  return value === 9 && descriptor.value === 9;
}
if (!updatesMappedArgument(1)) {
  throw "mapped arguments receiver write";
}

let otherGlobal = __lilaCreateRealm().global;
let foreignRangeError = false;
try {
  otherGlobal.Reflect.set([], "length", -1);
} catch (error) {
  foreignRangeError = error instanceof otherGlobal.RangeError
    && !(error instanceof RangeError);
}
if (!foreignRangeError) {
  throw "cross-realm array length error";
}

let target = {};
Object.defineProperty(target, "fixed", {
  configurable: false,
  writable: false,
  value: 10,
});
let proxy = new Proxy(target, {
  set() {
    return true;
  },
});
let invariantThrew = false;
try {
  proxy.fixed = 11;
} catch (error) {
  invariantThrew = error instanceof TypeError;
}
if (!invariantThrew || target.fixed !== 10) {
  throw "proxy set invariant";
}

true;
