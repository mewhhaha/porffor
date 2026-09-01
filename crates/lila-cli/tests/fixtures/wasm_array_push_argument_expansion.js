let target = [0];
let order = [];

function record(value) {
  order[order.length] = "arg" + value;
  return value;
}

let spread = {
  [Symbol.iterator]: function () {
    if (target.length !== 1) throw "push started before iterator acquisition";
    order[order.length] = "iterator";
    let value = 9;
    return {
      next: function () {
        if (target.length !== 1) throw "push started before spread expansion";
        if (value > 11) {
          order[order.length] = "done";
          return { done: true };
        }
        order[order.length] = "next" + value;
        return { done: false, value: value++ };
      },
    };
  },
};

let length = target.push(
  record(1),
  record(2),
  record(3),
  record(4),
  record(5),
  record(6),
  record(7),
  record(8),
  ...spread,
  record(12),
);

let expectedOrder = [
  "arg1",
  "arg2",
  "arg3",
  "arg4",
  "arg5",
  "arg6",
  "arg7",
  "arg8",
  "iterator",
  "next9",
  "next10",
  "next11",
  "done",
  "arg12",
];

let correct = length === 13 && target.length === 13 && order.length === expectedOrder.length;
for (let index = 0; index < expectedOrder.length; index++) {
  correct = correct && order[index] === expectedOrder[index];
}
for (let index = 0; index < target.length; index++) {
  correct = correct && target[index] === index;
}
correct;
