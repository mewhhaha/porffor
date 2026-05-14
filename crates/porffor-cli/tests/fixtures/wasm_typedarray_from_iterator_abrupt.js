let nextSentinel = { name: "next sentinel" };
let nextSource = {};
Object.defineProperty(nextSource, Symbol.iterator, {
  get: function () {
    return function () {
      return {
        next: function () {
          throw nextSentinel;
        }
      };
    };
  }
});

let caught = false;
try {
  Uint8Array.from(nextSource);
} catch (error) {
  caught = error === nextSentinel;
}
if (!caught) throw "iterator next abrupt";

let valueSentinel = { name: "value sentinel" };
let valueSource = {};
Object.defineProperty(valueSource, Symbol.iterator, {
  get: function () {
    return function () {
      return {
        next: function () {
          let step = {};
          Object.defineProperty(step, "value", {
            get: function () {
              throw valueSentinel;
            }
          });
          return step;
        }
      };
    };
  }
});

caught = false;
try {
  Float32Array.from(valueSource);
} catch (error) {
  caught = error === valueSentinel;
}
if (!caught) throw "iterator value abrupt";

262;
