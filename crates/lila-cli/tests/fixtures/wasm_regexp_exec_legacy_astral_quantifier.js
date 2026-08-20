function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

function exec(re, input, value, label) {
  var result = re.exec(input);
  check(result !== null, true, label + " result");
  check(result[0], value, label + " value");
  check(result.index, 0, label + " index");
  return result;
}

var lead = "\uD842";
var trail = "\uDFB7";
var pair = lead + trail;

exec(/^𠮷?$/, pair, pair, "legacy optional full pair");
exec(/^𠮷?$/, lead, lead, "legacy optional lone lead");
check(/^𠮷?$/.test(""), false, "legacy optional keeps lead mandatory");

exec(/^𠮷{0}$/, lead, lead, "legacy zero trails");
exec(/^𠮷{2}$/, lead + trail + trail, lead + trail + trail, "legacy two trails");
exec(/^𠮷+$/, lead + trail + trail, lead + trail + trail, "legacy repeated trails");

var greedy = /^(𠮷?)\uDFB7/.exec(lead + trail + trail);
check(greedy !== null, true, "legacy greedy result");
check(greedy[0], lead + trail + trail, "legacy greedy value");
check(greedy[1], pair, "legacy greedy capture");

var lazy = /^(𠮷??)\uDFB7/.exec(lead + trail + trail);
check(lazy !== null, true, "legacy lazy result");
check(lazy[0], pair, "legacy lazy value");
check(lazy[1], lead, "legacy lazy capture");

exec(/^𠮷?$/u, pair, pair, "unicode optional full scalar");
exec(/^𠮷?$/u, "", "", "unicode optional empty");
check(/^𠮷?$/u.test(lead), false, "unicode rejects lone lead as scalar");

true;
