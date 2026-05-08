function add(x, y) { return x + y; }
function pick() { return arguments[1]; }
let receiver = { v: 2 };
let aggregate = AggregateError([1, undefined, 3], "x");
let errorCause = { marker: 1 };
let aggregateCause = { marker: 2 };
let errorMessageDesc = Object.getOwnPropertyDescriptor(Error.prototype, "message");
let errorConstructorDesc = Object.getOwnPropertyDescriptor(Error.prototype, "constructor");
let aggregateMessageDesc = Object.getOwnPropertyDescriptor(AggregateError.prototype, "message");

add.call(null, 1, 2) === 3
  && add.apply(null, [1, 2]) === 3
  && ({ f: add }).f.call(receiver, 1, 2) === 3
  && pick.apply(null, [1, 2, 3]) === 2
  && aggregate.name === "AggregateError"
  && aggregate.errors[1] === undefined
  && new Error("m", { cause: errorCause }).cause === errorCause
  && new AggregateError([], "m", { cause: aggregateCause }).cause === aggregateCause
  && !Object.prototype.hasOwnProperty.call(new AggregateError([], "m"), "cause")
  && AggregateError.length === 2
  && Object.getPrototypeOf(AggregateError) === Error
  && AggregateError.prototype.constructor === AggregateError
  && AggregateError instanceof Function
  && errorMessageDesc.value === ""
  && errorMessageDesc.writable === true
  && errorMessageDesc.enumerable === false
  && errorMessageDesc.configurable === true
  && errorConstructorDesc.value === Error
  && errorConstructorDesc.writable === true
  && errorConstructorDesc.enumerable === false
  && errorConstructorDesc.configurable === true
  && aggregateMessageDesc.value === ""
  && aggregateMessageDesc.writable === true
  && aggregateMessageDesc.enumerable === false
  && aggregateMessageDesc.configurable === true;
