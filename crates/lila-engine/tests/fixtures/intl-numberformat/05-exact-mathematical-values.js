function check(value, message) { if (!value) throw new Error(message); }
const nf = new Intl.NumberFormat("en-US", { useGrouping: false, maximumFractionDigits: 20 });
for (const text of ["1.0000000000000001", "9007199254740993", "-987654321987654321"]) {
  check(nf.format(text) === text, "exact decimal string " + text);
}
check(nf.format(9007199254740993n) === "9007199254740993", "exact BigInt");
const pennies = new Intl.NumberFormat("en-US", { useGrouping: false, minimumFractionDigits: 2, maximumFractionDigits: 2 });
check(pennies.format(1.005) === "1.01", "Number shortest-decimal input");
check(nf.format(-0) === "-0" && nf.format("-0") === "-0", "negative zero");
const events = [];
check(nf.format({ [Symbol.toPrimitive](hint) { events.push(hint); return "9007199254740993"; } }) === "9007199254740993", "ToPrimitive exact string");
check(events.join(",") === "number", "one numeric primitive conversion");
print("ok exact mathematical values");
