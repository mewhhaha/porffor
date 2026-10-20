function check(value, message) { if (!value) throw new Error(message); }
function inspect(priority, ignoredFraction) {
  const events = [];
  function read(key, value) { events.push("get:" + key); return value; }
  function numeric(key, value, ignored) {
    return { valueOf() {
      events.push("number:" + key);
      if (ignored) throw new Error("ignored fraction coerced");
      return value;
    } };
  }
  function text(key, value) { return { toString() { events.push("string:" + key); return value; } }; }
  const options = {
    get minimumIntegerDigits() { return read("minimumIntegerDigits", numeric("minimumIntegerDigits", 1)); },
    get minimumFractionDigits() { return read("minimumFractionDigits", numeric("minimumFractionDigits", 2, ignoredFraction)); },
    get maximumFractionDigits() { return read("maximumFractionDigits", numeric("maximumFractionDigits", 2, ignoredFraction)); },
    get minimumSignificantDigits() { return read("minimumSignificantDigits", numeric("minimumSignificantDigits", 1)); },
    get maximumSignificantDigits() { return read("maximumSignificantDigits", numeric("maximumSignificantDigits", 3)); },
    get roundingIncrement() { return read("roundingIncrement", numeric("roundingIncrement", 1)); },
    get roundingMode() { return read("roundingMode", text("roundingMode", "halfEven")); },
    get roundingPriority() { return read("roundingPriority", text("roundingPriority", priority)); },
    get trailingZeroDisplay() { return read("trailingZeroDisplay", text("trailingZeroDisplay", "auto")); },
    get compactDisplay() { return read("compactDisplay", undefined); }
  };
  const nf = new Intl.NumberFormat("en-US", options);
  const expected = [
    "get:minimumIntegerDigits", "number:minimumIntegerDigits",
    "get:minimumFractionDigits", "get:maximumFractionDigits",
    "get:minimumSignificantDigits", "get:maximumSignificantDigits",
    "get:roundingIncrement", "number:roundingIncrement",
    "get:roundingMode", "string:roundingMode",
    "get:roundingPriority", "string:roundingPriority",
    "get:trailingZeroDisplay", "string:trailingZeroDisplay",
    "number:minimumSignificantDigits", "number:maximumSignificantDigits"
  ];
  if (!ignoredFraction) expected.push("number:minimumFractionDigits", "number:maximumFractionDigits");
  expected.push("get:compactDisplay");
  check(events.join(",") === expected.join(","), priority + " delayed coercion " + events.join(","));
  check(nf.format("1.25") === "1.25", "constructed formatter remains usable");
}
inspect("auto", true);
inspect("morePrecision", false);
print("ok digit coercion boundary");
