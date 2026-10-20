class ValueBase {
  constructor(value) {
    this.value = value;
    this.observedNewTarget = new.target;
  }
}
class ValueDirect extends ValueBase {}
class ValueDeeper extends ValueDirect {}

let valueDirect = new ValueDirect(11);
let valueDeeper = new ValueDeeper(22);
if (!(valueDirect.value === 11 &&
      Object.getPrototypeOf(valueDirect) === ValueDirect.prototype &&
      valueDirect.observedNewTarget === ValueDirect)) throw "case-1";
if (!(valueDeeper.value === 22 &&
      Object.getPrototypeOf(valueDeeper) === ValueDeeper.prototype &&
      valueDeeper.observedNewTarget === ValueDeeper)) throw "case-2";

let orderedBodyEvent = false;
let orderedGets = 0;
let orderedPrototype = { ordered: true };
let orderedNewTarget = new Proxy(function () {}, {
  get(target, key, receiver) {
    if (key === "prototype") {
      orderedGets++;
      if (!orderedBodyEvent) throw "case-3-order";
      return orderedPrototype;
    }
    return Reflect.get(target, key, receiver);
  }
});
class OrderedBase {}
class OrderedDerived extends OrderedBase {
  constructor() {
    orderedBodyEvent = true;
    super();
  }
}
let orderedValue = Reflect.construct(OrderedDerived, [], orderedNewTarget);
if (!(orderedGets === 1 && Object.getPrototypeOf(orderedValue) === orderedPrototype)) throw "case-3";

let returnOnlyGets = 0;
let returnOnlyNewTarget = new Proxy(function () {}, {
  get(target, key, receiver) {
    if (key === "prototype") returnOnlyGets++;
    return Reflect.get(target, key, receiver);
  }
});
class ReturnOnlyBase {}
class ReturnOnlyDerived extends ReturnOnlyBase {
  constructor() { return { returned: true }; }
}
let returnOnlyValue = Reflect.construct(ReturnOnlyDerived, [], returnOnlyNewTarget);
if (!(returnOnlyValue.returned === true && returnOnlyGets === 0)) throw "case-4";

let cachedBaseCalled = "";
class CachedBaseA { constructor() { cachedBaseCalled = "A"; } }
class CachedBaseB { constructor() { cachedBaseCalled = "B"; } }
class CachedDerived extends CachedBaseA {
  constructor() {
    super((Object.setPrototypeOf(CachedDerived, CachedBaseB), 0));
  }
}
new CachedDerived();
if (cachedBaseCalled !== "A") throw "case-5";

let repeatedBaseCalls = 0;
let repeatedCaught = false;
class RepeatedBase { constructor() { repeatedBaseCalls++; } }
class RepeatedDerived extends RepeatedBase {
  constructor() {
    super();
    let firstThis = this;
    try { super(); } catch (error) { repeatedCaught = error instanceof ReferenceError; }
    this.preservedFirstThis = this === firstThis;
  }
}
let repeatedValue = new RepeatedDerived();
if (!(repeatedBaseCalls === 2 && repeatedCaught && repeatedValue.preservedFirstThis)) throw "case-6";

let arrowBeforeCaught = false;
class ArrowBase {}
class ArrowDerived extends ArrowBase {
  constructor() {
    let readThis = () => this;
    try { readThis(); } catch (error) { arrowBeforeCaught = error instanceof ReferenceError; }
    super();
    let initializedThis = this;
    if (readThis() !== initializedThis) throw "case-7-same";
    return readThis();
  }
}
let arrowValue = new ArrowDerived();
if (!(arrowBeforeCaught && Object.getPrototypeOf(arrowValue) === ArrowDerived.prototype)) throw "case-7";

let escapedReadThis;
class EscapedBase {}
class EscapedDerived extends EscapedBase {
  constructor() {
    escapedReadThis = () => this;
    return { escaped: true };
  }
}
let escapedValue = new EscapedDerived();
let escapedCaught = false;
try { escapedReadThis(); } catch (error) { escapedCaught = error instanceof ReferenceError; }
if (!(escapedValue.escaped === true && escapedCaught)) throw "case-8";

let immediateBaseCalls = 0;
class ImmediateBase { constructor() { immediateBaseCalls++; } }
class ImmediateDerived extends ImmediateBase {
  constructor() { (() => super())(); }
}
new ImmediateDerived();
if (immediateBaseCalls !== 1) throw "case-9";

let nestedBaseCalls = 0;
class NestedBase { constructor() { nestedBaseCalls++; } }
class NestedDerived extends NestedBase {
  constructor() { (() => () => super())()(); }
}
new NestedDerived();
if (nestedBaseCalls !== 1) throw "case-10";

let reboundBaseCalls = 0;
let reboundCaught = false;
class ReboundBase { constructor() { reboundBaseCalls++; } }
class ReboundDerived extends ReboundBase {
  constructor() {
    let callSuper = () => super();
    callSuper();
    try { callSuper(); } catch (error) { reboundCaught = error instanceof ReferenceError; }
  }
}
new ReboundDerived();
if (!(reboundBaseCalls === 2 && reboundCaught)) throw "case-11";

class SuperPropertyBase {
  get markerValue() { return this.marker; }
}
class SuperPropertyDerived extends SuperPropertyBase {
  constructor() {
    super();
    this.marker = 12;
    this.directSuperProperty = super.markerValue;
    this.arrowSuperProperty = (() => super.markerValue)();
    this.nestedSuperProperty = (() => () => super.markerValue)()();
  }
}
let superPropertyValue = new SuperPropertyDerived();
if (!(superPropertyValue.directSuperProperty === 12 &&
      superPropertyValue.arrowSuperProperty === 12 &&
      superPropertyValue.nestedSuperProperty === 12)) throw "case-12";

class SuperMethodBase {
  markerMethod() { return this.marker; }
}
class SuperMethodDerived extends SuperMethodBase {
  constructor() {
    super();
    this.marker = 13;
    this.directSuperMethod = super.markerMethod();
    this.arrowSuperMethod = (() => super.markerMethod())();
    this.nestedSuperMethod = (() => () => super.markerMethod())()();
  }
}
let superMethodValue = new SuperMethodDerived();
if (!(superMethodValue.directSuperMethod === 13 &&
      superMethodValue.arrowSuperMethod === 13 &&
      superMethodValue.nestedSuperMethod === 13)) throw "case-13";

true;
