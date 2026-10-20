function check(actual, expected, label) { if (actual !== expected) throw label; }
function exec(re, input, value, index, label) {
  var result = re.exec(input);
  check(result !== null, true, label + " result");
  check(result[0], value, label + " value");
  check(result.index, index, label + " index");
  return result;
}

var pile = "\uD83D\uDCA9";
var high = "\uD83D";
var low = "\uDCA9";
var global = /./g;
exec(global, pile, high, 0, "global astral high");
check(global.lastIndex, 1, "global high lastIndex");
exec(global, pile, low, 1, "global astral low");
check(global.lastIndex, 2, "global low lastIndex");
check(global.exec(pile), null, "global astral reset");
check(global.lastIndex, 0, "global reset lastIndex");
exec(/../, pile, pile, 0, "two dots reconstruct astral scalar");
exec(/.x/, pile + "x", low + "x", 1, "unanchored low-half candidate");

var sticky = /./y;
sticky.lastIndex = 1;
exec(sticky, pile, low, 1, "sticky low half");
check(sticky.lastIndex, 2, "sticky low lastIndex");
exec(/./, high, high, 0, "lone surrogate dot");

for (var i = 0; i < 4; i = i + 1) {
  check(/./.exec(["\n", "\r", "\u2028", "\u2029"][i]), null, "line terminator " + i);
}
exec(/./, "\u000B", "\u000B", 0, "nearby nonterminator");
exec(/./, "\u00E9", "\u00E9", 0, "nonascii nonterminator");

exec(/(.|..)x/, pile + "x", pile + "x", 0, "backtracking restores half cursor");
var captured = /(.)*/.exec(pile);
check(captured[0], pile, "star dot full");
check(captured[1], low, "star dot final low capture");

var later = /..y/.exec(pile + "x" + pile + "y");
check(later[0], pile + "y", "failed high candidate finds later match");
check(later.index, 3, "failed high candidate index");
var lowStart = /x/g;
lowStart.lastIndex = 1;
exec(lowStart, pile + "x", "x", 2, "low start advances byte cursor");
check(lowStart.lastIndex, 3, "low start lastIndex");
true;
