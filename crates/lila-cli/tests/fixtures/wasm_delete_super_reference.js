class Base {}

let staticCaught = false;
class StaticDelete extends Base {
  remove() {
    delete super.missing;
  }
}
try {
  new StaticDelete().remove();
} catch (error) {
  staticCaught = error instanceof ReferenceError;
}

let keyEvaluations = 0;
let keyCoercions = 0;
let key = {
  toString() {
    keyCoercions++;
    return "missing";
  }
};
function evaluateKey() {
  keyEvaluations++;
  return key;
}

let computedCaught = false;
class ComputedDelete extends Base {
  remove() {
    delete super[evaluateKey()];
  }
}
try {
  new ComputedDelete().remove();
} catch (error) {
  computedCaught = error instanceof ReferenceError;
}

let marker = {};
function abruptKey() {
  throw marker;
}
let abruptKeyWon = false;
class AbruptKeyDelete extends Base {
  remove() {
    delete super[abruptKey()];
  }
}
try {
  new AbruptKeyDelete().remove();
} catch (error) {
  abruptKeyWon = error === marker;
}

let beforeSuperKeyEvaluations = 0;
function beforeSuperKey() {
  beforeSuperKeyEvaluations++;
  return "missing";
}
let beforeSuperCaught = false;
class BeforeSuperDelete extends Base {
  constructor() {
    try {
      delete super[beforeSuperKey()];
    } catch (error) {
      beforeSuperCaught = error instanceof ReferenceError;
    }
    super();
  }
}
new BeforeSuperDelete();

let nullBaseCaught = false;
class NullBaseDelete extends Base {
  remove() {
    delete super.missing;
  }
}
Object.setPrototypeOf(NullBaseDelete.prototype, null);
try {
  new NullBaseDelete().remove();
} catch (error) {
  nullBaseCaught = error instanceof ReferenceError;
}

let deleteTrapCalls = 0;
let trapTarget = { missing: 1 };
let trapBase = new Proxy(trapTarget, {
  deleteProperty() {
    deleteTrapCalls++;
    return true;
  }
});
let trapCaught = false;
class TrapDelete extends Base {
  remove() {
    delete super.missing;
  }
}
Object.setPrototypeOf(TrapDelete.prototype, trapBase);
try {
  new TrapDelete().remove();
} catch (error) {
  trapCaught = error instanceof ReferenceError;
}

staticCaught
  && computedCaught
  && keyEvaluations === 1
  && keyCoercions === 0
  && abruptKeyWon
  && beforeSuperCaught
  && beforeSuperKeyEvaluations === 0
  && nullBaseCaught
  && trapCaught
  && deleteTrapCalls === 0
  && trapTarget.missing === 1;
