function check(value, label) {
  if (!value) {
    throw "String toString/valueOf fixture failed: " + label;
  }
}

var toString = String.prototype.toString;
var valueOf = String.prototype.valueOf;
var bound = "bound";
var boxed = new String("box");

check("abc".toString() === "abc", "direct toString");
check("".toString() === "", "empty direct toString");
check(bound.toString() === "bound", "bound string toString");
check("abc".valueOf() === "abc", "direct valueOf");
check(bound.valueOf() === "bound", "bound string valueOf");
check(toString.call("str") === "str", "borrowed toString primitive");
check(valueOf.call("str") === "str", "borrowed valueOf primitive");
check(boxed.toString() === "box", "boxed toString");
check(boxed.valueOf() === "box", "boxed valueOf");
check(String.prototype.toString.name === "toString", "toString name");
check(String.prototype.valueOf.name === "valueOf", "valueOf name");
check(String.prototype.toString.length === 0, "toString length");
check(String.prototype.valueOf.length === 0, "valueOf length");

true;
