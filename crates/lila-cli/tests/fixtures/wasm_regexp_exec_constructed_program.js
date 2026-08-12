function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

check(/foo/.test("xfoox"), true, "literal test before constructed regexp");

var constructed = new RegExp("(.|\r|\n)*", "");
check(Object.getPrototypeOf(constructed), RegExp.prototype, "ordinary construction prototype");
var result = constructed.exec();
check(result.length, 2, "result length");
check(result[0], "undefined", "whole match");
check(result[1], "d", "last capture");
check(result.index, 0, "match index");
check(result.input, "undefined", "input");
check(result.groups, undefined, "groups");
check(constructed.lastIndex, 0, "lastIndex");

var constructedWithDefaultFlags = new RegExp("World");
var defaultFlagsResult = constructedWithDefaultFlags.exec("Hello World");
check(defaultFlagsResult[0], "World", "one-argument whole match");
check(defaultFlagsResult.index, 6, "one-argument match index");
true;
