var order = [];
var step = 0;
var ignoredSpread = {};
var receiver = [0];

Object.defineProperty(receiver, "0", {
  configurable: true,
  get: function() {
    order.push("get");
    return [1];
  }
});

ignoredSpread[Symbol.iterator] = function() {
  order.push("iterator");
  return {
    next: function() {
      step++;
      order.push("next" + step);
      if (step === 1) return { done: false, value: "ignored spread value" };
      return { done: true };
    }
  };
};

function depth() {
  order.push("depth");
  return 1;
}

function ignoredSecondArgument() {
  order.push("second");
  return "ignored second value";
}

var flattened = receiver.flat(
  depth(),
  ignoredSecondArgument(),
  ...ignoredSpread
);

flattened.length === 1 &&
  flattened[0] === 1 &&
  order.length === 6 &&
  order[0] === "depth" &&
  order[1] === "second" &&
  order[2] === "iterator" &&
  order[3] === "next1" &&
  order[4] === "next2" &&
  order[5] === "get";
