var proto = { p4: "p4" };
var object = { p1: "p1", p2: "p2", p3: "p3" };
Object.setPrototypeOf(object, proto);

var keys = [];
for (var key in object) {
  keys.push(key);
}

if (keys.length !== 4) {
  throw keys.length;
}

if (keys[0] !== "p1") {
  throw keys[0];
}

if (keys[1] !== "p2") {
  throw keys[1];
}

if (keys[2] !== "p3") {
  throw keys[2];
}

if (keys[3] !== "p4") {
  throw keys[3];
}

var shadowProto = { p2: "p2" };
var shadow = Object.create(shadowProto, {
  p1: { value: "p1", enumerable: true },
  p2: { value: "hidden", enumerable: false },
});

var shadowKeys = [];
for (var shadowKey in shadow) {
  shadowKeys.push(shadowKey);
}

if (shadowKeys.length !== 1) {
  throw shadowKeys.length;
}

if (shadowKeys[0] !== "p1") {
  throw shadowKeys[0];
}

true;
