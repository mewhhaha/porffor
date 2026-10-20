function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, action) { let error; try { action(); } catch (caught) { error = caught; } check(error instanceof kind, "expected " + kind.name); }
throws(TypeError, () => new Intl.NumberFormat([], { style: "currency", unit: "invalid" }));
throws(RangeError, () => new Intl.NumberFormat([], { style: "unit", currency: "invalid" }));
throws(RangeError, () => new Intl.NumberFormat([], { roundingIncrement: 3 }));
throws(TypeError, () => new Intl.NumberFormat([], { roundingIncrement: 2, minimumSignificantDigits: 1 }));
throws(RangeError, () => new Intl.NumberFormat([], { roundingIncrement: 2, minimumFractionDigits: 1, maximumFractionDigits: 2 }));
const formatter = new Intl.NumberFormat("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2, useGrouping: false });
const format = formatter.format;
check(format === formatter.format && format.length === 1 && format.name === "", "cached bound formatter");
check(format.call(null, 12.5) === "12.50", "bound receiver");
throws(TypeError, () => new format(12.5));
throws(TypeError, () => Object.getOwnPropertyDescriptor(Intl.NumberFormat.prototype, "format").get.call({}));
const resolved = formatter.resolvedOptions();
check(resolved.minimumFractionDigits === 2 && resolved.maximumFractionDigits === 2, "digit slots");
check(resolved.useGrouping === false && resolved.roundingMode === "halfExpand", "closed option slots");
print("ok options and bound format");
