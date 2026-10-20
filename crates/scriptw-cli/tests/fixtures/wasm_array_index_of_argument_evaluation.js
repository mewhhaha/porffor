var order = [];
var step = 0;
var ignoredSpread = {};
var receiver = [0];

Object.defineProperty(receiver, "0", {
  configurable: true,
  get: function() {
    order.push("get");
    return 42;
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

function searchElement() {
  order.push("search");
  return 42;
}

function fromIndex() {
  order.push("from");
  return 0;
}

function ignoredThirdArgument() {
  order.push("third");
  return "ignored third value";
}

var index = receiver.indexOf(
  searchElement(),
  fromIndex(),
  ignoredThirdArgument(),
  ...ignoredSpread
);

index === 0 &&
  order.length === 7 &&
  order[0] === "search" &&
  order[1] === "from" &&
  order[2] === "third" &&
  order[3] === "iterator" &&
  order[4] === "next1" &&
  order[5] === "next2" &&
  order[6] === "get";
