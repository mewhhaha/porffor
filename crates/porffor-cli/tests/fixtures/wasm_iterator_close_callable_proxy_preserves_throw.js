var closeCalls = 0;
var iterator = {
  __proto__: Iterator.prototype,
  next: function() {
    return { done: false, value: 1 };
  },
  return: new Proxy(function() {
    closeCalls += 1;
    throw "close throw";
  }, {}),
};

var helper = iterator.map(function() {
  throw "original throw";
});
var caught;
try {
  helper.next();
} catch (error) {
  caught = error;
}

closeCalls === 1 && caught === "original throw";
