// The unclamped choice-frame formula (120 splits x 320001 bytes x 32) is ~1.2 GiB,
// past the 1 GiB store cap; the clamped arena must still match with real-usage memory.
function check(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}
var re = /^(ab|aa|ac|ad|ae|af|ag|ah|ai|aj|ak|al|am|an|ao|ap|aq|ar|as|at|au|av|aw|ax|ay|az|ba|bb|bc|bd|be|bf|bg|bh|bi|bj|bk|bl|bm|bn|bo|bp|bq|br|bs|bt|bu|bv|bw|bx|by|bz|ca|cb|cc|cd|ce|cf|cg|ch|ci|cj|ck|cl|cm|cn|co|cp|cq|cr|cs|ct|cu|cv|cw|cx|cy|cz|da|db|dc|dd|de|df|dg|dh|di|dj|dk|dl|dm|dn|do|dp|dq|dr|ds|dt|du|dv|dw|dx|dy|dz|ea|eb|ec|ed|ee|ef|eg|eh|ei|ej|ek|el|em|en|eo|ep)+$/;
check(re.test("ab".repeat(160000)), true, "clamped-arena-matches");
check(re.test("ab".repeat(160000) + "!"), false, "clamped-arena-rejects");

true;
