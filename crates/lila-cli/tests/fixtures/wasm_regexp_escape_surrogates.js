function check(value) {
  if (!value) {
    throw "RegExp.escape surrogate fixture failed";
  }
}

check(RegExp.escape("\uD800") === "\\ud800");
check(RegExp.escape("\uDBFF") === "\\udbff");
check(RegExp.escape("\uDC00") === "\\udc00");
check(RegExp.escape("\uDFFF") === "\\udfff");
check(RegExp.escape(String.fromCharCode(0xD800)) === "\\ud800");
check(RegExp.escape(String.fromCharCode(0xDFFF)) === "\\udfff");

true;
