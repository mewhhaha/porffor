function add(x, y) { return x + y; }
function pick() { return arguments[1]; }
let receiver = { v: 2 };
let aggregate = AggregateError([1, undefined, 3], "x");

add.call(null, 1, 2) === 3
  && add.apply(null, [1, 2]) === 3
  && ({ f: add }).f.call(receiver, 1, 2) === 3
  && pick.apply(null, [1, 2, 3]) === 2
  && aggregate.name === "AggregateError"
  && aggregate.errors[1] === undefined
  && AggregateError instanceof Function;
