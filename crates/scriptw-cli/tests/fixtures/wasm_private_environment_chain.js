class OuterPrivateEnvironment {
  #field = 40;

  get #value() {
    return this.#field;
  }

  set #value(value) {
    this.#field = value;
  }

  #method(value) {
    return value;
  }

  makeNestedClass() {
    return class NestedPrivateEnvironment {
      #inner = 2;

      readOuterField(target) {
        return target.#field;
      }

      readOuterGetter(target) {
        return target.#value;
      }

      writeOuterSetter(target, value) {
        target.#value = value;
        return target.#field;
      }

      hasOuterField(target) {
        return #field in target;
      }

      readInnerField() {
        return this.#inner;
      }

      callOuterMethod(getTarget, getArgument) {
        return getTarget().#method(getArgument());
      }
    };
  }
}

const outer = new OuterPrivateEnvironment();
const NestedPrivateEnvironment = outer.makeNestedClass();
const nested = new NestedPrivateEnvironment();

if (nested.readOuterField(outer) !== 40) throw "nested outer private field";
if (nested.readOuterGetter(outer) !== 40) throw "nested outer private getter";
if (nested.writeOuterSetter(outer, 41) !== 41) throw "nested outer private setter";
if (!nested.hasOuterField(outer)) throw "nested outer private brand";
if (nested.readInnerField() !== 2) throw "nested own private field";

let privateMethodReceiverCalls = 0;
let privateMethodArgumentCalls = 0;
if (
  nested.callOuterMethod(
    () => {
      privateMethodReceiverCalls += 1;
      return outer;
    },
    () => {
      privateMethodArgumentCalls += 1;
      return 42;
    },
  ) !== 42
) {
  throw "nested outer private method";
}
if (privateMethodReceiverCalls !== 1 || privateMethodArgumentCalls !== 1) {
  throw "private method evaluation count";
}

let wrongOuterBrand = false;
try {
  nested.callOuterMethod(
    () => ({}),
    () => {
      privateMethodArgumentCalls += 1;
      return 43;
    },
  );
} catch (error) {
  wrongOuterBrand = error.name === "TypeError";
}
if (!wrongOuterBrand) throw "nested outer wrong brand";
if (privateMethodArgumentCalls !== 1) throw "private method argument before brand check";

true;
