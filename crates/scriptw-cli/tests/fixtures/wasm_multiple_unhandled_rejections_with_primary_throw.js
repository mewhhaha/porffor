Promise.reject("primary-first");

let handled = Promise.reject("primary-handled");
handled.catch(function () {});

Promise.reject("primary-second");
Promise.reject({
  toString: function () {
    throw new RangeError("primary-conversion");
  },
});
Promise.reject("primary-third");

throw new TypeError("primary-script-failure");
