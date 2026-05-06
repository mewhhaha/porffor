function throwsRangeError(fn) {
  try {
    fn();
  } catch (err) {
    return err instanceof RangeError;
  }
  return false;
}

let zero = new Array(0);
let three = new Array(3);
let elements = Array(1, 2);
let oneString = new Array("3");

zero.length === 0
  && three.length === 3
  && !("0" in three)
  && elements.length === 2
  && elements[0] === 1
  && elements[1] === 2
  && oneString.length === 1
  && oneString[0] === "3"
  && throwsRangeError(function () { new Array(-1); })
  && throwsRangeError(function () { new Array(1.5); })
  && throwsRangeError(function () { new Array(4294967296); })
  && throwsRangeError(function () { new Array(NaN); })
  && throwsRangeError(function () { new Array(Infinity); });
