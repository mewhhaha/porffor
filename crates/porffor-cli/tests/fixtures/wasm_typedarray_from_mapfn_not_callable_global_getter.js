var getIterator = 0;
var source = {};

Object.defineProperty(source, Symbol.iterator, {
  get: function () {
    getIterator++;
    return function () {
      return {
        next: function () {
          return { done: true };
        }
      };
    };
  }
});

var caught = false;
try {
  Uint8Array.from(source, {});
} catch (error) {
  caught = error instanceof TypeError;
}

if (!caught) throw "mapfn should be validated before iterator lookup";
if (getIterator !== 0) throw "iterator getter should not run";

262;
