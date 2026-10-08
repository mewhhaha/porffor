function check(value, message) { if (!value) throw new Error(message); }
function same(actual, expected, message) { check(Object.is(actual, expected), message); }
function throws(constructor, action, message) { let caught = false; try { action(); } catch (error) { caught = error instanceof constructor; } check(caught, message); }

const f = Intl.supportedValuesOf;
const descriptor = Object.getOwnPropertyDescriptor(Intl, "supportedValuesOf");
check(descriptor.writable && !descriptor.enumerable && descriptor.configurable, "method descriptor");
same(f.name, "supportedValuesOf", "name"); same(f.length, 1, "length");
check(!Object.hasOwn(f, "prototype"), "nonconstructor has no prototype property");
throws(TypeError, () => new f("unit"), "nonconstructible");
for (const key of [undefined, null, true, 7, "Calendar", "timezone", "", "calendars"]) {
  throws(RangeError, () => f(key), "invalid key: " + key);
}
throws(RangeError, () => f(), "missing argument is string undefined");
throws(TypeError, () => f(Symbol("timeZone")), "symbol ToString");
let calls = 0;
const key = { [Symbol.toPrimitive](hint) { calls++; same(hint, "string", "string hint"); return "timeZone"; } };
check(f.call(null, key).includes("UTC"), "coerced key ignores receiver"); same(calls, 1, "ToString exactly once");
const log = [];
f({ toString() { log.push("string"); return {}; }, valueOf() { log.push("value"); return "unit"; } });
same(log.join(","), "string,value", "ordinary string-hint order");

print("ok metadata_and_coercion");
262;
