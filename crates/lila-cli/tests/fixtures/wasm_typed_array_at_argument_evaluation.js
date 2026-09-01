var order = [];
var step = 0;
var ignoredSpread = {};
var receiver = new Uint8Array([4, 8, 15]);
var index = {
  valueOf: function() {
    order.push("coerce");
    return -1;
  }
};

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

function indexArgument() {
  order.push("index");
  return index;
}

function ignoredSecondArgument() {
  order.push("second");
  return "ignored second value";
}

var value = receiver.at(
  indexArgument(),
  ignoredSecondArgument(),
  ...ignoredSpread
);

value === 15 &&
  order.length === 6 &&
  order[0] === "index" &&
  order[1] === "second" &&
  order[2] === "iterator" &&
  order[3] === "next1" &&
  order[4] === "next2" &&
  order[5] === "coerce";
