function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

var sequence = /(?:ab|cd)\d?/g;
check(sequence.exec("ab  cd2  ab34  cd")[0], "ab", "sequence first match");
check(sequence.exec("ab  cd2  ab34  cd")[0], "cd2", "sequence second match");
check(sequence.exec("ab  cd2  ab34  cd")[0], "ab3", "sequence third match");
check(sequence.exec("ab  cd2  ab34  cd")[0], "cd", "sequence fourth match");
check(sequence.exec("ab  cd2  ab34  cd"), null, "sequence ends with null");
check(sequence.lastIndex, 0, "sequence failure resets lastIndex");
check(/(?:xy|zq)\d?/.exec("--zq7")[0], "zq7", "branch literals are generic");
check(/(?:a|ab)\d?/.exec("ab3")[0], "a", "first branch has precedence");

var input = "aacd2233ab12nm444ab42";
var first = /(?:ab|cd)\d?/g;
var firstMatch = first.exec(input);
check(firstMatch[0], "cd2", "first match consumes one digit");
check(firstMatch.index, 2, "first match index");
check(firstMatch.input, input, "first match input");
check(first.lastIndex, 5, "first match lastIndex");

var sticky = /(?:cat|dog)\d?/y;
sticky.lastIndex = 1;
var stickyMatch = sticky.exec("xdog7");
check(stickyMatch[0], "dog7", "sticky match consumes one digit");
check(stickyMatch.index, 1, "sticky match starts at lastIndex");
check(sticky.lastIndex, 5, "sticky match advances lastIndex");
sticky.lastIndex = 0;
check(sticky.exec("xdog7"), null, "sticky mismatch returns null");
check(sticky.lastIndex, 0, "sticky mismatch resets lastIndex");

var resumed = /(?:ab|cd)\d?/g;
resumed.lastIndex = 12;
var resumedMatch = resumed.exec(input);
check(resumedMatch[0], "ab4", "search resumes from lastIndex");
check(resumedMatch.index, 17, "resumed match index");
check(resumed.lastIndex, 20, "resumed match lastIndex");

var outOfRange = /(?:ab|cd)\d?/g;
outOfRange.lastIndex = 100;
check(outOfRange.exec("aacd22"), null, "out-of-range lastIndex returns null");
check(outOfRange.lastIndex, 0, "out-of-range lastIndex resets");

var negative = /(?:ab|cd)\d?/g;
negative.lastIndex = -100;
check(negative.exec("aacd22")[0], "cd2", "negative lastIndex starts at zero");
check(negative.lastIndex, 5, "negative lastIndex match advances");

var nan = /(?:ab|cd)\d?/g;
nan.lastIndex = Math.NaN;
check(nan.exec("aacd22")[0], "cd2", "NaN lastIndex starts at zero");
check(nan.lastIndex, 5, "NaN lastIndex match advances");

var throwing = /(?:ab|cd)\d?/g;
throwing.lastIndex = {
  valueOf: function () {
    throw "lastIndex valueOf";
  }
};
var propagated = false;
try {
  throwing.exec("ab");
} catch (error) {
  propagated = error === "lastIndex valueOf";
}
check(propagated, true, "lastIndex valueOf throw propagates");

var secondCall = /(?:ab|cd)\d?/g;
check(secondCall.exec("aacd22")[0], "cd2", "second-call setup match");
check(secondCall.exec("aacd22"), null, "second call without a match returns null");
check(secondCall.lastIndex, 0, "second call failure resets lastIndex");
true;
