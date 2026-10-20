// Bounds no addressable input reaches lower exactly: unsatisfiable minimums
// never match, unbindable maximums desugar to stars. Covers static literals,
// computed constructor patterns (dynamic path), and both flag modes.
function check(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}
var MAX = Number.MAX_SAFE_INTEGER;

check(/b{9007199254740991}/.test("b"), false, "lit-huge-min");
check(/b{0,9007199254740991}/.test("bbb"), true, "lit-huge-max");
check(/b{2,9007199254740991}/.test("bb"), true, "lit-huge-max-min");
check(/b{2,9007199254740991}/.test("b"), false, "lit-huge-max-short");
check(/b{9007199254740991}?/.test("b"), false, "lit-huge-min-lazy");
check(/b{0,9007199254740991}?/.test(""), true, "lit-huge-max-lazy");

check(new RegExp("b{" + MAX + "}", "u").test(""), false, "dyn-u-huge-min");
check(new RegExp("b{" + MAX + ",}?").test("a"), false, "dyn-lazy-huge-min");
check(new RegExp("b{" + MAX + "," + MAX + "}").test("b"), false, "dyn-huge-minmax");
check(new RegExp("b{0," + MAX + "}").test("bbb"), true, "dyn-huge-max");
check(new RegExp("b{2," + MAX + "}").test("bb"), true, "dyn-huge-max-min");
check(new RegExp("b{184467440737095516160}").test("b"), false, "dyn-overflow-min");
check(new RegExp("b{5,184467440737095516160}").test("bbbbb"), true, "dyn-overflow-max");

check(/^(?:b{9007199254740991})$/.test(""), false, "never-group");
check(/^b{9007199254740991}|^a$/.test("a"), true, "never-alt");
check(/(?<=b{9007199254740991})a/.test("a"), false, "never-lookbehind-pos");
check(/(?<!b{9007199254740991})a/.test("a"), true, "never-lookbehind-neg");

var reversedThrows = false;
try {
  new RegExp("b{" + MAX + ",5}");
} catch (error) {
  if (!(error instanceof SyntaxError)) throw error;
  reversedThrows = true;
}
check(reversedThrows, true, "reversed-huge-throws");

// Clean computed unicode patterns compile through the runtime parser.
var dyn = String.fromCharCode(0x61) + "+";
check(new RegExp(dyn, "u").test("aaa"), true, "dyn-u-clean-match");
check(new RegExp(dyn, "u").test("b"), false, "dyn-u-clean-nomatch");

true;
