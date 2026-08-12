let nonObjectError = null;
let unresolvableError = null;

class PrivateInBoundaries {
  #field;

  constructor() {
    try {
      #field in {} << 0;
    } catch (error) {
      nonObjectError = error;
    }
  }

  hasBrand(target) {
    return #field in target;
  }

  readUnresolvable() {
    try {
      #field in missingPrivateInTarget;
    } catch (error) {
      unresolvableError = error;
    }
  }
}

const instance = new PrivateInBoundaries();
if (nonObjectError.name !== "TypeError") throw "private-in non-object RHS";

instance.readUnresolvable();
if (unresolvableError.name !== "ReferenceError") {
  throw "private-in unresolvable RHS";
}

if (!instance.hasBrand(instance)) throw "private-in matching brand";
if (instance.hasBrand({})) throw "private-in plain object";
if (instance.hasBrand(new Proxy(instance, {}))) throw "private-in proxy brand";

true;
