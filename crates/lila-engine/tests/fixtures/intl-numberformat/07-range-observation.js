function check(value, message) { if (!value) throw new Error(message); }
const nf = new Intl.NumberFormat("en-US", { useGrouping: false });
const events = [];
const first = { [Symbol.toPrimitive](hint) { events.push("first:" + hint); return NaN; } };
const second = { [Symbol.toPrimitive](hint) { events.push("second:" + hint); return 2; } };
let error;
try { nf.formatRange(first, second); } catch (caught) { error = caught; }
check(error instanceof RangeError, "NaN range rejection");
check(events.join(",") === "first:number,second:number", "both conversions before NaN check");
check(typeof nf.formatRange(3n, 1n) === "string", "descending range is admitted");
const parts = nf.formatRangeToParts("9007199254740993", "9007199254740995");
check(parts.map(part => part.value).join("") === nf.formatRange("9007199254740993", "9007199254740995"), "range parts");
check(parts.some(part => part.source === "startRange") && parts.some(part => part.source === "endRange"), "range sources");
print("ok range observation");
