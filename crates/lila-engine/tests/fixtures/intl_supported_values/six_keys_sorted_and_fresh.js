function check(value, message) { if (!value) throw new Error(message); }
function same(actual, expected, message) { check(Object.is(actual, expected), message); }
function throws(constructor, action, message) { let caught = false; try { action(); } catch (error) { caught = error instanceof constructor; } check(caught, message); }

const f = Intl.supportedValuesOf;
for (const key of ["calendar", "collation", "currency", "numberingSystem", "timeZone", "unit"]) {
  const first = f(key), second = f(key);
  check(Array.isArray(first), "Array result: " + key);
  same(Object.getPrototypeOf(first), Array.prototype, "Array prototype: " + key);
  check(first !== second, "fresh Array: " + key);
  for (let i = 0; i < first.length; i++) {
    same(typeof first[i], "string", "primitive elements");
    same(first[i], second[i], "stable catalogue");
    if (i) check(first[i - 1] < first[i], "sorted unique list");
    const d = Object.getOwnPropertyDescriptor(first, String(i));
    check(d.writable && d.enumerable && d.configurable, "ordinary Array element");
  }
  const before = second.join("|"); first.push("private-mutated-value");
  same(f(key).join("|"), before, "mutable result cannot mutate data");
}
check(f("calendar").includes("chinese"), "retain actually supported Chinese calendar");
check(f("currency").includes("USD") && f("currency").includes("EUR"), "reachable currency labels");
check(f("numberingSystem").includes("latn") && f("numberingSystem").includes("arab"), "real positional digit systems");
check(f("timeZone").includes("UTC") && f("timeZone").includes("America/New_York"), "primary time zones");
check(!f("timeZone").includes("US/Eastern"), "exclude zone alias");
check(f("unit").includes("meter") && !f("unit").includes("meter-per-second"), "single sanctioned units");

print("ok six_keys_sorted_and_fresh");
262;
