let target = [3, 1, 2];
let order = [];
let receiver;
let callCount = 0;

function record(value) {
  order[order.length] = value;
  return value;
}

target.sort = function (first, second, third, fourth) {
  receiver = this;
  callCount++;
  return first + second + third + fourth;
};

let result = target.sort(record(1), ...[record(2), record(3)], record(4));

result === 10
  && receiver === target
  && callCount === 1
  && order.length === 4
  && order[0] === 1
  && order[1] === 2
  && order[2] === 3
  && order[3] === 4
  && target.length === 3
  && target[0] === 3
  && target[1] === 1
  && target[2] === 2;
