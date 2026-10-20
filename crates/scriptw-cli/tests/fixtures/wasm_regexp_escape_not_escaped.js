function check(value, label) {
  if (!value) {
    throw "RegExp.escape not-escaped fixture failed: " + label;
  }
}

var letters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
var digits = "0123456789";

check(RegExp.escape("") === "", "empty");
check(RegExp.escape(".a") === "\\.a", "dot a");
check(RegExp.escape(".1") === "\\.1", "dot 1");
check(RegExp.escape("." + letters) === "\\." + letters, "letters direct");
check(RegExp.escape("." + digits) === "\\." + digits, "digits direct");
check(RegExp.escape(".a1b2c3D4E5F6") === "\\.a1b2c3D4E5F6", "mixed direct");

var splitLetters = letters.split("");
check(splitLetters.length === 52, "split length");
check(splitLetters[0] === "a", "split first");
check(splitLetters[51] === "Z", "split last");

var letterCount = 0;
letters.split("").forEach(char => {
  check(RegExp.escape("." + char) === "\\." + char, "letter loop");
  letterCount++;
});
check(letterCount === 52, "letter count " + letterCount);

var digitCount = 0;
digits.split("").forEach(char => {
  check(RegExp.escape("." + char) === "\\." + char, "digit loop");
  digitCount++;
});
check(digitCount === 10, "digit count " + digitCount);

true;
