function check(actual, expected, label) {
  if (actual !== expected) {
    throw label;
  }
}

class RuntimeRegExp extends RegExp {}

var patterns = ["(?<letter>a)", "(b)"];
var flags = ["g", "gy"];
var named = new RuntimeRegExp(patterns[0], flags[0]);
check("aba".replaceAll(named, "$<letter>$&"), "aabaa", "named replacement");

var captured = new RuntimeRegExp(patterns[1], flags[1]);
check("bb".replaceAll(captured, function(match, capture, position, input) {
  return match + capture + position + input.length;
}), "bb02bb12", "functional replacement");

var empty = new RegExp(undefined, "g");
check(empty.flags, "g", "dynamic empty flags");
check(empty.global, true, "dynamic empty global");
var emptyMatch = empty.exec("asdf");
check(emptyMatch[0], "", "dynamic empty first match");
check(emptyMatch.index, 0, "dynamic empty first index");
empty.lastIndex = 1;
check(empty.lastIndex, 1, "dynamic empty assigned lastIndex");
check(empty.exec("asdf").index, 1, "dynamic empty second index");

true;
