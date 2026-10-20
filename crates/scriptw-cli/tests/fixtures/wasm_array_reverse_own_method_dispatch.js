let target = [1, 2, 3];
let order = [];
let receiver;
let callCount = 0;

function record(value) {
  order[order.length] = value;
  return value;
}

target.reverse = function (first, second, third, fourth) {
  receiver = this;
  callCount++;
  return first + second + third + fourth;
};

let result = target.reverse(record(1), ...[record(2), record(3)], record(4));

result === 10
  && receiver === target
  && callCount === 1
  && order.length === 4
  && order[0] === 1
  && order[1] === 2
  && order[2] === 3
  && order[3] === 4
  && target.length === 3
  && target[0] === 1
  && target[1] === 2
  && target[2] === 3;
