function check(value, label) {
  if (!value) {
    throw "regexp String split fixture failed: " + label;
  }
}

var splitL = new String("hello").split(/l/);
check(splitL.constructor === Array, "literal l constructor");
check(splitL.length === 3, "literal l length");
check(splitL[0] === "he", "literal l first");
check(splitL[1] === "", "literal l second");
check(splitL[2] === "o", "literal l third");

var splitLimit = new String("hello").split(/l/, 2);
check(splitLimit.constructor === Array, "literal l limit constructor");
check(splitLimit.length === 2, "literal l limit length");
check(splitLimit[0] === "he", "literal l limit first");
check(splitLimit[1] === "", "literal l limit second");

var constructed = new String("hello").split(new RegExp("l"));
check(constructed.constructor === Array, "constructed l constructor");
check(constructed.length === 3, "constructed l length");
check(constructed[0] === "he", "constructed l first");
check(constructed[1] === "", "constructed l second");
check(constructed[2] === "o", "constructed l third");

var constructedEmpty = new String("hello").split(new RegExp);
check(constructedEmpty.constructor === Array, "constructed empty constructor");
check(constructedEmpty.length === 5, "constructed empty length");
check(constructedEmpty[0] === "h", "constructed empty first");
check(constructedEmpty[1] === "e", "constructed empty second");
check(constructedEmpty[2] === "l", "constructed empty third");
check(constructedEmpty[3] === "l", "constructed empty fourth");
check(constructedEmpty[4] === "o", "constructed empty fifth");

var constructedEmptyLimit = new String("hello").split(new RegExp, 3);
check(constructedEmptyLimit.length === 3, "constructed empty limit length");
check(constructedEmptyLimit[0] === "h", "constructed empty limit first");
check(constructedEmptyLimit[1] === "e", "constructed empty limit second");
check(constructedEmptyLimit[2] === "l", "constructed empty limit third");

var comma = new String("one-1,two-2,four-4").split(/,/);
check(comma.constructor === Array, "comma constructor");
check(comma.length === 3, "comma length");
check(comma[0] === "one-1", "comma first");
check(comma[1] === "two-2", "comma second");
check(comma[2] === "four-4", "comma third");

var whitespace = new String("a b c de f").split(/\s/);
check(whitespace.length === 5, "whitespace length");
check(whitespace[0] === "a", "whitespace first");
check(whitespace[1] === "b", "whitespace second");
check(whitespace[2] === "c", "whitespace third");
check(whitespace[3] === "de", "whitespace fourth");
check(whitespace[4] === "f", "whitespace fifth");

var digits = new String("dfe23iu 34 =+65--").split(/\d+/);
check(digits.length === 4, "digits length");
check(digits[0] === "dfe", "digits first");
check(digits[1] === "iu ", "digits second");
check(digits[2] === " =+", "digits third");
check(digits[3] === "--", "digits fourth");

var letters = new String("abc").split(/[a-z]/);
check(letters.length === 4, "letters length");
check(letters[0] === "", "letters first");
check(letters[1] === "", "letters second");
check(letters[2] === "", "letters third");
check(letters[3] === "", "letters fourth");

true;
