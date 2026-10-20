var order = [];
var step = 0;
var callbackThis = {};
var ignoredSpread = {};
var receiver = [0];

Object.defineProperty(receiver, "0", {
  configurable: true,
  get: function() {
    order.push("get");
    return 1;
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

function mapper(value) {
  order.push("map");
  if (this !== callbackThis) throw "unexpected callback this";
  return [value + 1];
}

function callbackArgument() {
  order.push("callback");
  return mapper;
}

function thisArgument() {
  order.push("this");
  return callbackThis;
}

function ignoredThirdArgument() {
  order.push("third");
  return "ignored third value";
}

var flattened = receiver.flatMap(
  callbackArgument(),
  thisArgument(),
  ignoredThirdArgument(),
  ...ignoredSpread
);

flattened.length === 1 &&
  flattened[0] === 2 &&
  order.length === 8 &&
  order[0] === "callback" &&
  order[1] === "this" &&
  order[2] === "third" &&
  order[3] === "iterator" &&
  order[4] === "next1" &&
  order[5] === "next2" &&
  order[6] === "get" &&
  order[7] === "map";
