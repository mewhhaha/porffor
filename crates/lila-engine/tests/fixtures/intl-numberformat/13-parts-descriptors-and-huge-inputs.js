function check(value, message) { if (!value) throw new Error(message); }
function inspect(parts, range) {
  check(Object.getPrototypeOf(parts) === Array.prototype, "parts array prototype");
  for (const part of parts) {
    check(Object.getPrototypeOf(part) === Object.prototype, "part object prototype");
    const keys = Object.keys(part);
    check(keys.join(",") === (range ? "type,value,source" : "type,value"), "part keys");
    for (const key of keys) {
      const descriptor = Object.getOwnPropertyDescriptor(part, key);
      check(descriptor.writable && descriptor.enumerable && descriptor.configurable, "part field descriptor");
      check(descriptor.value === part[key] && typeof descriptor.value === "string", "part field value");
    }
  }
}
const nf = new Intl.NumberFormat("en-US", { useGrouping: false, maximumFractionDigits: 20 });
const huge = 12345678901234567890123456789012345678901234567890123456789012345678901234567890n;
const expected = "12345678901234567890123456789012345678901234567890123456789012345678901234567890";
check(nf.format(huge) === expected && nf.format(expected) === expected, "huge exact input transport");
for (const value of [huge, "9007199254740993", "1.0000000000000001", -0, NaN, Infinity]) {
  const parts = nf.formatToParts(value);
  inspect(parts, false);
  check(parts.map(part => part.value).join("") === nf.format(value), "scalar parts authority");
}
for (const endpoints of [["9007199254740993", "9007199254740995"], [3n, 1n], [-0, -0]]) {
  const parts = nf.formatRangeToParts(endpoints[0], endpoints[1]);
  inspect(parts, true);
  check(parts.map(part => part.value).join("") === nf.formatRange(endpoints[0], endpoints[1]), "range parts authority");
}
print("ok parts descriptors and huge inputs");
