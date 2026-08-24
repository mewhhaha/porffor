Promise.reject("checkpoint-first");

let handled = Promise.reject("checkpoint-handled");
handled.catch(function () {});

Promise.reject("checkpoint-second");
Promise.reject(Symbol("checkpoint-symbol"));
Promise.reject({
  toString: function () {
    throw new RangeError("checkpoint-conversion");
  },
});

let recursivelyRejected = {
  toString: function () {
    Promise.reject(recursivelyRejected);
    return "checkpoint-reentrant";
  },
};
Promise.reject(recursivelyRejected);
Promise.reject("checkpoint-third");
