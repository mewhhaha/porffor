var order = [];
var step = 0;
var ignoredSpread = {};

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

var found = [42].includes(
  searchElement(),
  fromIndex(),
  ignoredThirdArgument(),
  ...ignoredSpread
);

found === true &&
  order.length === 6 &&
  order[0] === "search" &&
  order[1] === "from" &&
  order[2] === "third" &&
  order[3] === "iterator" &&
  order[4] === "next1" &&
  order[5] === "next2";
