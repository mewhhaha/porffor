let dynamicLengthKey = ["len", "gth"].join("");
let empty = [];
let one = [1];
let named = [1];
named.prop = true;

function argumentsHasOwnLength() {
  return arguments.hasOwnProperty(dynamicLengthKey);
}

let ok = empty.hasOwnProperty(dynamicLengthKey)
  && one.hasOwnProperty(dynamicLengthKey)
  && argumentsHasOwnLength(1)
  && delete empty.length === false
  && empty.hasOwnProperty(dynamicLengthKey)
  && delete one[0] === true
  && one.hasOwnProperty("0") === false
  && delete named.prop === true
  && named.prop === undefined;

ok;
