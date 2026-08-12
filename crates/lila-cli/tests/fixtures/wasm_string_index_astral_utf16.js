function check(value, label) {
  if (!value) {
    throw "String indexed access fixture failed: " + label;
  }
}

var astral = "💩";
var high = "\uD83D";
var low = "\uDCA9";

check(astral.length === 2, "astral length is UTF-16 units");
check(astral[0] === high, "astral high surrogate");
check(astral[1] === low, "astral low surrogate");
check(astral === "💩", "astral equality");

check("abc"[0] === "a", "BMP first unit");
check("abc"[1] === "b", "BMP middle unit");
check("abc"[2] === "c", "BMP last unit");

check(high[0] === high, "lone high surrogate");
check(low[0] === low, "lone low surrogate");
check((high + low)[0] === high, "escaped surrogate pair high unit");
check((high + low)[1] === low, "escaped surrogate pair low unit");

true;
