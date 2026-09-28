function same(actual, expected, label) {
  if (!Object.is(actual, expected)) {
    throw new Error(label + ": expected " + expected + ", got " + actual);
  }
}

for (var parse of [parseInt, Number.parseInt]) {
  // Every binary64 value this large has zero as its residue modulo 2^32.
  same(parse("0x10", 1e308), 16, "large positive radix and hex prefix");
  same(parse("10", -1e308), 10, "large negative radix");
  same(parse("10", 9007199254740994), 2, "positive residue above 2^53");
  same(parse("10", -9007199254740990), 2, "negative residue near 2^53");
  same(parse("10", 72057594037927950), 16, "rounded radix near 2^56");
  same(parse("10", -4294967292), 4, "negative residue");
  same(parse("0x10", NaN), 16, "NaN radix");
  same(parse("0x10", Infinity), 16, "positive infinity radix");
  same(parse("0x10", -Infinity), 16, "negative infinity radix");
  same(parse("10", 4294967297), NaN, "out-of-range residue");

  var trace = [];
  var string = { toString: function() { trace.push("string"); return "10"; } };
  var radix = { valueOf: function() { trace.push("radix"); return 4294967298; } };
  same(parse(string, radix), 2, "coerced radix residue");
  same(trace.join(","), "string,radix", "coercion order");

  var stringError = new Error("string conversion");
  trace = [];
  var threw = false;
  try {
    parse(
      { toString: function() { trace.push("string"); throw stringError; } },
      { valueOf: function() { trace.push("radix"); return 10; } }
    );
  } catch (error) {
    same(error, stringError, "string conversion error");
    threw = true;
  }
  same(threw, true, "string conversion must throw");
  same(trace.join(","), "string", "radix conversion after string error");

  var radixError = new Error("radix conversion");
  trace = [];
  threw = false;
  try {
    parse(
      { toString: function() { trace.push("string"); return "10"; } },
      { valueOf: function() { trace.push("radix"); throw radixError; } }
    );
  } catch (error) {
    same(error, radixError, "radix conversion error");
    threw = true;
  }
  same(threw, true, "radix conversion must throw");
  same(trace.join(","), "string,radix", "radix conversion after string coercion");
}

true;
