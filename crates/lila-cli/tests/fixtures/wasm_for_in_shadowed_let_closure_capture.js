let x = "outside";
var first;
var second;

for (let x in { a: 0, b: 0 }) {
  if (!first) {
    first = function () {
      return x;
    };
  } else {
    second = function () {
      return x;
    };
  }
}

if (first() !== "a") {
  throw first();
}

if (second() !== "b") {
  throw second();
}

true;
