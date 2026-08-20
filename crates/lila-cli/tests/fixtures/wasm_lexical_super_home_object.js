class HomeBase {
  method() { return this.marker + ":base"; }
  get value() { return this.marker + ":get-base"; }
}

class AlienBase {
  method() { return this.marker + ":alien"; }
  get value() { return this.marker + ":get-alien"; }
}

class HomeDerived extends HomeBase {
  direct() { return super.method(); }
  makeArrow() { return () => super.method(); }
  makeNestedArrow() { return () => () => super.method(); }
  makeComputedArrow(key) { return () => super[key](); }
  makeGetterArrow() { return () => super.value; }
}

let alienReceiver = Object.create(AlienBase.prototype);
alienReceiver.marker = "receiver";

if (HomeDerived.prototype.direct.call(alienReceiver) !== "receiver:base") throw "case-1";

let arrow = HomeDerived.prototype.makeArrow.call(alienReceiver);
if (arrow.call({ marker: "wrong" }) !== "receiver:base") throw "case-2";

let nestedArrow = HomeDerived.prototype.makeNestedArrow.call(alienReceiver)();
if (nestedArrow() !== "receiver:base") throw "case-3";

let computedArrow = HomeDerived.prototype.makeComputedArrow.call(alienReceiver, "method");
if (computedArrow() !== "receiver:base") throw "case-4";

let getterArrow = HomeDerived.prototype.makeGetterArrow.call(alienReceiver);
if (getterArrow() !== "receiver:get-base") throw "case-5";

class StaticBase {
  static method() { return this.name + ":static-base"; }
}

class StaticAlienBase {
  static method() { return this.name + ":static-alien"; }
}

class StaticAlienDerived extends StaticAlienBase {}

class StaticDerived extends StaticBase {
  static direct() { return super.method(); }
  static makeArrow() { return () => super.method(); }
}

if (StaticDerived.direct.call(StaticAlienDerived) !== "StaticAlienDerived:static-base") throw "case-6";
let staticArrow = StaticDerived.makeArrow.call(StaticAlienDerived);
if (staticArrow.call(StaticDerived) !== "StaticAlienDerived:static-base") throw "case-7";

class MutableBaseOne {
  method() { return "one"; }
}
class MutableBaseTwo {
  method() { return "two"; }
}
class MutableDerived extends MutableBaseOne {
  direct() { return super.method(); }
  makeArrow() { return () => super.method(); }
}

let mutableValue = new MutableDerived();
let mutableArrow = mutableValue.makeArrow();
Object.setPrototypeOf(MutableDerived.prototype, MutableBaseTwo.prototype);
if (mutableValue.direct() !== "two") throw "case-8";
if (mutableArrow() !== "two") throw "case-9";

let mutableStaticArrow = StaticDerived.makeArrow();
Object.setPrototypeOf(StaticDerived, StaticAlienBase);
if (StaticDerived.direct() !== "StaticDerived:static-alien") throw "case-10";
if (mutableStaticArrow() !== "StaticDerived:static-alien") throw "case-11";

true;
