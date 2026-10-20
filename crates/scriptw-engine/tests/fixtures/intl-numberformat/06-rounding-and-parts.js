function check(value, message) { if (!value) throw new Error(message); }
const modes = [
  ["ceil", "1.3", "-1.2"], ["floor", "1.2", "-1.3"], ["expand", "1.3", "-1.3"], ["trunc", "1.2", "-1.2"],
  ["halfCeil", "1.3", "-1.2"], ["halfFloor", "1.2", "-1.3"], ["halfExpand", "1.3", "-1.3"], ["halfTrunc", "1.2", "-1.2"], ["halfEven", "1.2", "-1.2"]
];
for (const row of modes) {
  const nf = new Intl.NumberFormat("en-US", { useGrouping: false, minimumFractionDigits: 1, maximumFractionDigits: 1, roundingMode: row[0] });
  check(nf.format("1.25") === row[1] && nf.format("-1.25") === row[2], row[0]);
  for (const value of ["1.25", "-1.25"]) check(nf.formatToParts(value).map(part => part.value).join("") === nf.format(value), "parts " + row[0]);
}
const nickel = new Intl.NumberFormat("en-US", { useGrouping: false, minimumFractionDigits: 2, maximumFractionDigits: 2, roundingIncrement: 5, roundingMode: "halfEven" });
check(nickel.format("1.025") === "1.00" && nickel.format("1.075") === "1.10", "increment parity");
print("ok rounding and parts");
