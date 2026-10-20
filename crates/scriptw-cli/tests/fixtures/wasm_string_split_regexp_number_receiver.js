function check(value, label) {
  if (!value) {
    throw "regexp number split fixture failed: " + label;
  }
}

var re = /\u0037\u0037/g;
Number.prototype.split = String.prototype.split;

var parts = (6776767677.006771122677555).split(re);

check(parts.constructor === Array, "constructor");
check(parts.length === 4, "length");
check(parts[0] === "6", "first");
check(parts[1] === "67676", "second");
check(parts[2] === ".006", "third");
check(parts[3] === "1", "fourth");

true;
