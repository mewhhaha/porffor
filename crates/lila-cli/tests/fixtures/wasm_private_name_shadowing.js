function assertPrivateSetterShadowing(Outer) {
  const outer = new Outer();
  const Inner = outer.Inner;
  const inner = new Inner();

  if (!inner.has(inner)) throw "inner private field brand missing";
  if (inner.has(outer)) throw "inner private field reused outer setter brand";

  inner.write(inner, "inner");
  if (inner.read() !== "inner") {
    if (inner.value === "inner") throw "inner write selected outer private setter";
    throw "inner private field write";
  }

  outer.write("outer");
  if (outer.value !== "outer") throw "outer private setter write";

  try {
    inner.write(outer, "wrong brand");
  } catch (error) {
    if (error.name === "TypeError") return;
  }

  throw "shadowed private field accepted outer setter brand";
}

class OuterDeclaration {
  set #value(value) {
    this.value = value;
  }

  write(value) {
    this.#value = value;
  }

  Inner = class {
    #value;

    write(target, value) {
      target.#value = value;
    }

    read() {
      return this.#value;
    }

    has(target) {
      return #value in target;
    }
  };
}

const OuterExpression = class {
  set #value(value) {
    this.value = value;
  }

  write(value) {
    this.#value = value;
  }

  Inner = class {
    #value;

    write(target, value) {
      target.#value = value;
    }

    read() {
      return this.#value;
    }

    has(target) {
      return #value in target;
    }
  };
};

assertPrivateSetterShadowing(OuterDeclaration);
assertPrivateSetterShadowing(OuterExpression);

true;
