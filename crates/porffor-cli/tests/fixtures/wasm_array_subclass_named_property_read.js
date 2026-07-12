class DefaultArray extends Array {
  has() { return true; }
  #privateMethod() { return 5; }
  #privateField = 7;
  callPrivateMethod() { return this.#privateMethod(); }
  readPrivateField() { return this.#privateField; }
  push() { return "overridden push"; }
  join() { return "overridden join"; }
  get accessorReceiver() { return this.ownNamed; }
}

class NestedDefaultArray extends DefaultArray {}
class InheritedPushArray extends Array {}
class BaseResult {
  constructor(value) {
    this.value = value;
    this.baseNewTarget = new.target;
    ({ leaked: true });
  }
}
class DerivedResult extends BaseResult {
  constructor() {
    super(2);
    this.derivedNewTarget = new.target;
  }
}
class PrimitiveReturn {
  constructor() {
    this.kept = true;
    return 1;
  }
}
class ExplicitObjectReturn {
  constructor() { return { explicit: true }; }
}

var direct = new DefaultArray();
var nested = new NestedDefaultArray();
var inheritedPush = new InheritedPushArray();
var baseResult = new BaseResult(1);
var derivedResult = new DerivedResult();
var primitiveReturn = new PrimitiveReturn();
var explicitObjectReturn = new ExplicitObjectReturn();
var computed = "has";
var computedPush = "push";
direct.ownNamed = "own";

if (!direct.has()) throw "direct";
if (!direct["has"]()) throw "bracket";
if (!direct[computed]()) throw "computed";
if (!nested.has()) throw "nested";
if (direct.callPrivateMethod() !== 5) throw "private method";
if (direct.readPrivateField() !== 7) throw "private field";
if (direct.push() !== "overridden push") throw "direct push";
if (direct[computedPush]() !== "overridden push") throw "computed push";
if (direct.join() !== "overridden join") throw "join";
if (inheritedPush.push(1) !== 1) throw "inherited push";
if (!direct.accessorReceiver) throw "accessor";
if (direct.ownNamed !== "own") throw "own";
if (Object.getPrototypeOf(direct) !== DefaultArray.prototype) throw "direct prototype";
if (Object.getPrototypeOf(nested) !== NestedDefaultArray.prototype) throw "nested prototype";
if (!(baseResult instanceof BaseResult) || baseResult.leaked === true) throw "base normal result";
if (baseResult.value !== 1 || baseResult.baseNewTarget !== BaseResult) throw "base new.target";
if (!(derivedResult instanceof DerivedResult) || derivedResult.value !== 2) throw "derived result";
if (derivedResult.baseNewTarget !== DerivedResult || derivedResult.derivedNewTarget !== DerivedResult) throw "derived new.target";
if (!(primitiveReturn instanceof PrimitiveReturn) || !primitiveReturn.kept) throw "primitive return";
if (!explicitObjectReturn.explicit || explicitObjectReturn instanceof ExplicitObjectReturn) throw "object return";
nested instanceof DefaultArray;
