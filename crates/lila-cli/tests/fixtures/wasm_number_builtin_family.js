function check(condition, label) {
  if (!condition) throw label;
}

function convert(value) {
  return Number(value);
}

function predicates(integer, fraction, unsafeInteger, notNumber, text) {
  return Number.isInteger(integer)
    && !Number.isInteger(fraction)
    && Number.isSafeInteger(integer)
    && !Number.isSafeInteger(unsafeInteger)
    && Number.isFinite(integer)
    && !Number.isFinite(Infinity)
    && Number.isNaN(notNumber)
    && !Number.isNaN(text);
}

function formats(value) {
  return value.toExponential(1) === "1.2e+1"
    && value.toFixed(1) === "12.0"
    && value.toPrecision(3) === "12.0"
    && value.toString(16) === "c"
    && value.toLocaleString() === "12";
}

check(Number() === 0, "Number() default");
check(convert("12") === 12, "Number(value)");

var boxed = new Number(12);
check(boxed.valueOf() === 12, "boxed Number valueOf");
check(
  predicates(12, 12.5, 9007199254740992, 0 / 0, "NaN"),
  "Number predicates"
);
check(formats(12), "Number prototype formatting");

var incompatibleReceiverThrew = false;
try {
  Number.prototype.valueOf.call({});
} catch (error) {
  incompatibleReceiverThrew = error instanceof TypeError;
}
check(incompatibleReceiverThrew, "Number prototype receiver");

true;
