var object = { p1: "p1", p2: "p2", p3: "p3" };
object.p4 = "p4";
object[2] = "2";
object[0] = "0";
object[1] = "1";
delete object.p1;
delete object.p3;
object.p1 = "p1";

var keys = [];
for (var key in object) {
  keys.push(key);
}

if (keys.length !== 6) {
  throw keys.length;
}

if (keys[0] !== "0") {
  throw keys[0];
}

if (keys[1] !== "1") {
  throw keys[1];
}

if (keys[2] !== "2") {
  throw keys[2];
}

if (keys[3] !== "p2") {
  throw keys[3];
}

if (keys[4] !== "p4") {
  throw keys[4];
}

if (keys[5] !== "p1") {
  throw keys[5];
}

true;
