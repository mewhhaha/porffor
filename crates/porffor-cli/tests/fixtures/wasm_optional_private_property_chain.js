let ok = true;
let baseCalls = 0;
let getterCalls = 0;

function wrapped(value) {
  baseCalls += 1;
  return value;
}

class First {
  #field = "first";

  get #value() {
    getterCalls += 1;
    return this.#field;
  }

  #method() {
    return this;
  }

  readWrapped(value) {
    return value?.c.#field;
  }

  readDirect(value) {
    return value?.#field;
  }

  readGetter(value) {
    return value?.#value;
  }

  callMethod(value) {
    return value?.#method();
  }
}

class Second {
  #field = "second";

  readDirect(value) {
    return value?.#field;
  }
}

const first = new First();
const second = new Second();

ok = ok && first.readWrapped(wrapped({ c: first })) === "first";
ok = ok && baseCalls === 1;
ok = ok && first.readWrapped(null) === undefined;
ok = ok && first.readWrapped(undefined) === undefined;
ok = ok && first.readDirect(null) === undefined;
ok = ok && first.readDirect(undefined) === undefined;

let nullishTailThrew = false;
try {
  first.readWrapped({ c: null });
} catch (error) {
  nullishTailThrew = error instanceof TypeError;
}
ok = ok && nullishTailThrew;

let unbrandedThrew = false;
try {
  first.readWrapped({ c: {} });
} catch (error) {
  unbrandedThrew = error instanceof TypeError;
}
ok = ok && unbrandedThrew;

ok = ok && first.readGetter(first) === "first";
ok = ok && getterCalls === 1;
ok = ok && first.callMethod(first) === first;

let firstRejectedSecond = false;
try {
  first.readDirect(second);
} catch (error) {
  firstRejectedSecond = error instanceof TypeError;
}
ok = ok && firstRejectedSecond;

let secondRejectedFirst = false;
try {
  second.readDirect(first);
} catch (error) {
  secondRejectedFirst = error instanceof TypeError;
}
ok = ok && secondRejectedFirst;

true && ok;
