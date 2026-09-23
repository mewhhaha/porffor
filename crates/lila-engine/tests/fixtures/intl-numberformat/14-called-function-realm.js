function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, action) {
  let caught;
  try { action(); } catch (error) { caught = error; }
  check(caught instanceof kind, "called-function exception Realm");
}
const other = __lilaCreateRealm().global;
const local = new Intl.NumberFormat("en-US");
const foreign = new other.Intl.NumberFormat("en-US");
const foreignPrototype = other.Intl.NumberFormat.prototype;
const foreignGetter = Object.getOwnPropertyDescriptor(foreignPrototype, "format").get;
const format = foreignGetter.call(local);
check(Object.getPrototypeOf(format) === other.Function.prototype, "bound formatter getter Realm");
check(format === local.format && format === foreignGetter.call(local), "bound formatter identity across getters");
throws(other.TypeError, () => format(Symbol()));
throws(other.TypeError, () => foreignGetter.call({}));
throws(other.RangeError, () => foreignPrototype.formatRange.call(local, NaN, 1));
throws(other.TypeError, () => foreignPrototype.formatRangeToParts.call({}, 1, 2));
const scalarParts = foreignPrototype.formatToParts.call(local, 12);
const rangeParts = foreignPrototype.formatRangeToParts.call(local, 1, 2);
for (const parts of [scalarParts, rangeParts]) {
  check(Object.getPrototypeOf(parts) === other.Array.prototype, "foreign method array Realm");
  for (const part of parts) check(Object.getPrototypeOf(part) === other.Object.prototype, "foreign method part Realm");
}
const localParts = Intl.NumberFormat.prototype.formatToParts.call(foreign, 12);
check(Object.getPrototypeOf(localParts) === Array.prototype, "receiver Realm does not select result array");
check(Object.getPrototypeOf(localParts[0]) === Object.prototype, "receiver Realm does not select result part");
check(Object.getPrototypeOf(foreignPrototype.resolvedOptions.call(local)) === other.Object.prototype, "resolvedOptions Realm");
check(Object.getPrototypeOf(other.Intl.NumberFormat.supportedLocalesOf(["en-US"])) === other.Array.prototype, "supportedLocalesOf Realm");
throws(other.TypeError, () => other.Number.prototype.toLocaleString.call(1, null));
throws(other.TypeError, () => other.BigInt.prototype.toLocaleString.call(1n, "en-US", { style: "currency" }));
print("ok called-function Realm");
