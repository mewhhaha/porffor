function check(value, label) {
  if (!value) {
    throw "boxed String split fixture failed: " + label;
  }
}

var blank = new String(" ");
var blankParts = blank.split("");
check(blankParts.constructor === Array, "blank constructor");
check(blankParts.length === 1, "blank length");
check(blankParts[0] === " ", "blank element");

var words = new String("one two three");
var chars = words.split("");
check(chars.constructor === Array, "chars constructor");
check(chars.length === words.length, "chars length");
check(chars[0] === "o", "chars first");
check(chars[1] === "n", "chars second");
check(chars[11] === "e", "chars index 11");
check(chars[12] === "e", "chars index 12");

var whole = words.split();
check(whole.length === 1, "undefined separator length");
check(whole[0] === "one two three", "undefined separator element");

true;
