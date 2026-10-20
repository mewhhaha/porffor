function check(value, message) { if (!value) throw new Error(message); }
const descriptor = Object.getOwnPropertyDescriptor(Intl, "NumberFormat");
check(descriptor !== undefined && typeof descriptor.value === "function", "missing Intl.NumberFormat");
check(descriptor.writable && !descriptor.enumerable && descriptor.configurable, "constructor descriptor");
const NF = descriptor.value;
check(NF.name === "NumberFormat" && NF.length === 0, "constructor metadata");
check(typeof NF.supportedLocalesOf === "function", "supportedLocalesOf");
const prototype = NF.prototype;
check(prototype.constructor === NF, "prototype constructor");
for (const method of ["formatToParts", "formatRange", "formatRangeToParts", "resolvedOptions"]) {
  check(typeof prototype[method] === "function", "method " + method);
}
check(typeof Object.getOwnPropertyDescriptor(prototype, "format").get === "function", "format getter");
check(Object.prototype.toString.call(new NF()) === "[object Intl.NumberFormat]", "instance tag");
print("ok intrinsic family");
