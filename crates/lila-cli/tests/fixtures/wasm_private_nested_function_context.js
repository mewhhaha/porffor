class PrivateContextBase {
  baseValue() {
    return 10;
  }
}

class PrivateContext extends PrivateContextBase {
  #value = 1;

  get #paired() {
    return this.#value;
  }

  set #paired(value) {
    this.#value = value;
  }

  #method(offset) {
    return this.#value + offset;
  }

  static #staticValue = 20;

  static get #staticPaired() {
    return this.#staticValue;
  }

  static set #staticPaired(value) {
    this.#staticValue = value;
  }

  static #staticMethod(offset) {
    return this.#staticValue + offset;
  }

  makeOrdinary() {
    const receiver = this;
    function inner(value) {
      receiver.#paired = value;
      return receiver.#paired + receiver.#method(1);
    }
    return inner;
  }

  makeArrow() {
    return (value) => {
      this.#paired = value;
      return this.#paired + this.#method(1);
    };
  }

  makeSuperArrow() {
    return () => super.baseValue() + this.#value;
  }

  makeNamed() {
    const receiver = this;
    return function named(recurse) {
      return recurse ? named(false) : receiver.#value;
    };
  }

  makeTransitive() {
    const receiver = this;
    function middle() {
      return () => receiver.#value;
    }
    return middle;
  }

  static makeOrdinary() {
    const receiver = this;
    function inner(value) {
      receiver.#staticPaired = value;
      return receiver.#staticPaired + receiver.#staticMethod(1);
    }
    return inner;
  }

  static makeArrow() {
    return (value) => {
      this.#staticPaired = value;
      return this.#staticPaired + this.#staticMethod(1);
    };
  }
}

const instance = new PrivateContext();
const ordinary = instance.makeOrdinary();
const arrow = instance.makeArrow();
if (ordinary(2) !== 5) throw "ordinary private context";
if (arrow(3) !== 7) throw "arrow private context";
if (instance.makeSuperArrow()() !== 13) throw "private context changed super home object";
if (instance.makeNamed()(true) !== 3) throw "named private context";
if (instance.makeTransitive()()() !== 3) throw "transitive private context";

if (PrivateContext.makeOrdinary()(21) !== 43) throw "static ordinary private context";
if (PrivateContext.makeArrow()(22) !== 45) throw "static arrow private context";

function makePrivateFactory(value) {
  return class PrivateFactoryResult {
    #value = value;

    makeReader() {
      return (target) => target.#value;
    }
  };
}

const FirstPrivateFactoryResult = makePrivateFactory(30);
const SecondPrivateFactoryResult = makePrivateFactory(40);
const firstFactoryInstance = new FirstPrivateFactoryResult();
const secondFactoryInstance = new SecondPrivateFactoryResult();
const readFirstFactory = firstFactoryInstance.makeReader();
const readSecondFactory = secondFactoryInstance.makeReader();
if (readFirstFactory(firstFactoryInstance) !== 30) throw "first private factory context";
if (readSecondFactory(secondFactoryInstance) !== 40) throw "second private factory context";

let crossedFactoryBrand = false;
try {
  readFirstFactory(secondFactoryInstance);
} catch (error) {
  crossedFactoryBrand = error.name === "TypeError";
}
if (!crossedFactoryBrand) throw "private factory contexts shared identity";

class OuterPrivateContext {
  #value = 50;
  #outerOnly = 2;

  makeInner() {
    const readOuterShadow = (target) => target.#value;
    return class InnerPrivateContext {
      #value = 60;

      makeReader(outerTarget) {
        const receiver = this;
        return function innerReader() {
          return readOuterShadow(outerTarget) + outerTarget.#outerOnly + receiver.#value;
        };
      }
    };
  }
}

const outer = new OuterPrivateContext();
const InnerPrivateContext = outer.makeInner();
const inner = new InnerPrivateContext();
const nestedReader = inner.makeReader(outer);
if (nestedReader() !== 112) throw "nested private environment shadowing";

true;
