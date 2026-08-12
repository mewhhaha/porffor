let order = "";

class PrivateAssignmentReferences {
  #field = 0;

  wrongBrandUsesPutValueOrder(wrong) {
    try {
      (order += "b", wrong).#field = (order += "v", 1);
    } catch (error) {
      order += error.name === "TypeError" ? "t" : "x";
    }
  }

  exerciseAssignmentTargets() {
    order = "";
    for ((order += "o", this).#field of [1, 2]) {}
    if (order !== "oo") throw "private for-of target evaluation";

    order = "";
    for ((order += "i", this).#field in { first: 1, second: 2 }) {}
    if (order !== "ii") throw "private for-in target evaluation";

    let normalBaseCount = 0;
    [(normalBaseCount += 1, this).#field] = [3];
    if (normalBaseCount !== 1 || this.#field !== 3) {
      throw "private array target preparation";
    }

    let restBaseCount = 0;
    [...(restBaseCount += 1, this).#field] = [4, 5];
    if (restBaseCount !== 1) {
      throw "private array rest target preparation";
    }

    let objectBaseCount = 0;
    ({ value: (objectBaseCount += 1, this).#field } = { value: 6 });
    if (objectBaseCount !== 1 || this.#field !== 6) {
      throw "private object assignment target";
    }
  }

  throwingForOfClosesIterator(wrong) {
    let closed = false;
    const iterator = {
      next() {
        return { done: false, value: 1 };
      },
      return() {
        closed = true;
        return { done: true };
      },
    };
    const iterable = {
      [Symbol.iterator]() {
        return iterator;
      },
    };

    try {
      for (wrong.#field of iterable) {}
    } catch (error) {
      if (error.name !== "TypeError") throw "private for-of wrong error";
    }
    if (!closed) throw "private for-of iterator close";
  }
}

const instance = new PrivateAssignmentReferences();
instance.wrongBrandUsesPutValueOrder({});
if (order !== "bvt") throw "private write PutValue order";
instance.exerciseAssignmentTargets();
instance.throwingForOfClosesIterator({});

true;
