let root = { object: { x: 1 }, array: [2] };
let rootRole = false;
let objectRole = false;
let nestedObjectRole = false;
let arrayRole = false;
let arrayElementRole = false;

function replacer(key, value) {
  if (key === "") rootRole = this !== root && this[""] === root && value === root;
  if (key === "object") objectRole = this === root && value === root.object;
  if (key === "x") nestedObjectRole = this === root.object && value === 1;
  if (key === "array") arrayRole = this === root && value === root.array;
  if (key === "0") arrayElementRole = this === root.array && value === 2;
  return value;
}

let serialized = JSON.stringify(root, replacer);
let thrown = { marker: 1 };
let abruptIdentity = false;
try {
  JSON.stringify({ boom: 1 }, function (key, value) {
    if (key === "boom") throw thrown;
    return value;
  });
} catch (error) {
  abruptIdentity = error === thrown;
}

serialized === '{"object":{"x":1},"array":[2]}'
  && rootRole
  && objectRole
  && nestedObjectRole
  && arrayRole
  && arrayElementRole
  && abruptIdentity;
